use std::collections::BTreeMap;

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
    Frame,
};

use crate::agents::{AgentStatus, ApprovalType, MonitoredAgent, SubagentStatus};
use crate::app::AppState;

/// Widget for displaying agents in a tree organized by session/window
pub struct AgentTreeWidget;

/// Type alias for window key (window number, window name)
type WindowKey<'a> = (u32, &'a str);

/// Type alias for agents in a window (index, agent reference)
type WindowAgents<'a> = Vec<(usize, &'a MonitoredAgent)>;

/// Type alias for windows map
type WindowsMap<'a> = BTreeMap<WindowKey<'a>, WindowAgents<'a>>;

/// Type alias for sessions map
type SessionsMap<'a> = BTreeMap<&'a str, WindowsMap<'a>>;

/// Represents the hierarchical structure: Session -> Window -> Agents
struct SessionWindowTree<'a> {
    sessions: SessionsMap<'a>,
}

impl<'a> SessionWindowTree<'a> {
    fn new(agents: &[&'a MonitoredAgent]) -> Self {
        let mut sessions: SessionsMap<'a> = BTreeMap::new();

        for (idx, agent) in agents.iter().enumerate() {
            sessions
                .entry(&agent.session)
                .or_default()
                .entry((agent.window, &agent.window_name))
                .or_default()
                .push((idx, *agent));
        }

        Self { sessions }
    }
}

impl AgentTreeWidget {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        // Get filtered agents (returns Vec<&MonitoredAgent>)
        let filtered_agents = state.filtered_agents();
        let _agents = &state.agents.root_agents; // Keep for counts
        let active_count = state.agents.active_count();
        let subagent_count = state.agents.running_subagent_count();
        let selected_count = state.selected_agents.len();

        // Build title (show filtered count)
        let title = if selected_count > 0 {
            format!(" {} sel │ {} pending ", selected_count, active_count)
        } else if subagent_count > 0 {
            format!(" {} pending │ {} subs ", active_count, subagent_count)
        } else if active_count > 0 {
            format!(" ⚠ {} pending ", active_count)
        } else {
            format!(" {} agents ", filtered_agents.len())
        };

        let border_color = if !state.is_input_focused() {
            Color::Cyan
        } else {
            Color::Gray
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color));

        if filtered_agents.is_empty() {
            let empty_msg = if let Some(pattern) = &state.filter_pattern {
                format!("  No agents match filter '{}'", pattern)
            } else {
                "  No agents detected".to_string()
            };
            let empty_text = List::new(vec![ListItem::new(Line::from(vec![Span::styled(
                empty_msg,
                Style::default().fg(Color::DarkGray),
            )]))])
            .block(block);
            frame.render_widget(empty_text, area);
            return;
        }

        let tree = SessionWindowTree::new(&filtered_agents);
        let mut items: Vec<ListItem> = Vec::new();
        let available_width = area.width.saturating_sub(4) as usize;

