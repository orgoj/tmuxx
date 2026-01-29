use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use regex::Regex;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::agents::{AgentStatus, MonitoredAgent};
use crate::app::config::NotificationMode;
use crate::app::{AgentTree, Config};
use crate::parsers::ParserRegistry;
use crate::tmux::{refresh_process_cache, TmuxClient};

/// Update message sent from monitor to UI
#[derive(Debug, Clone)]
pub struct MonitorUpdate {
    pub agents: AgentTree,
}

/// Background task that monitors tmux panes for AI agents
pub struct MonitorTask {
    tmux_client: Arc<TmuxClient>,
    parser_registry: Arc<ParserRegistry>,
    tx: mpsc::Sender<MonitorUpdate>,
    poll_interval: Duration,
    /// Configuration for session filtering
    config: Config,
    /// Current session name (for ignore_self feature)
    current_session: Option<String>,
    /// Track when each agent was last seen as "active" (Processing/AwaitingApproval)
    /// Key: agent target string
    last_active: HashMap<String, Instant>,
    /// When approval was first detected (by target)
    approval_since: HashMap<String, Instant>,
    /// Agents already notified in "each" mode (by target)
    notified_agents: HashSet<String>,
    /// Global flag for "first" mode
    global_notification_sent: bool,
    /// Shared flag - UI sets true on interaction, monitor reads and clears
    user_interacted: Arc<AtomicBool>,
}

impl MonitorTask {
    pub fn new(
        tmux_client: Arc<TmuxClient>,
        parser_registry: Arc<ParserRegistry>,
        tx: mpsc::Sender<MonitorUpdate>,
        poll_interval: Duration,
        config: Config,
        user_interacted: Arc<AtomicBool>,
    ) -> Self {
        // Get current session once at startup (for ignore_self feature)
        let current_session = tmux_client.get_current_session().ok().flatten();

        Self {
            tmux_client,
            parser_registry,
            tx,
            poll_interval,
            config,
            current_session,
            last_active: HashMap::new(),
            approval_since: HashMap::new(),
            notified_agents: HashSet::new(),
            global_notification_sent: false,
            user_interacted,
        }
    }

    /// Runs the monitoring loop
    pub async fn run(mut self) {
        loop {
            match self.poll_agents().await {
                Ok(tree) => {
                    let update = MonitorUpdate { agents: tree };
                    if self.tx.send(update).await.is_err() {
                        debug!("Monitor channel closed, stopping");
                        break;
                    }
                }
                Err(e) => {
                    warn!("Monitor poll error: {}", e);
                }
            }

            tokio::time::sleep(self.poll_interval).await;
        }
    }

    async fn poll_agents(&mut self) -> anyhow::Result<AgentTree> {
        // Refresh process cache once per poll cycle (much faster than per-pane)
        refresh_process_cache();

        let panes = self.tmux_client.list_panes()?;
        let mut tree = AgentTree::new();

        for pane in panes {
            // Filter out ignored sessions (before any processing)
            if self
                .config
                .should_ignore_session(&pane.session, self.current_session.as_deref())
            {
                debug!("Ignoring session: {}", pane.session);
                continue;
            }

            // Find suitable parser (possibly checking content)
            let candidates = self.parser_registry.find_candidates_for_pane(&pane);
            if candidates.is_empty() {
                continue;
            }

            let mut selected_parser = None;
            let mut captured_content = None;
            let target = pane.target();

            for parser in candidates {
                if parser.requires_content_check() {
                    if captured_content.is_none() {
                        match self.tmux_client.capture_pane(&target) {
                            Ok(c) => captured_content = Some(c),
                            Err(e) => {
                                error!("Failed to capture pane {}: {}", target, e);
                                continue;
                            }
                        }
                    }
                    if let Some(content) = &captured_content {
                        if parser.match_content(content) {
                            selected_parser = Some(parser);
                            break;
                        }
                    }
                } else {
                    selected_parser = Some(parser);
                    break;
                }
            }

            if let Some(parser) = selected_parser {
                let content = if let Some(c) = captured_content {
                    c
                } else {
                    match self.tmux_client.capture_pane(&target) {
                        Ok(c) => c,
                        Err(e) => {
                            error!("Failed to capture pane {}: {}", target, e);
                            continue;
                        }
                    }
                };

                // Parse status from content
                let mut status = parser.parse_status(&content);

                // apply hysteresis: if status is now Idle but was recently active, keep as Processing
                let now = Instant::now();
                let is_active = matches!(
                    status,
                    AgentStatus::Processing { .. } | AgentStatus::AwaitingApproval { .. }
                );

                if is_active {
                    // Update last active time
                    self.last_active.insert(target.clone(), now);
                } else if matches!(status, AgentStatus::Idle { .. }) {
                    // Check if we were recently active
                    if let Some(last) = self.last_active.get(&target) {
                        let hysteresis = Duration::from_millis(self.config.timing.hysteresis_ms);
                        if now.duration_since(*last) < hysteresis {
                            // Keep as Processing to avoid flicker
                            status = AgentStatus::Processing {
                                activity: "Working...".to_string(),
                            };
                        }
                    }
                }

                // Parse subagents
                let subagents = parser.parse_subagents(&content);

                // Parse context remaining
                let context_remaining = parser.parse_context_remaining(&content);

                // Calculate process indicators
                let mut active_indicators = Vec::new();
                for indicator in parser.process_indicators() {
                    match Regex::new(&indicator.ancestor_pattern) {
                        Ok(re) => {
                            for cmd in &pane.ancestor_commands {
                                if re.is_match(cmd) {
                                    active_indicators.push(indicator.icon.clone());
                                    break;
                                }
                            }
                        }
                        Err(e) => warn!("Invalid process indicator regex: {}", e),
                    }
                }

                // Create monitored agent
                let mut agent = MonitoredAgent::new(
                    format!("{}-{}", target, pane.pid),
                    parser.agent_id().to_string(),
                    parser.agent_name().to_string(),
                    parser.agent_color().map(|s| s.to_string()),
                    target,
                    pane.session.clone(),
                    pane.window,
                    pane.window_name.clone(),
                    pane.pane,
                    pane.path.clone(),
                    parser.agent_type(),
                    parser.agent_background_color().map(|s| s.to_string()),
                    pane.pid,
                );
                agent.status = status;
                agent.subagents = subagents;
                agent.last_content = content;
                agent.context_remaining = context_remaining;
                agent.active_indicators = active_indicators;
                agent.touch(); // Update last_updated

                tree.root_agents.push(agent);
            }
        }

        // Sort agents by target for consistent ordering
        tree.root_agents.sort_by(|a, b| a.target.cmp(&b.target));

        // Notification logic
        self.handle_notifications(&tree);

        Ok(tree)
    }

