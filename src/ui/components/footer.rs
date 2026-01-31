use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{AppState, Config};

/// Footer widget showing clickable buttons (single line, no border)
pub struct FooterWidget;

impl FooterWidget {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState, _config: &Config) {
        let sep = state.styles.dimmed;
        let mut spans = Vec::new();

        if !state.selected_agents.is_empty() {
            spans.push(Span::styled(
                format!("({} selected)", state.selected_agents.len()),
                state.styles.header,
            ));
            spans.push(Span::styled(" ", sep));
        }

        if let Some(msg) = &state.last_message {
            let style = match msg.kind {
                crate::app::MessageKind::Info => state.styles.idle,
                crate::app::MessageKind::Success => state.styles.idle,
                crate::app::MessageKind::Error => state.styles.error,
                crate::app::MessageKind::Welcome => state.styles.normal,
            };

            spans.push(Span::styled(
                truncate_error(&msg.text, area.width as usize - 2),
                style,
            ));
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line);
        frame.render_widget(paragraph, area);
    }
}

fn truncate_error(s: &str, max_len: usize) -> String {
    if max_len == 0 {
        return String::new();
    }
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        format!(
            "{}…",
            s.chars()
                .take(max_len.saturating_sub(1))
                .collect::<String>()
        )
    }
}
