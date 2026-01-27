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
use crate::ui::Styles;

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
    fn new(agents: &[(usize, &'a MonitoredAgent)]) -> Self {
        let mut sessions: SessionsMap<'a> = BTreeMap::new();

        for (original_idx, agent) in agents.iter() {
            sessions
                .entry(&agent.session)
                .or_default()
                .entry((agent.window, &agent.window_name))
                .or_default()
                .push((*original_idx, *agent));
        }

        Self { sessions }
    }
}

/// Context for rendering an agent line
struct AgentRenderCtx<'a, 'b> {
    state: &'a AppState,
    session: &'a str,
    window_id: u32,
    window_name: &'a str,
    available_width: usize,
    is_cursor: bool,
    is_selected: bool,
    color_cache: &'b mut HashMap<String, Color>,
}

use std::collections::HashMap;

impl AgentTreeWidget {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        // Get filtered agents with original indices
        let filtered_agents = state.filtered_agents_with_indices();
        let _agents = &state.agents.root_agents; // Keep for counts
        let active_count = state.agents.active_count();
        let subagent_count = state.agents.running_subagent_count();
        let selected_count = state.selected_agents.len();

        // Build title
        let title = if selected_count > 0 {
            format!(
                " {} {} │ {} {} ",
                selected_count,
                state.config.messages.label_sel,
                active_count,
                state.config.messages.label_pending
            )
        } else if subagent_count > 0 {
            format!(
                " {} {} │ {} {} ",
                active_count,
                state.config.messages.label_pending,
                subagent_count,
                state.config.messages.label_subs
            )
        } else if active_count > 0 {
            format!(
                " ⚠ {} {} ",
                active_count, state.config.messages.label_pending
            )
        } else {
            format!(
                " {} {} ",
                filtered_agents.len(),
                state.config.messages.label_agents
            )
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

        let mode = state.config.pane_tree.mode.as_str();
        let template = if mode == "compact" {
            &state.config.pane_tree.compact_template
        } else {
            &state.config.pane_tree.full_template
        };
        // Pre-parse template to avoid parsing in loop
        let parsed_template = parse_template(template);

        let header_template = &state.config.pane_tree.header_template;

        let selected_bg = Styles::parse_color(&state.config.current_item_bg_color);
        let multi_select_bg = state
            .config
            .multi_selection_bg_color
            .as_ref()
            .map(|c| Styles::parse_color(c));
        let header_fg = Styles::parse_color(&state.config.pane_tree.session_header_fg_color);
        let header_bg = state
            .config
            .pane_tree
            .session_header_bg_color
            .as_ref()
            .map(|c| Styles::parse_color(c));

        let mut color_cache: HashMap<String, Color> = HashMap::new();

        for (session, windows) in tree.sessions.iter() {
            // Render Session Header (once per session)
            let header_str = if header_template.is_empty() {
                format!("▼ {}", session)
            } else {
                header_template.replace("{session}", session)
            };

            // Text style (FG only)
            let text_style = Style::default().fg(header_fg).add_modifier(Modifier::BOLD);

            // Item style (BG applies to full width)
            let mut item_style = Style::default();
            if let Some(bg) = header_bg {
                item_style = item_style.bg(bg);
            }

            items.push(
                ListItem::new(Line::from(vec![Span::styled(header_str, text_style)]))
                    .style(item_style),
            );

            for ((window_num, window_name), window_agents) in windows.iter() {
                for (original_idx, agent) in window_agents.iter() {
                    let is_cursor = *original_idx == state.selected_index;
                    let is_selected = state.is_multi_selected(*original_idx);

                    let mut ctx = AgentRenderCtx {
                        state,
                        session,
                        window_id: *window_num,
                        window_name,
                        available_width,
                        is_cursor,
                        is_selected,
                        color_cache: &mut color_cache,
                    };

                    // Render agent using pre-parsed template
                    let rendered_lines = render_parsed_template(&parsed_template, agent, &mut ctx);

                    // Create ONE ListItem for the whole agent (fixes cropping)
                    let mut item = ListItem::new(rendered_lines);

                    // Apply style to the whole item
                    if is_cursor {
                        item = item.style(Style::default().bg(selected_bg));
                    } else if is_selected {
                        if let Some(bg) = multi_select_bg {
                            item = item.style(Style::default().bg(bg));
                        } else if let Some(bg_color) = &agent.background_color {
                            item = item.style(Style::default().bg(Styles::parse_color(bg_color)));
                        }
                    } else if let Some(bg_color) = &agent.background_color {
                        item = item.style(Style::default().bg(Styles::parse_color(bg_color)));
                    }

                    items.push(item);
                }
            }
        }

        let list = List::new(items).block(block);
        let mut list_state = ListState::default();

        // We need to find the visual index of the selected agent.
        let mut visual_index = 0;
        let mut found = false;

        // Re-traverse to find visual index
        'outer: for (_session, windows) in tree.sessions.iter() {
            visual_index += 1; // Header

            for ((_window_num, _window_name), window_agents) in windows.iter() {
                for (original_idx, _agent) in window_agents.iter() {
                    if *original_idx == state.selected_index {
                        found = true;
                        break 'outer;
                    }

                    visual_index += 1; // Each agent is now 1 item
                }
            }
        }

        if found {
            list_state.select(Some(visual_index));
        } else {
            list_state.select(None);
        }

        frame.render_stateful_widget(list, area, &mut list_state);
    }

    /// Maps a visual row index (relative to list content) to an agent index
    pub fn get_agent_index_at_row(row: usize, state: &AppState, width: usize) -> Option<usize> {
        let filtered_agents = state.filtered_agents_with_indices();

        let tree = SessionWindowTree::new(&filtered_agents);

        let mode = state.config.pane_tree.mode.as_str();
        let template = if mode == "compact" {
            &state.config.pane_tree.compact_template
        } else {
            &state.config.pane_tree.full_template
        };
        let parsed_template = parse_template(template);
        let mut color_cache = HashMap::new();

        // Re-traverse to find agent at row
        let mut visual_index = 0;

        for (session, windows) in tree.sessions.iter() {
            // Header takes 1 line
            if visual_index == row {
                // Clicked on session header - maybe in future this can collapse select session etc.
                return None;
            }
            visual_index += 1;

            for ((window_num, window_name), window_agents) in windows.iter() {
                for (original_idx, agent) in window_agents.iter() {
                    let mut ctx = AgentRenderCtx {
                        state,
                        session,
                        window_id: *window_num,
                        window_name,
                        available_width: width,
                        is_cursor: false,
                        is_selected: false,
                        color_cache: &mut color_cache,
                    };

                    // Calculate height by rendering (fast enough for click handling)
                    let height = render_parsed_template(&parsed_template, agent, &mut ctx).len();

                    // Check if row matches this agent item block
                    if row >= visual_index && row < visual_index + height {
                        return Some(*original_idx);
                    }

                    visual_index += height;
                }
            }
        }

        None
    }
}