        for (session, windows) in tree.sessions.iter() {
            // Session header
            let session_line = Line::from(vec![
                Span::styled("▼ ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    *session,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]);
            items.push(ListItem::new(session_line));

            for (window_idx, ((window_num, window_name), window_agents)) in
                windows.iter().enumerate()
            {
                let is_last_window = window_idx == windows.len() - 1;
                let window_prefix = if is_last_window { "└─" } else { "├─" };

                // Window header
                let window_line = Line::from(vec![
                    Span::styled(
                        format!(" {} ", window_prefix),
                        Style::default().fg(Color::DarkGray),
                    ),
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

                    let cont_prefix = if is_last_window { "    " } else { " │  " };

                    let tree_prefix = if is_last_window {
                        if is_last_agent && agent.subagents.is_empty() {
                            "    └─"
                        } else {
                            "    ├─"
                        }
                    } else if is_last_agent && agent.subagents.is_empty() {
                        " │  └─"
                    } else {
                        " │  ├─"
                    };

                    let select_indicator = if is_selected && is_cursor {
                        "▶☑" // Cursor + multi-selected
                    } else if is_selected {
                        "  ☑"
                    } else if is_cursor {
                        "▶ " // Cursor indicator
                    } else {
                        "   "
                    };

                    // Status indicator and text
                    let (status_char, status_text, status_style) = match &agent.status {
                        AgentStatus::Idle => ("●", "Idle", Style::default().fg(Color::Green)),
                        AgentStatus::Processing { .. } => (
                            state.spinner_frame(),
                            "Working",
                            Style::default().fg(Color::Yellow),
                        ),
                        AgentStatus::AwaitingApproval { .. } => (
                            "⚠",
                            "Waiting",
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        ),
                        AgentStatus::Error { .. } => {
                            ("✗", "Error", Style::default().fg(Color::Red))
                        }
                        AgentStatus::Tui { name } => ("○", name.as_str(), Style::default().fg(Color::Blue)),
                        AgentStatus::Unknown => {
                            ("○", "Unknown", Style::default().fg(Color::DarkGray))
                        }
                    };

                    let type_style = if let Some(c) = &agent.color {
                        Style::default().fg(parse_color(c))
                    } else {
                        Style::default().fg(parse_color(&state.config.agent_name_color))
                    };

                    let path_style = Style::default().fg(Color::Cyan);
                    let divider_style = Style::default().fg(Color::DarkGray);

                    let item_style = if is_cursor {
                        Style::default()
                            .bg(parse_color(&state.config.current_item_bg_color))
                            .add_modifier(Modifier::BOLD)
                    } else if is_selected {
                        if let Some(bg) = &state.config.multi_selection_bg_color {
                            Style::default().bg(parse_color(bg))
                        } else if let Some(bg_color) = &agent.background_color {
                            Style::default().bg(parse_color(bg_color))
                        } else {
                            Style::default()
                        }
                    } else if let Some(bg_color) = &agent.background_color {
                        Style::default().bg(parse_color(bg_color))
                    } else {
                        Style::default()
                    };

                    // Main line: status + path
                    let line = Line::from(vec![
                        Span::styled(
                            select_indicator,
                            if is_cursor {
                                Style::default()
                                    .fg(Color::Rgb(0, 100, 200))
                                    .add_modifier(Modifier::BOLD)
                            } else if is_selected {
                                Style::default().fg(Color::Cyan)
                            } else {
                                Style::default().fg(Color::White)
                            },
                        ),
                        Span::styled(tree_prefix, divider_style),
                        Span::styled(status_char, status_style),
                        Span::raw(" "),
                        Span::styled(agent.abbreviated_path(), path_style),
                    ]);
                    items.push(ListItem::new(line).style(item_style));

                    // Info line: type | status | pid | uptime | context
                    let mut info_parts = vec![
                        Span::raw("  "),
                        Span::styled(format!("{}│  ", cont_prefix), divider_style),
                        Span::styled(agent.name.clone(), type_style),
                        Span::styled(" │ ", divider_style),
                        Span::styled(status_text, status_style),
                        Span::styled(" │ ", divider_style),
                        Span::styled(format!("pid:{}", agent.pid), divider_style),
                        Span::styled(" │ ", divider_style),
                        Span::styled(agent.uptime_str(), divider_style),
                    ];

                    // Context bar if available
                    if let Some(ctx) = agent.context_remaining {
                        let bar_color = if ctx > 50 {
                            Color::Green
                        } else if ctx > 20 {
                            Color::Yellow
                        } else {
                            Color::Red
                        };
                        info_parts.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
                        info_parts.push(Span::styled(
                            context_bar(ctx),
                            Style::default().fg(bar_color),
                        ));
                    }

                    items.push(ListItem::new(Line::from(info_parts)).style(item_style));

                    // Status details
                    match &agent.status {
                        AgentStatus::AwaitingApproval {
                            approval_type,
                            details,
                        } => {
                            let approval_line = Line::from(vec![
                                Span::raw("  "),
                                Span::styled(
                                    format!("{}│  ", cont_prefix),
                                    Style::default().fg(Color::DarkGray),
                                ),
                                Span::styled("⚠ ", Style::default().fg(Color::Red)),
                                Span::styled(
                                    format!("{}", approval_type),
                                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                                ),
                            ]);
                            items.push(ListItem::new(approval_line).style(item_style));

                            if !details.is_empty() {
                                let detail_text =
                                    truncate_str(details, available_width.saturating_sub(14));
                                let detail_line = Line::from(vec![
                                    Span::raw("  "),
                                    Span::styled(
                                        format!("{}│  ", cont_prefix),
                                        Style::default().fg(Color::DarkGray),
                                    ),
                                    Span::styled("  → ", Style::default().fg(Color::DarkGray)),
                                    Span::styled(detail_text, Style::default().fg(Color::White)),
                                ]);
                                items.push(ListItem::new(detail_line).style(item_style));
                            }

                            if let ApprovalType::UserQuestion { choices, .. } = approval_type {
                                for (i, choice) in choices.iter().take(4).enumerate() {
                                    let choice_text =
                                        truncate_str(choice, available_width.saturating_sub(14));
                                    let choice_line = Line::from(vec![
                                        Span::raw("  "),
                                        Span::styled(
                                            format!("{}│  ", cont_prefix),
                                            Style::default().fg(Color::DarkGray),
                                        ),
                                        Span::styled(
                                            format!("  {}. ", i + 1),
                                            Style::default().fg(Color::Yellow),
                                        ),
                                        Span::styled(
                                            choice_text,
                                            Style::default().fg(Color::White),
                                        ),
                                    ]);
                                    items.push(ListItem::new(choice_line).style(item_style));
                                }
                                if choices.len() > 4 {
                                    let more_line = Line::from(vec![
                                        Span::raw("  "),
                                        Span::styled(
                                            format!("{}│  ", cont_prefix),
                                            Style::default().fg(Color::DarkGray),
                                        ),
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
                                let activity_text =
                                    truncate_str(activity, available_width.saturating_sub(14));
                                let activity_line = Line::from(vec![
                                    Span::raw("  "),
                                    Span::styled(
                                        format!("{}│  ", cont_prefix),
                                        Style::default().fg(Color::DarkGray),
                                    ),
                                    Span::styled(
                                        format!("{} ", state.spinner_frame()),
                                        Style::default().fg(Color::Yellow),
                                    ),
                                    Span::styled(activity_text, Style::default().fg(Color::Yellow)),
                                ]);
                                items.push(ListItem::new(activity_line).style(item_style));
                            }
                        }
                        AgentStatus::Error { message } => {
                            let error_text =
                                truncate_str(message, available_width.saturating_sub(14));
                            let error_line = Line::from(vec![
                                Span::raw("  "),
                                Span::styled(
                                    format!("{}│  ", cont_prefix),
                                    Style::default().fg(Color::DarkGray),
                                ),
                                Span::styled("✗ ", Style::default().fg(Color::Red)),
                                Span::styled(error_text, Style::default().fg(Color::Red)),
                            ]);
                            items.push(ListItem::new(error_line).style(item_style));
                        }
                        _ => {}
                    }

                    // Subagents
                    for (sub_idx, subagent) in agent.subagents.iter().enumerate() {
                        let is_last_sub = sub_idx == agent.subagents.len() - 1;
                        let sub_branch = if is_last_sub { "└─" } else { "├─" };

                        let (sub_char, sub_style) = match subagent.status {
                            SubagentStatus::Running => {
                                (state.spinner_frame(), Style::default().fg(Color::Cyan))
                            }
                            SubagentStatus::Completed => ("✓", Style::default().fg(Color::Green)),
                            SubagentStatus::Failed => ("✗", Style::default().fg(Color::Red)),
                            SubagentStatus::Unknown => ("?", Style::default().fg(Color::DarkGray)),
                        };

                        let duration = if matches!(subagent.status, SubagentStatus::Running) {
                            format!(" ({})", subagent.duration_str())
                        } else {
                            String::new()
                        };

                        let sub_line = Line::from(vec![
                            Span::raw("  "),
                            Span::styled(
                                format!("{}{}", cont_prefix, sub_branch),
                                Style::default().fg(Color::DarkGray),
                            ),
                            Span::styled(sub_char, sub_style),
                            Span::raw(" "),
                            Span::styled(
                                subagent.subagent_type.display_name(),
                                Style::default()
                                    .fg(Color::White)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(duration, Style::default().fg(Color::Yellow)),
                        ]);
                        items.push(ListItem::new(sub_line));

                        if !subagent.description.is_empty() {
                            let desc_prefix = if is_last_sub { "   " } else { "│  " };
                            let desc_text = truncate_str(
                                &subagent.description,
                                available_width.saturating_sub(14),
                            );
                            let desc_line = Line::from(vec![
                                Span::raw("  "),
                                Span::styled(
                                    format!("{}{}", cont_prefix, desc_prefix),
                                    Style::default().fg(Color::DarkGray),
                                ),
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

fn parse_color(name: &str) -> Color {
    let name = name.trim().to_lowercase();
    
    // Hex support (#RRGGBB)
    if name.starts_with('#') && name.len() == 7 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&name[1..3], 16),
            u8::from_str_radix(&name[3..5], 16),
            u8::from_str_radix(&name[5..7], 16),
        ) {
            return Color::Rgb(r, g, b);
        }
    }

    // RGB support (rgb(r,g,b))
    if name.starts_with("rgb(") && name.ends_with(')') {
        let parts: Vec<&str> = name[4..name.len()-1].split(',').map(|s| s.trim()).collect();
        if parts.len() == 3 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                parts[0].parse::<u8>(),
                parts[1].parse::<u8>(),
                parts[2].parse::<u8>(),
            ) {
                return Color::Rgb(r, g, b);
            }
        }
    }

    match name.as_str() {
        "magenta" => Color::Magenta,
        "blue" => Color::Blue,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "cyan" => Color::Cyan,
        "red" => Color::Red,
        "white" => Color::White,
        "black" => Color::Rgb(0, 0, 0),
        "gray" | "grey" => Color::Gray,
        "darkgray" | "darkgrey" => Color::DarkGray,
        "lightmagenta" => Color::LightMagenta,
        "lightblue" => Color::LightBlue,
        "lightgreen" => Color::LightGreen,
        "lightyellow" => Color::LightYellow,
        "lightcyan" => Color::LightCyan,
        "lightred" => Color::LightRed,
        _ => Color::Gray, // Safer fallback than bright cyan
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        format!(
            "{}..",
            s.chars()
                .take(max_len.saturating_sub(2))
                .collect::<String>()
        )
    }
}

fn context_bar(percent: u8) -> String {
    let total_blocks = 10;
    let filled = (percent as usize * total_blocks) / 100;
    let empty = total_blocks - filled;
    format!(
        "{}{}│{:>3}%",
        "█".repeat(filled),
        "░".repeat(empty),
        percent
    )
}
