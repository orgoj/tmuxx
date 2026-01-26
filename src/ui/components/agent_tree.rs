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
        // Get filtered agents
        let filtered_agents = state.filtered_agents();
        let _agents = &state.agents.root_agents; // Keep for counts
        let active_count = state.agents.active_count();
        let subagent_count = state.agents.running_subagent_count();
        let selected_count = state.selected_agents.len();

        // Build title
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

        let mode = state.config.pane_tree.mode.as_str();
        let template = if mode == "compact" {
             &state.config.pane_tree.compact_template
        } else {
             &state.config.pane_tree.full_template
        };
        let header_template = &state.config.pane_tree.header_template;

        for (session, windows) in tree.sessions.iter() {
            // Render Session Header (once per session)
            let header_str = if header_template.is_empty() {
                format!("▼ {}", session)
            } else {
                header_template.replace("{session}", session)
            };
            items.push(ListItem::new(Line::from(vec![
                Span::styled(header_str, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            ])));

            for ((window_num, window_name), window_agents) in windows.iter() {
                for (original_idx, agent) in window_agents.iter() {
                     let is_cursor = *original_idx == state.selected_index;
                     let is_selected = state.is_multi_selected(*original_idx);

                     // Render agent using template
                     let rendered_lines = render_agent(
                         template,
                         agent,
                         state,
                         is_cursor,
                         is_selected,
                         available_width,
                         session,
                         *window_num,
                         window_name
                     );

                     for (i, line) in rendered_lines.into_iter().enumerate() {
                         let style = if i == 0 {
                             // Apply selection background only to the first line (main line)
                             if is_cursor {
                                Style::default().bg(parse_color(&state.config.current_item_bg_color))
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
                             }
                         } else {
                             Style::default()
                         };
                         items.push(ListItem::new(line).style(style));
                     }
                }
            }
        }

        let list = List::new(items).block(block);
        let mut list_state = ListState::default();
        
        // We need to find the visual index of the selected agent.
        let mut visual_index = 0;
        let mut found = false;
        
        // Re-traverse to find visual index
        'outer: for (session, windows) in tree.sessions.iter() {
             visual_index += 1; // Header

             for ((window_num, window_name), window_agents) in windows.iter() {
                 for (original_idx, agent) in window_agents.iter() {
                     if *original_idx == state.selected_index {
                         found = true;
                         break 'outer;
                     }
                     // Count lines this agent takes
                     let lines = render_agent(template, agent, state, false, false, available_width, session, *window_num, window_name).len();
                     visual_index += lines; 
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
        let filtered_agents = state.filtered_agents();
        
        let tree = SessionWindowTree::new(&filtered_agents);
        
        let mode = state.config.pane_tree.mode.as_str();
        let template = if mode == "compact" {
             &state.config.pane_tree.compact_template
        } else {
             &state.config.pane_tree.full_template
        };

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
                     // Calculate height of this agent
                     let lines = render_agent(template, agent, state, false, false, width, session, *window_num, window_name).len();
                     
                     // Check if row falls within this agent's block
                     if row >= visual_index && row < visual_index + lines {
                         return Some(*original_idx);
                     }
                     
                     visual_index += lines;
                 }
             }
        }
        
        None
    }
}