enum TemplatePart {
    Text(String),
    Placeholder(String),
}

fn parse_template(template: &str) -> Vec<Vec<TemplatePart>> {
    template
        .lines()
        .map(|line| {
            let mut parts = Vec::new();
            let mut last_end = 0;
            while let Some(start) = line[last_end..].find('{') {
                let abs_start = last_end + start;
                if abs_start > last_end {
                    parts.push(TemplatePart::Text(line[last_end..abs_start].to_string()));
                }
                if let Some(end) = line[abs_start..].find('}') {
                    let abs_end = abs_start + end;
                    parts.push(TemplatePart::Placeholder(
                        line[abs_start + 1..abs_end].to_string(),
                    ));
                    last_end = abs_end + 1;
                } else {
                    parts.push(TemplatePart::Text("{".to_string()));
                    last_end = abs_start + 1;
                }
            }
            if last_end < line.len() {
                parts.push(TemplatePart::Text(line[last_end..].to_string()));
            }
            parts
        })
        .collect()
}

fn render_parsed_template<'a, 'b>(
    parsed_lines: &[Vec<TemplatePart>],
    agent: &'a MonitoredAgent,
    ctx: &mut AgentRenderCtx<'a, 'b>,
) -> Vec<Line<'a>> {
    let mut lines = Vec::with_capacity(parsed_lines.len());

    for line_parts in parsed_lines {
        // Special case: {subagents} placeholder expands to multiple lines
        if line_parts.len() == 1 {
            if let TemplatePart::Placeholder(p) = &line_parts[0] {
                if p == "subagents" {
                    lines.extend(render_subagents(agent, ctx.state, ctx.available_width));
                    continue;
                }
            }
        }

        let mut spans = Vec::with_capacity(line_parts.len());
        for part in line_parts {
            match part {
                TemplatePart::Text(t) => spans.push(Span::raw(t.clone())),
                TemplatePart::Placeholder(name) => {
                    spans.push(render_placeholder(name, agent, ctx));
                    if name == "name" {
                        for icon in &agent.active_indicators {
                            spans.push(Span::raw(format!(" {}", icon)));
                        }
                    }
                }
            }
        }
        lines.push(Line::from(spans));
    }

    // Auto-append status details (approval/questions) if not explicitly handled
    if agent.status.needs_attention() {
        lines.extend(render_status_details(agent, ctx.available_width));
    }

    lines
}

