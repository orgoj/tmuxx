use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::agents::AgentStatus;
use crate::app::AppState;
use crate::parsers::ParserRegistry;

/// Truncate a line to fit within max_width
/// Returns (truncated_string, was_truncated)
fn truncate_line(line: &str, max_width: usize) -> (String, bool) {
    let width = line.width();
    if width <= max_width {
        return (line.to_string(), false);
    }

    // Truncate to max_width - 1 for ellipsis
    let target = max_width.saturating_sub(1);
    let mut current_width = 0;
    let truncated: String = line
        .chars()
        .take_while(|c| {
            let char_width = c.width().unwrap_or(1);
            // Check BEFORE adding to avoid off-by-one with wide chars
            if current_width + char_width <= target {
                current_width += char_width;
                true
            } else {
                false
            }
        })
        .collect();

    (format!("{}…", truncated), true)
}

/// Widget for previewing the selected pane content
pub struct PanePreviewWidget;

impl PanePreviewWidget {
    /// Render a summary view with TODO (left) and activity (right) in 2-column layout
    pub fn render_summary(frame: &mut Frame, area: Rect, state: &AppState) {
        let agent = state.selected_visible_agent();

        if let Some(agent) = agent {
            // Use config-driven parser summary
            let registry = ParserRegistry::with_config(&state.config);
            let summary = if let Some(parser) = registry
                .all_parsers()
                .find(|p| p.agent_name() == agent.name)
            {
                parser.parse_summary(&agent.last_content)
            } else {
                crate::parsers::AgentSummary::default()
            };

            // Outer block for the entire summary area
            let outer_block = Block::default()
                .title(format!(" {} ", agent.name))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Gray));

            let inner_area = outer_block.inner(area);
            frame.render_widget(outer_block, area);

            // Determine if we should show TODO in full width
            let has_todo = if state.config.todo_from_file {
                state.current_todo.is_some()
            } else {
                !summary.tasks.is_empty()
            };

            let use_full_width = state.config.todo_full_width && has_todo;

            // Split into columns: TODO | Activity (if not full width)
            let columns = if use_full_width {
                vec![inner_area]
            } else {
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(inner_area)
                    .to_vec()
            };