/// Renders a single agent based on the template
fn render_agent<'a>(
    template: &str,
    agent: &'a MonitoredAgent,
    state: &'a AppState,
    is_cursor: bool,
    is_selected: bool,
    available_width: usize,
    session: &str,
    window_id: u32,
    window_name: &str,
) -> Vec<Line<'a>> {
    let mut lines = Vec::new();

    // Default template if empty
    let effective_template = if template.is_empty() {
        "{selection} {status_char} {name} {uptime}" 
    } else {
        template
    };

    // Split template by newlines
    for tmpl_line in effective_template.lines() {
        // Special case: {subagents} placeholder expands to multiple lines
        if tmpl_line.contains("{subagents}") {
             if tmpl_line.trim() == "{subagents}" {
                 // Standalone subagents placeholder - render proper subagent lines
                 lines.extend(render_subagents(agent, state, available_width));
                 continue;
             }
             // If mixed with other text, handle it (though usually it should be on its own line)
        }

        let rendered_line = render_template_line(tmpl_line, agent, state, is_cursor, is_selected, available_width, session, window_id, window_name);
        lines.push(rendered_line);
    }
    
    // Auto-append status details (approval/questions) if not explicitly handled
    // The previous implementation always showed them. We should probably keep showing them
    // unless we add specific placeholders for them.
    // For now, let's append them at the end if the status requires attention.
    if agent.status.needs_attention() {
         lines.extend(render_status_details(agent, available_width));
    }

    lines
}

fn render_template_line<'a>(
    tmpl_line: &str,
    agent: &'a MonitoredAgent,
    state: &'a AppState,
    is_cursor: bool,
    is_selected: bool,
    available_width: usize,
    session: &str,
    window_id: u32,
    window_name: &str,
) -> Line<'a> {
    let mut spans = Vec::new();
    let chars: Vec<char> = tmpl_line.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        if chars[i] == '{' {
            // Find closing '}'
            if let Some(end) = chars[i..].iter().position(|&c| c == '}') {
                let end_idx = i + end;
                let placeholder: String = chars[i+1..end_idx].iter().collect();
                
                // Render placeholder
                spans.push(render_placeholder(&placeholder, agent, state, is_cursor, is_selected, available_width, session, window_id, window_name));
                
                i = end_idx + 1;
                continue;
            }
        }
        
        // Regular char
        spans.push(Span::raw(chars[i].to_string()));
        i += 1;
    }
    
    Line::from(spans)
}

fn render_placeholder<'a>(
    name: &str,
    agent: &'a MonitoredAgent,
    state: &'a AppState,
    is_cursor: bool,
    is_selected: bool,
    _width: usize,
    session: &str,
    window_id: u32,
    window_name: &str,
) -> Span<'a> {
     match name {
         "session" => Span::styled(session.to_string(), Style::default().fg(Color::Cyan)),
         "window_id" => Span::styled(window_id.to_string(), Style::default().fg(Color::DarkGray)),
         "window_name" => Span::styled(window_name.to_string(), Style::default().fg(Color::White)),
         "selection" => {
             let text = if is_selected && is_cursor {
                "▶☑"
            } else if is_selected {
                " ☑"
            } else if is_cursor {
                "▶ "
            } else {
                "  "
            };
            if is_cursor {
                 Span::styled(text, Style::default().fg(Color::Rgb(0, 100, 200)).add_modifier(Modifier::BOLD))
            } else if is_selected {
                 Span::styled(text, Style::default().fg(Color::Cyan))
            } else {
                 Span::styled(text, Style::default().fg(Color::White))
            }
         },
         "status_char" => {
             match &agent.status {
                AgentStatus::Idle => Span::styled("●", Style::default().fg(Color::Green)),
                AgentStatus::Processing { .. } => Span::styled(state.spinner_frame(), Style::default().fg(Color::Yellow)),
                AgentStatus::AwaitingApproval { .. } => Span::styled("⚠", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                AgentStatus::Error { .. } => Span::styled("✗", Style::default().fg(Color::Red)),
                AgentStatus::Tui { .. } => Span::styled("○", Style::default().fg(Color::Blue)),
                AgentStatus::Unknown => Span::styled("○", Style::default().fg(Color::DarkGray)),
             }
         },
         "name" => {
             let color = agent.color.as_deref().unwrap_or(&state.config.agent_name_color);
             Span::styled(&agent.name, Style::default().fg(parse_color(color)))
         },
         "pid" => Span::styled(agent.pid.to_string(), Style::default().fg(Color::DarkGray)),
         "uptime" => Span::styled(agent.uptime_str(), Style::default().fg(Color::DarkGray)),
         "path" => Span::styled(agent.abbreviated_path(), Style::default().fg(Color::Cyan)),
         "status_text" => {
             let (text, style) = match &agent.status {
                 AgentStatus::Idle => ("Idle", Style::default().fg(Color::Green)),
                 AgentStatus::Processing { .. } => ("Working", Style::default().fg(Color::Yellow)),
                 AgentStatus::AwaitingApproval { .. } => ("Waiting", Style::default().fg(Color::Red)),
                 AgentStatus::Error { .. } => ("Error", Style::default().fg(Color::Red)),
                 AgentStatus::Tui { name } => (name.as_str(), Style::default().fg(Color::Blue)),
                 AgentStatus::Unknown => ("Unknown", Style::default().fg(Color::DarkGray)),
             };
             Span::styled(text, style)
         },
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
         },
         "subagents" => Span::raw(""), // Handled separately
         _ => Span::raw(format!("{{{}}}", name)),
     }
}

