use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::mpsc;
use tracing::{debug, error, warn};

use crate::agents::{AgentStatus, MonitoredAgent};
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
}

impl MonitorTask {
    pub fn new(
        tmux_client: Arc<TmuxClient>,
        parser_registry: Arc<ParserRegistry>,
        tx: mpsc::Sender<MonitorUpdate>,
        poll_interval: Duration,
        config: Config,
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

                // Create monitored agent
                let mut agent = MonitoredAgent::new(
                    format!("{}-{}", target, pane.pid),
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
                agent.touch(); // Update last_updated

                tree.root_agents.push(agent);
            }
        }

        // Sort agents by target for consistent ordering
        tree.root_agents.sort_by(|a, b| a.target.cmp(&b.target));

        Ok(tree)
    }
}
