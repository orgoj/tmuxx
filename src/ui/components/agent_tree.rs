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
            state.styles.border_focused.fg.unwrap_or(Color::Cyan)
        } else {
            state.styles.border.fg.unwrap_or(Color::Gray)
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

        let header_fg = state.styles.header.fg.unwrap_or(Color::Cyan);
        let header_bg = state.styles.header.bg;

        let selected_bg = state.styles.selected.bg;
        let multi_select_bg = state
            .config
            .multi_selection_bg_color
            .as_ref()
            .and_then(|c| {
                if c.to_lowercase() == "none" {
                    None
                } else {
                    Styles::parse_color(c)
                }
            });

        let mut color_cache: HashMap<String, Color> = HashMap::new();
        let _selection_mode = state.config.selection_mode.as_str();

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
                    let mut item_style = Style::default();

                    // 1. Base background from agent config
                    if let Some(bg_color) = &agent.background_color {
                        if let Some(c) = Styles::parse_color(bg_color) {
                            item_style = item_style.bg(c);
                        }
                    }

                    // 2. Apply selection background if configured
                    if is_cursor {
                        if let Some(bg_color) = selected_bg {
                            item_style = item_style.bg(bg_color);
                        }
                    } else if is_selected {
                        if let Some(bg_color) = multi_select_bg {
                            item_style = item_style.bg(bg_color);
                        }
                    }

                    item = item.style(item_style);
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
                    // Calculate height by rendering (fast enough for click handling)
                    let mut ctx = AgentRenderCtx {
                        state,
                        session,
                        window_id: *window_num,
                        window_name,
                        available_width: width,
                        is_cursor: false, // height same regardless
                        is_selected: false,
                        color_cache: &mut color_cache,
                    };
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
            // In bar mode, we want the bar at the very beginning of EVERY line
            // We'll handle this during rendering.

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
    let selection_mode = ctx.state.config.selection_mode.as_str();
    let selection_char = &ctx.state.config.selection_char;
    let selection_repeat = ctx.state.config.selection_char_repeat;
    let bar_width = ctx.state.config.selection_bar_width as usize;

    // Determine bar colors
    let bar_fg = ctx
        .state
        .config
        .selection_bar_fg_color
        .as_ref()
        .and_then(|c| Styles::parse_color(c))
        .unwrap_or_else(|| ctx.state.styles.selected.bg.unwrap_or(Color::Yellow));

    let bar_bg = ctx
        .state
        .config
        .selection_bar_bg_color
        .as_ref()
        .and_then(|c| Styles::parse_color(c));

    let mut bar_style = Style::default().fg(bar_fg);
    if let Some(bg) = bar_bg {
        bar_style = bar_style.bg(bg);
    }

    for (line_idx, line_parts) in parsed_lines.iter().enumerate() {
        // Special case: {subagents} placeholder expands to multiple lines
        if line_parts.len() == 1 {
            if let TemplatePart::Placeholder(p) = &line_parts[0] {
                if p == "subagents" {
                    let mut sub_lines = render_subagents(agent, ctx.state, ctx.available_width);
                    // Apply selection bar to subagent lines too
                    if ctx.is_cursor && selection_mode == "bar" && selection_repeat {
                        for line in &mut sub_lines {
                            // Prepend bar
                            let mut new_spans =
                                vec![Span::styled(selection_char.repeat(bar_width), bar_style)];

                            // Trim leading spaces from the first span of the sub-line
                            if let Some(first_span) = line.spans.first_mut() {
                                let mut text = first_span.content.to_string();
                                let mut removed = 0;
                                while removed < bar_width && text.starts_with(' ') {
                                    text.remove(0);
                                    removed += 1;
                                }
                                *first_span = Span::styled(text, first_span.style);
                            }

                            new_spans.extend(line.spans.clone());
                            line.spans = new_spans;
                        }
                    } else if ctx.is_cursor && selection_mode == "bar" && !selection_repeat {
                        // Just add padding if not repeating
                        for line in &mut sub_lines {
                            let mut new_spans = vec![Span::raw(" ".repeat(bar_width))];
                            new_spans.extend(line.spans.clone());
                            line.spans = new_spans;
                        }
                    }
                    lines.extend(sub_lines);
                    continue;
                }
            }
        }

        let mut spans = Vec::with_capacity(line_parts.len() + 1);

        // Add vertical selection bar at the start if in bar mode
        if ctx.is_cursor && selection_mode == "bar" {
            if line_idx == 0 || selection_repeat {
                spans.push(Span::styled(selection_char.repeat(bar_width), bar_style));
            } else {
                // If not repeating, we still need to add padding to keep alignment
                spans.push(Span::raw(" ".repeat(bar_width)));
            }
        }

        for (i, part) in line_parts.iter().enumerate() {
            match part {
                TemplatePart::Text(t) => {
                    let mut text = t.clone();
                    // If we added a bar, we might want to trim leading spaces from the first text part
                    // to keep the layout consistent with non-bar mode or non-selected state.
                    if i == 0 && ctx.is_cursor && selection_mode == "bar" {
                        let spaces_to_remove = bar_width;
                        let mut removed = 0;
                        while removed < spaces_to_remove && text.starts_with(' ') {
                            text.remove(0);
                            removed += 1;
                        }
                    }
                    spans.push(Span::raw(text));
                }
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
        let mut detail_lines = render_status_details(agent, ctx);
        // Apply selection bar to detail lines too
        if ctx.is_cursor && selection_mode == "bar" {
            for line in &mut detail_lines {
                // Prepend bar or spaces
                let span = if selection_repeat {
                    Span::styled(selection_char.repeat(bar_width), bar_style)
                } else {
                    Span::raw(" ".repeat(bar_width))
                };
                let mut new_spans = vec![span];

                // Trim leading spaces from the first span
                if let Some(first_span) = line.spans.first_mut() {
                    let mut text = first_span.content.to_string();
                    let mut removed = 0;
                    while removed < bar_width && text.starts_with(' ') {
                        text.remove(0);
                        removed += 1;
                    }
                    *first_span = Span::styled(text, first_span.style);
                }

                new_spans.extend(line.spans.clone());
                line.spans = new_spans;
            }
        }
        lines.extend(detail_lines);
    }

    lines
}

fn render_placeholder<'a, 'b>(
    name: &str,
    agent: &'a MonitoredAgent,
    ctx: &mut AgentRenderCtx<'a, 'b>,
) -> Span<'a> {
    match name {
        "session" => Span::styled(ctx.session.to_string(), ctx.state.styles.header),
        "window_id" => Span::styled(ctx.window_id.to_string(), ctx.state.styles.dimmed),
        "window_name" => Span::styled(ctx.window_name.to_string(), ctx.state.styles.normal),
        "selection" => {
            let selection_mode = ctx.state.config.selection_mode.as_str();
            let selection_char = &ctx.state.config.selection_char;

            let text = if ctx.is_selected && ctx.is_cursor {
                format!("{}☑", selection_char)
            } else if ctx.is_selected {
                " ☑".to_string()
            } else if ctx.is_cursor {
                format!("{} ", selection_char)
            } else {
                "  ".to_string()
            };

            // In bar mode, we don't need the selection placeholder to render the bar
            // (it's now rendered for every line), so we just show status icons/padding
            if selection_mode == "bar" {
                let bar_text = if ctx.is_selected { "☑" } else { " " };
                return Span::styled(bar_text, ctx.state.styles.header);
            }

            if ctx.is_cursor {
                Span::styled(text, ctx.state.styles.highlight)
            } else if ctx.is_selected {
                Span::styled(text, ctx.state.styles.header)
            } else {
                Span::styled(text, ctx.state.styles.normal)
            }
        }
        "status_char" => match &agent.status {
            AgentStatus::Idle { .. } => {
                Span::styled(&ctx.state.config.indicators.idle, ctx.state.styles.idle)
            }
            AgentStatus::Processing { .. } => {
                Span::styled(ctx.state.spinner_frame(), ctx.state.styles.processing)
            }
            AgentStatus::AwaitingApproval { .. } => Span::styled(
                &ctx.state.config.indicators.approval,
                ctx.state.styles.awaiting_approval,
            ),
            AgentStatus::Error { .. } => {
                Span::styled(&ctx.state.config.indicators.error, ctx.state.styles.error)
            }
            AgentStatus::Unknown => Span::styled(
                &ctx.state.config.indicators.unknown,
                ctx.state.styles.unknown,
            ),
        },
        "name" => {
            let color_name = agent
                .color
                .as_deref()
                .unwrap_or(&ctx.state.config.agent_name_color);

            let color = if let Some(c) = ctx.color_cache.get(color_name) {
                Some(*c)
            } else {
                let c = Styles::parse_color(color_name);
                if let Some(c_val) = c {
                    ctx.color_cache.insert(color_name.to_string(), c_val);
                }
                c
            };

            if let Some(c) = color {
                Span::styled(&agent.name, Style::default().fg(c))
            } else {
                Span::raw(&agent.name)
            }
        }
        "pid" => Span::styled(agent.pid.to_string(), ctx.state.styles.dimmed),
        "uptime" => Span::styled(agent.uptime_str(), ctx.state.styles.dimmed),
        "path" => Span::styled(agent.abbreviated_path(), ctx.state.styles.header),
        "status_text" => {
            let (text, style) = match &agent.status {
                AgentStatus::Idle { label } => {
                    (label.as_deref().unwrap_or("Idle"), ctx.state.styles.idle)
                }
                AgentStatus::Processing { .. } => ("Working", ctx.state.styles.processing),
                AgentStatus::AwaitingApproval { .. } => {
                    ("Waiting", ctx.state.styles.awaiting_approval)
                }
                AgentStatus::Error { .. } => ("Error", ctx.state.styles.error),
                AgentStatus::Unknown => ("Unknown", ctx.state.styles.unknown),
            };
            Span::styled(text, style)
        }
        "context" => {
            if let Some(percent) = agent.context_remaining {
                let bar_color = if percent > 50 {
                    ctx.state.styles.idle.fg.unwrap_or(Color::Green)
                } else if percent > 20 {
                    ctx.state.styles.processing.fg.unwrap_or(Color::Yellow)
                } else {
                    ctx.state.styles.error.fg.unwrap_or(Color::Red)
                };
                Span::styled(context_bar(percent), Style::default().fg(bar_color))
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
            SubagentStatus::Running => (state.spinner_frame(), state.styles.subagent_running),
            SubagentStatus::Completed => (
                state.config.indicators.subagent_completed.as_str(),
                state.styles.subagent_completed,
            ),
            SubagentStatus::Failed => (
                state.config.indicators.subagent_failed.as_str(),
                state.styles.subagent_failed,
            ),
            SubagentStatus::Unknown => (
                state.config.indicators.unknown.as_str(),
                state.styles.unknown,
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
                state.styles.normal.add_modifier(Modifier::BOLD),
            ),
            Span::styled(duration, state.styles.highlight),
        ]);
        lines.push(line);

        // Description
        if !subagent.description.is_empty() {
            lines.push(Line::from(vec![
                Span::raw("     "), // Indent
                Span::styled(
                    truncate_str(&subagent.description, width.saturating_sub(10)),
                    state.styles.dimmed,
                ),
            ]));
        }
    }
    lines
}

fn render_status_details<'a>(
    agent: &'a MonitoredAgent,
    ctx: &mut AgentRenderCtx<'a, '_>,
) -> Vec<Line<'a>> {
    let width = ctx.available_width;
    let mut lines = Vec::new();
    match &agent.status {
        AgentStatus::AwaitingApproval {
            approval_type,
            details,
        } => {
            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled("⚠ ", ctx.state.styles.awaiting_approval),
                Span::styled(
                    format!("{}", approval_type),
                    ctx.state.styles.awaiting_approval,
                ),
            ]));

            if !details.is_empty() {
                lines.push(Line::from(vec![
                    Span::raw("      "),
                    Span::styled(
                        truncate_str(details, width.saturating_sub(10)),
                        ctx.state.styles.normal,
                    ),
                ]));
            }

            if let ApprovalType::UserQuestion { choices, .. } = approval_type {
                for (i, choice) in choices.iter().take(4).enumerate() {
                    lines.push(Line::from(vec![
                        Span::raw("      "),
                        Span::styled(format!("{}. ", i + 1), ctx.state.styles.highlight),
                        Span::styled(
                            truncate_str(choice, width.saturating_sub(14)),
                            ctx.state.styles.normal,
                        ),
                    ]));
                }
                if choices.len() > 4 {
                    lines.push(Line::from(vec![
                        Span::raw("      "),
                        Span::styled(
                            format!("...+{} more", choices.len() - 4),
                            ctx.state.styles.dimmed,
                        ),
                    ]));
                }
            }
        }
        AgentStatus::Processing { activity } if !activity.is_empty() => {
            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled(activity, ctx.state.styles.processing),
            ]));
        }
        AgentStatus::Error { message } => {
            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled(
                    "Error: ",
                    ctx.state.styles.error.add_modifier(Modifier::BOLD),
                ),
                Span::styled(message, ctx.state.styles.error),
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
