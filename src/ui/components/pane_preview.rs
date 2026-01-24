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

/// Parsed summary info from Claude Code content
struct ClaudeCodeSummary {
    /// Current status/activity line (✽ ...)
    current_activity: Option<String>,
    /// TODO items: (is_completed, text)
    todos: Vec<(bool, String)>,
    /// Recent tool executions (⏺ ...)
    recent_tools: Vec<String>,
}

impl ClaudeCodeSummary {
    fn parse(content: &str) -> Self {
        let mut current_activity = None;
        let mut todos = Vec::new();
        let mut recent_tools = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // Current activity: ✽ text... or · text...
            if trimmed.starts_with('✽') || trimmed.starts_with('·') {
                let activity = trimmed
                    .trim_start_matches('✽')
                    .trim_start_matches('·')
                    .trim();
                // Extract just the main part (before parentheses with timing info)
                let main_part = activity.split('(').next().unwrap_or(activity).trim();
                if !main_part.is_empty() {
                    current_activity = Some(main_part.to_string());
                }
            }

            // TODOs: ☐ (pending) or ☑/✓ (completed)
            if trimmed.starts_with('☐') {
                let text = trimmed.trim_start_matches('☐').trim();
                todos.push((false, text.to_string()));
            } else if trimmed.starts_with('☑') || trimmed.starts_with('✓') {
                let text = trimmed
                    .trim_start_matches('☑')
                    .trim_start_matches('✓')
                    .trim();
                todos.push((true, text.to_string()));
            }

            // Tool executions: ⏺ Tool(...) or ⏺ text
            if trimmed.starts_with('⏺') {
                let tool_text = trimmed.trim_start_matches('⏺').trim();
                // Only keep recent non-completed ones, or limit to last few
                if !tool_text.contains("completed")
                    && !tool_text.contains("finished")
                    && recent_tools.len() < 3
                {
                    // Truncate long tool lines (character-based for UTF-8 safety)
                    let char_count = tool_text.chars().count();
                    let short = if char_count > 60 {
                        let truncated: String = tool_text.chars().take(57).collect();
                        format!("{}...", truncated)
                    } else {
                        tool_text.to_string()
                    };
                    recent_tools.push(short);
                }
            }
        }

        Self {
            current_activity,
            todos,
            recent_tools,
        }
    }
}

/// Widget for previewing the selected pane content
pub struct PanePreviewWidget;

impl PanePreviewWidget {
    /// Render a summary view with TODO (left) and activity (right) in 2-column layout
    pub fn render_summary(frame: &mut Frame, area: Rect, state: &AppState) {
        let agent = state.selected_agent();

        if let Some(agent) = agent {
            let summary = ClaudeCodeSummary::parse(&agent.last_content);

            // Outer block for the entire summary area
            let outer_block = Block::default()
                .title(format!(" {} ", agent.agent_type.short_name()))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Gray));

            let inner_area = outer_block.inner(area);
            frame.render_widget(outer_block, area);

            // Split into 2 columns: TODO (left 50%) | Activity (right 50%)
            let columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(inner_area);

            // Left column: TODOs
            let mut todo_lines: Vec<Line> = Vec::new();
            if !summary.todos.is_empty() {
                todo_lines.push(Line::from(vec![Span::styled(
                    "TODOs:",
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::BOLD),
                )]));
                for (completed, text) in &summary.todos {
                    let (icon, style) = if *completed {
                        ("☑ ", Style::default().fg(Color::DarkGray))
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
                    "No TODOs",
                    Style::default().fg(Color::DarkGray),
                )]));
            }

            let todo_paragraph = Paragraph::new(todo_lines);
            frame.render_widget(todo_paragraph, columns[0]);

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
            if !summary.recent_tools.is_empty() {
                activity_lines.push(Line::from(vec![Span::styled(
                    "Tools:",
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::BOLD),
                )]));
                for tool in &summary.recent_tools {
                    activity_lines.push(Line::from(vec![
                        Span::styled(" ⏺ ", Style::default().fg(Color::Cyan)),
                        Span::styled(tool.clone(), Style::default().fg(Color::White)),
                    ]));
                }
            }

            // If no activity info, show status
            if activity_lines.is_empty() {
                let status_text = match &agent.status {
                    AgentStatus::Idle => "Ready for input",
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
        let agent = state.selected_agent();

        let (title, content) = if let Some(agent) = agent {
            let title = format!(" Preview: {} ({}) ", agent.target, agent.agent_type);

            // Show approval details if awaiting
            let content = if let AgentStatus::AwaitingApproval {
                approval_type,
                details,
            } = &agent.status
            {
                format!(
                    "⚠ {} wants: {}\n\nDetails: {}\n\nPress [Y] to approve or [N] to reject",
                    agent.agent_type, approval_type, details
                )
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
        let agent = state.selected_agent();

        // Calculate available lines (area height minus border)
        let available_lines = area.height.saturating_sub(2) as usize;

        // Calculate max line width for truncation
        let max_line_width = state
            .config
            .max_line_width
            .map(|w| w as usize)
            .unwrap_or_else(|| area.width.saturating_sub(2) as usize);

        let (title, lines) = if let Some(agent) = agent {
            let title = format!(" {} ({}) ", agent.target, agent.agent_type);

            let mut styled_lines: Vec<Line> = Vec::new();

            // Take enough lines to fill the area
            // First trim trailing empty lines
            let mut content_lines: Vec<&str> = agent.last_content.lines().collect();
            while content_lines.last().is_some_and(|l| l.trim().is_empty()) {
                content_lines.pop();
            }
            let start = content_lines.len().saturating_sub(available_lines);

            for line in &content_lines[start..] {
                // Always truncate lines to fit display width (no wrapping)
                let (display_line, _was_truncated) = truncate_line(line, max_line_width);

                // Apply syntax highlighting to the display line
                let spans = if display_line.starts_with('+') && !display_line.starts_with("+++") {
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