fn render_placeholder<'a, 'b>(
    name: &str,
    agent: &'a MonitoredAgent,
    ctx: &mut AgentRenderCtx<'a, 'b>,
) -> Span<'a> {
    match name {
        "session" => Span::styled(ctx.session.to_string(), Style::default().fg(Color::Cyan)),
        "window_id" => Span::styled(
            ctx.window_id.to_string(),
            Style::default().fg(Color::DarkGray),
        ),
        "window_name" => Span::styled(
            ctx.window_name.to_string(),
            Style::default().fg(Color::White),
        ),
        "selection" => {
            let text = if ctx.is_selected && ctx.is_cursor {
                "▶☑"
            } else if ctx.is_selected {
                " ☑"
            } else if ctx.is_cursor {
                "▶ "
            } else {
                "  "
            };
            if ctx.is_cursor {
                Span::styled(
                    text,
                    Style::default()
                        .fg(Color::Rgb(0, 100, 200))
                        .add_modifier(Modifier::BOLD),
                )
            } else if ctx.is_selected {
                Span::styled(text, Style::default().fg(Color::Cyan))
            } else {
                Span::styled(text, Style::default().fg(Color::White))
            }
        }
        "status_char" => match &agent.status {
            AgentStatus::Idle { .. } => Span::styled(
                &ctx.state.config.indicators.idle,
                Style::default().fg(Color::Green),
            ),
            AgentStatus::Processing { .. } => Span::styled(
                ctx.state.spinner_frame(),
                Style::default().fg(Color::Yellow),
            ),
            AgentStatus::AwaitingApproval { .. } => Span::styled(
                &ctx.state.config.indicators.approval,
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            AgentStatus::Error { .. } => Span::styled(
                &ctx.state.config.indicators.error,
                Style::default().fg(Color::Red),
            ),
            AgentStatus::Unknown => Span::styled(
                &ctx.state.config.indicators.unknown,
                Style::default().fg(Color::DarkGray),
            ),
        },
        "name" => {
            let color_name = agent
                .color
                .as_deref()
                .unwrap_or(&ctx.state.config.agent_name_color);

            let color = if let Some(c) = ctx.color_cache.get(color_name) {
                *c
            } else {
                let c = Styles::parse_color(color_name);
                ctx.color_cache.insert(color_name.to_string(), c);
                c
            };

            Span::styled(&agent.name, Style::default().fg(color))
        }
        "pid" => Span::styled(agent.pid.to_string(), Style::default().fg(Color::DarkGray)),
        "uptime" => Span::styled(agent.uptime_str(), Style::default().fg(Color::DarkGray)),
        "path" => Span::styled(agent.abbreviated_path(), Style::default().fg(Color::Cyan)),
        "status_text" => {
            let (text, style) = match &agent.status {
                AgentStatus::Idle { label } => (
                    label.as_deref().unwrap_or("Idle"),
                    Style::default().fg(Color::Green),
                ),
                AgentStatus::Processing { .. } => ("Working", Style::default().fg(Color::Yellow)),
                AgentStatus::AwaitingApproval { .. } => {
                    ("Waiting", Style::default().fg(Color::Red))
                }
                AgentStatus::Error { .. } => ("Error", Style::default().fg(Color::Red)),
                AgentStatus::Unknown => ("Unknown", Style::default().fg(Color::DarkGray)),
            };
            Span::styled(text, style)
        }
        "context" => {
            if let Some(ctx) = agent.context_remaining {
                let bar_color = if ctx > 50 {
                    Color::Green
                } else if ctx > 20 {
                    Color::Yellow
                } else {
                    Color::Red
                };
                Span::styled(context_bar(ctx), Style::default().fg(bar_color))
            } else {
                Span::raw("")
            }
        }
        "subagents" => Span::raw(""), // Handled separately
        _ => Span::raw(format!("{{{}}}", name)),
    }
}