fn render_subagents<'a>(agent: &'a MonitoredAgent, state: &'a AppState, width: usize) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    for subagent in &agent.subagents {
        let (sub_char, sub_style) = match subagent.status {
            SubagentStatus::Running => (state.spinner_frame(), Style::default().fg(Color::Cyan)),
            SubagentStatus::Completed => ("✓", Style::default().fg(Color::Green)),
            SubagentStatus::Failed => ("✗", Style::default().fg(Color::Red)),
            SubagentStatus::Unknown => ("?", Style::default().fg(Color::DarkGray)),
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
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
            Span::styled(duration, Style::default().fg(Color::Yellow)),
        ]);
        lines.push(line);
        
        // Description
        if !subagent.description.is_empty() {
            lines.push(Line::from(vec![
                Span::raw("     "), // Indent
                Span::styled(truncate_str(&subagent.description, width.saturating_sub(10)), Style::default().fg(Color::DarkGray))
            ]));
        }
    }
    lines
}

fn render_status_details<'a>(agent: &'a MonitoredAgent, width: usize) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    match &agent.status {
        AgentStatus::AwaitingApproval { approval_type, details } => {
             lines.push(Line::from(vec![
                 Span::raw("   "),
                 Span::styled("⚠ ", Style::default().fg(Color::Red)),
                 Span::styled(format!("{}", approval_type), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
             ]));
             
             if !details.is_empty() {
                 lines.push(Line::from(vec![
                     Span::raw("      "),
                     Span::styled(truncate_str(details, width.saturating_sub(10)), Style::default().fg(Color::White))
                 ]));
             }
             
             if let ApprovalType::UserQuestion { choices, .. } = approval_type {
                for (i, choice) in choices.iter().take(4).enumerate() {
                    lines.push(Line::from(vec![
                        Span::raw("      "),
                        Span::styled(format!("{}. ", i + 1), Style::default().fg(Color::Yellow)),
                        Span::styled(truncate_str(choice, width.saturating_sub(14)), Style::default().fg(Color::White))
                    ]));
                }
                if choices.len() > 4 {
                    lines.push(Line::from(vec![
                         Span::raw("      "),
                         Span::styled(format!("...+{} more", choices.len() - 4), Style::default().fg(Color::DarkGray))
                    ]));
                }
             }
        },
        AgentStatus::Processing { activity } if !activity.is_empty() => {
             lines.push(Line::from(vec![
                 Span::raw("   "),
                 Span::styled(activity, Style::default().fg(Color::Yellow))
             ]));
        },
        AgentStatus::Error { message } => {
             lines.push(Line::from(vec![
                 Span::raw("   "),
                 Span::styled("Error: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                 Span::styled(message, Style::default().fg(Color::Red))
             ]));
        },
        _ => {}
    }
    lines
    
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
        let parts: Vec<&str> = name[4..name.len() - 1]
            .split(',')
            .map(|s| s.trim())
            .collect();
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


