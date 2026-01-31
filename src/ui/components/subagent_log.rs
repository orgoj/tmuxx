use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem},
    Frame,
};

use crate::agents::SubagentStatus;
use crate::app::AppState;

/// Widget for displaying subagent activity log
pub struct SubagentLogWidget;

impl SubagentLogWidget {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        let agent = state.selected_agent();

        let title = if let Some(agent) = agent {
            format!(" Subagent Log: {} ", agent.target)
        } else {
            " Subagent Log ".to_string()
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(state.styles.border);

        let items: Vec<ListItem> = if let Some(agent) = agent {
            if agent.subagents.is_empty() {
                vec![ListItem::new(Line::from(vec![Span::styled(
                    "  No subagent activity detected",
                    state.styles.dimmed,
                )]))]
            } else {
                agent
                    .subagents
                    .iter()
                    .map(|subagent| {
                        let (indicator, style) = match subagent.status {
                            SubagentStatus::Running => (
                                state.config.indicators.subagent_running.as_str(),
                                state.styles.subagent_running,
                            ),
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

                        let duration = subagent.duration_str();

                        let line = Line::from(vec![
                            Span::raw("  "),
                            Span::styled(indicator, style),
                            Span::raw(" "),
                            Span::styled(
                                subagent.subagent_type.display_name(),
                                state.styles.normal,
                            ),
                            Span::raw("  "),
                            Span::styled(&subagent.description, state.styles.normal),
                            Span::raw("  "),
                            Span::styled(format!("[{}]", duration), state.styles.dimmed),
                        ]);

                        ListItem::new(line)
                    })
                    .collect()
            }
        } else {
            vec![ListItem::new(Line::from(vec![Span::styled(
                "  No agent selected",
                state.styles.dimmed,
            )]))]
        };

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }
}