fn render_subagents<'a>(
    agent: &'a MonitoredAgent,
    state: &'a AppState,
    width: usize,
) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    for subagent in &agent.subagents {
        let (sub_char, sub_style) = match subagent.status {
            SubagentStatus::Running => (state.spinner_frame(), Style::default().fg(Color::Cyan)),
            SubagentStatus::Completed => (
                state.config.indicators.subagent_completed.as_str(),
                Style::default().fg(Color::Green),
            ),
            SubagentStatus::Failed => (
                state.config.indicators.subagent_failed.as_str(),
                Style::default().fg(Color::Red),
            ),
            SubagentStatus::Unknown => (
                state.config.indicators.unknown.as_str(),
                Style::default().fg(Color::DarkGray),
            ),
        };

        let duration = if matches!(subagent.status, SubagentStatus::Running) {
            format!(" ({})", subagent.duration_str())
        } else {
            String::new()
        };

        let line = Line::from(vec![
            Span::raw("   "), // Indent
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
        lines.push(line);

        // Description
        if !subagent.description.is_empty() {
            lines.push(Line::from(vec![
                Span::raw("     "), // Indent
                Span::styled(
                    truncate_str(&subagent.description, width.saturating_sub(10)),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
    }
    lines
}

fn render_status_details<'a>(agent: &'a MonitoredAgent, width: usize) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    match &agent.status {
        AgentStatus::AwaitingApproval {
            approval_type,
            details,
        } => {
            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled("⚠ ", Style::default().fg(Color::Red)),
                Span::styled(
                    format!("{}", approval_type),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            ]));

            if !details.is_empty() {
                lines.push(Line::from(vec![
                    Span::raw("      "),
                    Span::styled(
                        truncate_str(details, width.saturating_sub(10)),
                        Style::default().fg(Color::White),
                    ),
                ]));
            }

            if let ApprovalType::UserQuestion { choices, .. } = approval_type {
                for (i, choice) in choices.iter().take(4).enumerate() {
                    lines.push(Line::from(vec![
                        Span::raw("      "),
                        Span::styled(format!("{}. ", i + 1), Style::default().fg(Color::Yellow)),
                        Span::styled(
                            truncate_str(choice, width.saturating_sub(14)),
                            Style::default().fg(Color::White),
                        ),
                    ]));
                }
                if choices.len() > 4 {
                    lines.push(Line::from(vec![
                        Span::raw("      "),
                        Span::styled(
                            format!("...+{} more", choices.len() - 4),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]));
                }
            }
        }
        AgentStatus::Processing { activity } if !activity.is_empty() => {
            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled(activity, Style::default().fg(Color::Yellow)),
            ]));
        }
        AgentStatus::Error { message } => {
            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled(
                    "Error: ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(message, Style::default().fg(Color::Red)),
            ]));
        }
        _ => {}
    }
    lines
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