    /// Handle desktop notifications for agents awaiting approval
    fn handle_notifications(&mut self, tree: &AgentTree) {
        // Only if notification_command is configured
        if self.config.notification_command.is_none() {
            self.global_notification_sent = false;
            self.notified_agents.clear();
            return;
        }

        let awaiting: Vec<_> = tree
            .root_agents
            .iter()
            .filter(|a| a.status.needs_attention())
            .collect();

        let now = Instant::now();

        // Check if UI cleared the flag (user interacted)
        if self.user_interacted.swap(false, Ordering::Relaxed) {
            self.global_notification_sent = false;
            self.notified_agents.clear();
            // Reset timers so delay starts again from now after interaction
            for since in self.approval_since.values_mut() {
                *since = now;
            }
        }

        if awaiting.is_empty() {
            self.approval_since.clear();
            // Reset state so that new approvals trigger notification again after delay
            self.global_notification_sent = false;
            self.notified_agents.clear();
            return;
        }

        // Track when approval started for each agent
        for agent in &awaiting {
            self.approval_since
                .entry(agent.target.clone())
                .or_insert(now);
        }
        // Remove tracking for cleared agents
        self.approval_since
            .retain(|t, _| awaiting.iter().any(|a| &a.target == t));

        // Notification logic per mode
        match self.config.notification_mode {
            NotificationMode::First => {
                if !self.global_notification_sent {
                    for agent in &awaiting {
                        if let Some(since) = self.approval_since.get(&agent.target) {
                            if since.elapsed().as_millis()
                                >= self.config.notification_delay_ms as u128
                            {
                                self.send_notification(agent, awaiting.len());
                                self.global_notification_sent = true;
                                break;
                            }
                        }
                    }
                }
            }
            NotificationMode::Each => {
                for agent in &awaiting {
                    if !self.notified_agents.contains(&agent.target) {
                        if let Some(since) = self.approval_since.get(&agent.target) {
                            if since.elapsed().as_millis()
                                >= self.config.notification_delay_ms as u128
                            {
                                self.send_notification(agent, awaiting.len());
                                self.notified_agents.insert(agent.target.clone());
                            }
                        }
                    }
                }
                // Clear notified status for agents no longer awaiting
                self.notified_agents
                    .retain(|t| awaiting.iter().any(|a| &a.target == t));
            }
        }
    }

    /// Escape a string for safe use in shell commands (single-quote escaping)
    fn shell_escape(s: &str) -> String {
        // Replace single quotes with '\'' (end quote, escaped quote, start quote)
        format!("'{}'", s.replace('\'', "'\\''"))
    }

    /// Send a desktop notification for an agent
    fn send_notification(&self, agent: &MonitoredAgent, count: usize) {
        let cmd_template = match &self.config.notification_command {
            Some(cmd) => cmd,
            None => return,
        };

        let approval_type = match &agent.status {
            AgentStatus::AwaitingApproval { approval_type, .. } => approval_type.short_desc(),
            AgentStatus::Error { .. } => "error",
            _ => "attention",
        };

        // Shell-escape all dynamic placeholder values to prevent injection
        let cmd = cmd_template
            .replace("{title}", "tmuxx") // Hardcoded, safe
            .replace(
                "{message}",
                &Self::shell_escape(&format!("{} needs approval", agent.name)),
            )
            .replace("{agent}", &Self::shell_escape(&agent.name))
            .replace("{session}", &Self::shell_escape(&agent.session))
            .replace("{target}", &Self::shell_escape(&agent.target))
            .replace("{path}", &Self::shell_escape(&agent.path))
            .replace("{approval_type}", approval_type) // Internal enum, safe
            .replace("{count}", &count.to_string()); // Number, safe

        debug!("Sending notification: {}", cmd);

        match std::process::Command::new("bash")
            .args(["-c", &cmd])
            .spawn()
        {
            Ok(_) => info!("Notification sent for agent: {}", agent.name),
            Err(e) => warn!("Failed to spawn notification command: {}", e),
        }
    }
}