            // Left column (or full): TODOs (from file or from agent parsing)
            let mut todo_lines: Vec<Line> = Vec::new();
            if state.config.todo_from_file {
                if let Some(todo) = &state.current_todo {
                    todo_lines.push(Line::from(vec![Span::styled(
                        &state.config.messages.label_todo,
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::BOLD),
                    )]));
                    for line in todo.lines() {
                        todo_lines.push(Line::from(vec![Span::styled(
                            line,
                            Style::default().fg(Color::White),
                        )]));
                    }
                } else {
                    todo_lines.push(Line::from(vec![Span::styled(
                        "No Project TODO",
                        Style::default().fg(Color::DarkGray),
                    )]));
                }
            } else if !summary.tasks.is_empty() {
                todo_lines.push(Line::from(vec![Span::styled(
                    &state.config.messages.label_tasks,
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::BOLD),
                )]));
                for (completed, text) in &summary.tasks {
                    let (icon, style) = if *completed {
                        (
                            state.config.indicators.subagent_completed.as_str(),
                            Style::default().fg(Color::DarkGray),
                        )
                    } else {
                        ("☐ ", Style::default().fg(Color::White))
                    };
                    todo_lines.push(Line::from(vec![
                        Span::styled(format!(" {}", icon), style),
                        Span::styled(text.clone(), style),
                    ]));
                }
            } else {
                todo_lines.push(Line::from(vec![Span::styled(
                    "No Tasks",
                    Style::default().fg(Color::DarkGray),
                )]));
            }

            let todo_paragraph =
                Paragraph::new(todo_lines).wrap(ratatui::widgets::Wrap { trim: false });
            frame.render_widget(todo_paragraph, columns[0]);

            if !use_full_width {
                // Right column: Activity and tools
                let mut activity_lines: Vec<Line> = Vec::new();

                // Current activity
                if let Some(activity) = &summary.current_activity {
                    activity_lines.push(Line::from(vec![
                        Span::styled("▶ ", Style::default().fg(Color::Yellow)),
                        Span::styled(
                            activity.clone(),
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]));
                    activity_lines.push(Line::from(""));
                }

                // Running tools
                if !summary.tools.is_empty() {
                    activity_lines.push(Line::from(vec![Span::styled(
                        &state.config.messages.label_tools,
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::BOLD),
                    )]));
                    for tool in &summary.tools {
                        activity_lines.push(Line::from(vec![
                            Span::styled(" ⏺ ", Style::default().fg(Color::Cyan)),
                            Span::styled(tool.clone(), Style::default().fg(Color::White)),
                        ]));
                    }
                }

                // If no activity info, show status
                if activity_lines.is_empty() {
                    let status_text = match &agent.status {
                        AgentStatus::Idle { label } => {
                            label.as_deref().unwrap_or("Ready for input")
                        }
                        AgentStatus::Processing { activity } => activity.as_str(),
                        AgentStatus::AwaitingApproval { approval_type, .. } => {
                            activity_lines.push(Line::from(vec![
                                Span::styled("⚠ ", Style::default().fg(Color::Red)),
                                Span::styled(
                                    format!("Waiting: {}", approval_type),
                                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                                ),
                            ]));
                            ""
                        }
                        AgentStatus::Error { message } => message.as_str(),
                        AgentStatus::Unknown => "...",
                    };
                    if !status_text.is_empty() && activity_lines.is_empty() {
                        activity_lines.push(Line::from(vec![Span::styled(
                            status_text,
                            Style::default().fg(Color::Gray),
                        )]));
                    }
                }

                let activity_paragraph = Paragraph::new(activity_lines);
                frame.render_widget(activity_paragraph, columns[1]);
            }
        } else {
            // No agent selected
            let block = Block::default()
                .title(" Summary ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Gray));

            let paragraph = Paragraph::new(vec![Line::from(vec![Span::styled(
                "No agent selected",
                Style::default().fg(Color::DarkGray),
            )])])
            .block(block);

            frame.render_widget(paragraph, area);
        }
    }

    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        let agent = state.selected_visible_agent();

        let (title, content) = if let Some(agent) = agent {
            let title = format!(" Preview: {} ({}) ", agent.target, agent.name);

            // Use config-driven parser for approval keys
            let registry = ParserRegistry::with_config(&state.config);
            let (approve_key, reject_key) = if let Some(parser) = registry
                .all_parsers()
                .find(|p| p.agent_name() == agent.name)
            {
                (parser.approval_keys(), parser.rejection_keys())
            } else {
                ("y", "n")
            };

            // Show approval details if awaiting
            let content = if let AgentStatus::AwaitingApproval {
                approval_type,
                details,
            } = &agent.status
            {
                state
                    .config
                    .messages
                    .approval_prompt
                    .replace("{agent_type}", &agent.agent_type.to_string())
                    .replace("{approval_type}", &approval_type.to_string())
                    .replace("{details}", details)
                    .replace("{approve_key}", approve_key)
                    .replace("{reject_key}", reject_key)
            } else {
                // Show last portion of pane content
                // First trim trailing empty lines
                let mut lines: Vec<&str> = agent.last_content.lines().collect();
                while lines.last().is_some_and(|l| l.trim().is_empty()) {
                    lines.pop();
                }
                let start = lines.len().saturating_sub(20);
                lines[start..].join("\n")
            };

            (title, content)
        } else {
            (" Preview ".to_string(), "No agent selected".to_string())
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Gray));

        let paragraph = Paragraph::new(content)
            .block(block)
            .style(Style::default().fg(Color::White));

        frame.render_widget(paragraph, area);
    }

    /// Renders a detailed preview with syntax highlighting for diffs
    pub fn render_detailed(frame: &mut Frame, area: Rect, state: &AppState) {
        let agent = state.selected_visible_agent();

        // Calculate available lines (area height minus border)
        let available_lines = area.height.saturating_sub(2) as usize;

        // Calculate max line width for truncation
        let max_line_width = state
            .config
            .max_line_width
            .map(|w| w as usize)
            .unwrap_or_else(|| area.width.saturating_sub(2) as usize);

        let (title, lines) = if let Some(agent) = agent {
            let title = format!(" {} ({}) ", agent.target, agent.name);

            let mut styled_lines: Vec<Line> = Vec::new();

            // Take enough lines to fill the area
            // First trim trailing empty lines
            let mut content_lines: Vec<&str> = agent.last_content.lines().collect();
            while content_lines.last().is_some_and(|l| l.trim().is_empty()) {
                content_lines.pop();
            }
            let start = content_lines.len().saturating_sub(available_lines);

            let registry = ParserRegistry::with_config(&state.config);
            let parser = registry
                .all_parsers()
                .find(|p| p.agent_name() == agent.name);

            for line in &content_lines[start..] {
                // Always truncate lines to fit display width (no wrapping)
                let (display_line, _was_truncated) = truncate_line(line, max_line_width);

                // Apply syntax highlighting based on config rules
                let style = parser.and_then(|p| p.highlight_line(&display_line));

                // If no rule matched, use default behavior (some fallback highlighting)
                if let Some(s) = style {
                    styled_lines.push(Line::from(vec![Span::styled(display_line, s)]));
                } else {
                    let spans = if display_line.starts_with('+') && !display_line.starts_with("+++")
                    {
                        vec![Span::styled(
                            display_line,
                            Style::default().fg(Color::Green),
                        )]
                    } else if display_line.starts_with('-') && !display_line.starts_with("---") {
                        vec![Span::styled(display_line, Style::default().fg(Color::Red))]
                    } else if display_line.starts_with("@@") {
                        vec![Span::styled(display_line, Style::default().fg(Color::Cyan))]
                    } else if display_line.contains("[y/n]") || display_line.contains("[Y/n]") {
                        vec![Span::styled(
                            display_line,
                            Style::default().fg(Color::Yellow),
                        )]
                    } else if display_line.contains("⚠")
                        || display_line.contains("Error")
                        || display_line.contains("error")
                    {
                        vec![Span::styled(display_line, Style::default().fg(Color::Red))]
                    } else if display_line.starts_with("❯") || display_line.starts_with(">") {
                        vec![Span::styled(display_line, Style::default().fg(Color::Cyan))]
                    } else {
                        vec![Span::raw(display_line)]
                    };
                    styled_lines.push(Line::from(spans));
                }
            }

            (title, styled_lines)
        } else {
            (
                " Preview ".to_string(),
                vec![Line::from(vec![Span::styled(
                    "No agent selected",
                    Style::default().fg(Color::DarkGray),
                )])],
            )
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Gray));

        // Never wrap - lines are truncated, each source line = 1 visual line
        // This ensures "last N lines" shows exactly the last N visual lines
        let paragraph = Paragraph::new(lines).block(block);

        frame.render_widget(paragraph, area);
    }
}
