use std::collections::BTreeMap;

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::agents::{AgentStatus, AgentType, ApprovalType, MonitoredAgent, SubagentStatus};
use crate::app::AppState;

/// Widget for displaying agents in a tree organized by session/window
pub struct AgentTreeWidget;

/// Represents the hierarchical structure: Session -> Window -> Agents
struct SessionWindowTree<'a> {
    sessions: BTreeMap<&'a str, BTreeMap<(u32, &'a str), Vec<(usize, &'a MonitoredAgent)>>>,
}

impl<'a> SessionWindowTree<'a> {
    fn new(agents: &'a [MonitoredAgent]) -> Self {
        let mut sessions: BTreeMap<&str, BTreeMap<(u32, &str), Vec<(usize, &MonitoredAgent)>>> =
            BTreeMap::new();

        for (idx, agent) in agents.iter().enumerate() {
            sessions
                .entry(&agent.session)
                .or_default()
                .entry((agent.window, &agent.window_name))
                .or_default()
                .push((idx, agent));
        }

        Self { sessions }
    }
}

impl AgentTreeWidget {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        let agents = &state.agents.root_agents;
        let active_count = state.agents.active_count();
        let subagent_count = state.agents.running_subagent_count();
        let selected_count = state.selected_agents.len();

        // Build title with counts
        let title = if selected_count > 0 {
            format!(" {} sel, {} pending ", selected_count, active_count)
        } else if subagent_count > 0 {
            format!(" {} pending, {} subagents ", active_count, subagent_count)
        } else {
            format!(" {} pending ", active_count)
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Gray));

        if agents.is_empty() {
            let empty_text = List::new(vec![ListItem::new(Line::from(vec![Span::styled(
                "  No agents detected",
                Style::default().fg(Color::DarkGray),
            )]))])
            .block(block);
            frame.render_widget(empty_text, area);
            return;
        }

        let tree = SessionWindowTree::new(agents);
        let mut items: Vec<ListItem> = Vec::new();
        let available_width = area.width.saturating_sub(4) as usize;

        for (session, windows) in tree.sessions.iter() {
            // Session header
            let session_line = Line::from(vec![
                Span::styled("▼ ", Style::default().fg(Color::Cyan)),
                Span::styled(*session, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]);
            items.push(ListItem::new(session_line));

            for (window_idx, ((window_num, window_name), window_agents)) in windows.iter().enumerate() {
                let is_last_window = window_idx == windows.len() - 1;
                let window_prefix = if is_last_window { "└─" } else { "├─" };

                // Window header
                let window_line = Line::from(vec![
                    Span::styled(format!(" {}", window_prefix), Style::default().fg(Color::DarkGray)),
                    Span::raw(" "),
                    Span::styled(
                        format!("{}: {}", window_num, window_name),
                        Style::default().fg(Color::White),
                    ),
                ]);
                items.push(ListItem::new(window_line));

                for (agent_idx, (original_idx, agent)) in window_agents.iter().enumerate() {
                    let is_cursor = *original_idx == state.selected_index;
                    let is_selected = state.is_multi_selected(*original_idx);
                    let is_last_agent = agent_idx == window_agents.len() - 1;

                    // Continuation prefix for subsequent lines
                    let cont_prefix = if is_last_window {
                        "    "
                    } else {
                        " │  "
                    };

                    // Tree prefix
                    let tree_prefix = if is_last_window {
                        if is_last_agent && agent.subagents.is_empty() { "    └─" } else { "    ├─" }
                    } else {
                        if is_last_agent && agent.subagents.is_empty() { " │  └─" } else { " │  ├─" }
                    };

                    // Selection/cursor indicator
                    let select_indicator = if is_selected && is_cursor {
                        "▸●"
                    } else if is_selected {
                        " ●"
                    } else if is_cursor {
                        "▸ "
                    } else {
                        "  "
                    };

                    // Status indicator with color
                    let (status_char, status_style) = match &agent.status {
                        AgentStatus::Idle => ("●", Style::default().fg(Color::Green)),
                        AgentStatus::Processing { .. } => ("◐", Style::default().fg(Color::Yellow)),
                        AgentStatus::AwaitingApproval { .. } => (
                            "⚠",
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        ),
                        AgentStatus::Error { .. } => ("✗", Style::default().fg(Color::Red)),
                        AgentStatus::Unknown => ("○", Style::default().fg(Color::DarkGray)),
                    };

                    // Agent type color
                    let type_style = match agent.agent_type {
                        AgentType::ClaudeCode => Style::default().fg(Color::Magenta),
                        AgentType::OpenCode => Style::default().fg(Color::Blue),
                        AgentType::CodexCli => Style::default().fg(Color::Green),
                        AgentType::GeminiCli => Style::default().fg(Color::Yellow),
                        AgentType::Unknown => Style::default().fg(Color::DarkGray),
                    };

                    // Main agent line: abbreviated path
                    let line = Line::from(vec![
                        Span::styled(select_indicator, if is_selected {
                            Style::default().fg(Color::Cyan)
                        } else {
                            Style::default().fg(Color::White)
                        }),
                        Span::styled(tree_prefix, Style::default().fg(Color::DarkGray)),
                        Span::styled(status_char, status_style),
                        Span::raw(" "),
                        Span::styled(agent.abbreviated_path(), Style::default().fg(Color::Cyan)),
                    ]);

                    let item_style = if is_cursor {
                        Style::default().bg(Color::DarkGray)
                    } else if is_selected {
                        Style::default().bg(Color::Rgb(40, 40, 60))
                    } else {
                        Style::default()
                    };

                    items.push(ListItem::new(line).style(item_style));

                    // Agent type line
                    let type_line = Line::from(vec![
                        Span::raw("  "),
                        Span::styled(format!("{}│  ", cont_prefix), Style::default().fg(Color::DarkGray)),
                        Span::styled(agent.agent_type.display_name(), type_style),
                    ]);
                    items.push(ListItem::new(type_line).style(item_style));

                    // Status detail line (if has meaningful status)
                    match &agent.status {
                        AgentStatus::AwaitingApproval { approval_type, details } => {
                            // Show approval type
                            let approval_line = Line::from(vec![
                                Span::raw("  "),
                                Span::styled(format!("{}│  ", cont_prefix), Style::default().fg(Color::DarkGray)),
                                Span::styled("⚠ ", Style::default().fg(Color::Red)),
                                Span::styled(
                                    format!("{}", approval_type),
                                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                                ),
                            ]);
                            items.push(ListItem::new(approval_line).style(item_style));

                            // Show details if available
                            if !details.is_empty() {
                                let detail_text = truncate_str(details, available_width.saturating_sub(12));
                                let detail_line = Line::from(vec![
                                    Span::raw("  "),
                                    Span::styled(format!("{}│  ", cont_prefix), Style::default().fg(Color::DarkGray)),
                                    Span::styled("  → ", Style::default().fg(Color::DarkGray)),
                                    Span::styled(detail_text, Style::default().fg(Color::White)),
                                ]);
                                items.push(ListItem::new(detail_line).style(item_style));
                            }

                            // Show choices for UserQuestion
                            if let ApprovalType::UserQuestion { choices, .. } = approval_type {
                                for (i, choice) in choices.iter().take(4).enumerate() {
                                    let choice_text = truncate_str(choice, available_width.saturating_sub(14));
                                    let choice_line = Line::from(vec![
                                        Span::raw("  "),
                                        Span::styled(format!("{}│  ", cont_prefix), Style::default().fg(Color::DarkGray)),
                                        Span::styled(format!("  {}. ", i + 1), Style::default().fg(Color::Yellow)),
                                        Span::styled(choice_text, Style::default().fg(Color::White)),
                                    ]);
                                    items.push(ListItem::new(choice_line).style(item_style));
                                }
                                if choices.len() > 4 {
                                    let more_line = Line::from(vec![
                                        Span::raw("  "),
                                        Span::styled(format!("{}│  ", cont_prefix), Style::default().fg(Color::DarkGray)),
                                        Span::styled(
                                            format!("     ...+{} more", choices.len() - 4),
                                            Style::default().fg(Color::DarkGray),
                                        ),
                                    ]);
                                    items.push(ListItem::new(more_line).style(item_style));
                                }
                            }
                        }
                        AgentStatus::Processing { activity } => {
                            if !activity.is_empty() {
                                let activity_text = truncate_str(activity, available_width.saturating_sub(12));
                                let activity_line = Line::from(vec![
                                    Span::raw("  "),
                                    Span::styled(format!("{}│  ", cont_prefix), Style::default().fg(Color::DarkGray)),
                                    Span::styled("◐ ", Style::default().fg(Color::Yellow)),
                                    Span::styled(activity_text, Style::default().fg(Color::Yellow)),
                                ]);
                                items.push(ListItem::new(activity_line).style(item_style));
                            }
                        }
                        AgentStatus::Error { message } => {
                            let error_text = truncate_str(message, available_width.saturating_sub(12));
                            let error_line = Line::from(vec![
                                Span::raw("  "),
                                Span::styled(format!("{}│  ", cont_prefix), Style::default().fg(Color::DarkGray)),
                                Span::styled("✗ ", Style::default().fg(Color::Red)),
                                Span::styled(error_text, Style::default().fg(Color::Red)),
                            ]);
                            items.push(ListItem::new(error_line).style(item_style));
                        }
                        _ => {}
                    }

                    // Show subagents
                    for (sub_idx, subagent) in agent.subagents.iter().enumerate() {
                        let is_last_sub = sub_idx == agent.subagents.len() - 1;
                        let sub_branch = if is_last_sub { "└─" } else { "├─" };

                        let (sub_char, sub_style) = match subagent.status {
                            SubagentStatus::Running => ("▶", Style::default().fg(Color::Cyan)),
                            SubagentStatus::Completed => ("✓", Style::default().fg(Color::Green)),
                            SubagentStatus::Failed => ("✗", Style::default().fg(Color::Red)),
                            SubagentStatus::Unknown => ("?", Style::default().fg(Color::DarkGray)),
                        };

                        // Duration for running subagents
                        let duration = if matches!(subagent.status, SubagentStatus::Running) {
                            format!(" ({})", subagent.duration_str())
                        } else {
                            String::new()
                        };

                        let sub_line = Line::from(vec![
                            Span::raw("  "),
                            Span::styled(format!("{}{}", cont_prefix, sub_branch), Style::default().fg(Color::DarkGray)),
                            Span::styled(sub_char, sub_style),
                            Span::raw(" "),
                            Span::styled(
                                subagent.subagent_type.display_name(),
                                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(duration, Style::default().fg(Color::Yellow)),
                        ]);
                        items.push(ListItem::new(sub_line));

                        // Show subagent description
                        if !subagent.description.is_empty() {
                            let desc_prefix = if is_last_sub { "   " } else { "│  " };
                            let desc_text = truncate_str(&subagent.description, available_width.saturating_sub(14));
                            let desc_line = Line::from(vec![
                                Span::raw("  "),
                                Span::styled(format!("{}{}", cont_prefix, desc_prefix), Style::default().fg(Color::DarkGray)),
                                Span::styled("  ", Style::default()),
                                Span::styled(desc_text, Style::default().fg(Color::DarkGray)),
                            ]);
                            items.push(ListItem::new(desc_line));
                        }
                    }
                }
            }
        }

        let list = List::new(items).block(block);
        let mut list_state = ListState::default();
        list_state.select(Some(state.selected_index));
        frame.render_stateful_widget(list, area, &mut list_state);
    }
}

/// Truncate a string to a maximum length, adding ".." if truncated
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        format!("{}..", s.chars().take(max_len.saturating_sub(2)).collect::<String>())
    }
}
