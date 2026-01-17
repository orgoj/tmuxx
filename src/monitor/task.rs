use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;
use tracing::{debug, error, warn};

use crate::agents::MonitoredAgent;
use crate::app::AgentTree;
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
}

impl MonitorTask {
    pub fn new(
        tmux_client: Arc<TmuxClient>,
        parser_registry: Arc<ParserRegistry>,
        tx: mpsc::Sender<MonitorUpdate>,
        poll_interval: Duration,
    ) -> Self {
        Self {
            tmux_client,
            parser_registry,
            tx,
            poll_interval,
        }
    }

    /// Runs the monitoring loop
    pub async fn run(&self) {
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

    async fn poll_agents(&self) -> anyhow::Result<AgentTree> {
        // Refresh process cache once per poll cycle (much faster than per-pane)
        refresh_process_cache();

        let panes = self.tmux_client.list_panes()?;
        let mut tree = AgentTree::new();

        for pane in panes {
            // Try to find a matching parser for the pane (checks command, title, cmdline)
            if let Some(parser) = self.parser_registry.find_parser_for_pane(&pane) {
                let target = pane.target();

                // Capture pane content
                let content = match self.tmux_client.capture_pane(&target) {
                    Ok(c) => c,
                    Err(e) => {
                        error!("Failed to capture pane {}: {}", target, e);
                        continue;
                    }
                };

                // Parse status
                let status = parser.parse_status(&content);

                // Parse subagents
                let subagents = parser.parse_subagents(&content);

                // Create monitored agent
                let mut agent = MonitoredAgent::new(
                    format!("{}-{}", target, pane.pid),
                    target,
                    pane.session.clone(),
                    pane.window,
                    pane.window_name.clone(),
                    pane.pane,
                    pane.path.clone(),
                    parser.agent_type(),
                    pane.pid,
                );
                agent.status = status;
                agent.subagents = subagents;
                agent.last_content = content;

                tree.root_agents.push(agent);
            }
        }

        // Sort agents by target for consistent ordering
        tree.root_agents.sort_by(|a, b| a.target.cmp(&b.target));

        Ok(tree)
    }
}
