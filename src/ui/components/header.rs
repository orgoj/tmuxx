use crate::app::AppState;
use chrono::Local;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

pub struct HeaderWidget;

impl HeaderWidget {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        let ai_count = state.agents.ai_agent_count();
        let generic_count = state.agents.generic_count();
        let processing = state.agents.processing_count();
        let pending = state.agents.active_count();
        let time = Local::now().format("%H:%M").to_string();

        let mut spans = vec![
            Span::styled(" Tmuxx ", state.styles.header),
            Span::styled("│", state.styles.dimmed),
            Span::styled(format!(" {} agents ", ai_count), state.styles.normal),
        ];

        if generic_count > 0 {
            spans.push(Span::styled("│", state.styles.dimmed));
            spans.push(Span::styled(
                format!(" {} other ", generic_count),
                state.styles.dimmed,
            ));
        }

        // Filter status
        if state.filter_active {
            spans.push(Span::styled("│", state.styles.dimmed));
            spans.push(Span::styled(" [Active] ", state.styles.header));
        }
        if state.filter_selected {
            spans.push(Span::styled("│", state.styles.dimmed));
            spans.push(Span::styled(" [Selected] ", state.styles.highlight));
        }

        // Processing count
        if processing > 0 {
            spans.push(Span::styled("│", state.styles.dimmed));
            spans.push(Span::styled(
                format!(" {} {} working ", state.spinner_frame(), processing),
                state.styles.processing,
            ));
        }

        // Pending count
        spans.push(Span::styled("│", state.styles.dimmed));
        if pending > 0 {
            spans.push(Span::styled(
                format!(" ⚠ {} pending ", pending),
                state.styles.awaiting_approval,
            ));
        } else {
            spans.push(Span::styled(" ✓ ready ", state.styles.idle));
        }

        // System stats: CPU
        spans.push(Span::styled("│", state.styles.dimmed));
        let cpu_color = if state.system_stats.cpu_usage > 80.0 {
            state.styles.error.fg.unwrap_or(Color::Red)
        } else if state.system_stats.cpu_usage > 50.0 {
            state.styles.processing.fg.unwrap_or(Color::Yellow)
        } else {
            state.styles.idle.fg.unwrap_or(Color::Green)
        };
        spans.push(Span::styled(
            format!(" CPU {:4.1}% ", state.system_stats.cpu_usage),
            Style::default().fg(cpu_color),
        ));

        // System stats: Memory
        spans.push(Span::styled("│", state.styles.dimmed));
        let mem_percent = state.system_stats.memory_percent();
        let mem_color = if mem_percent > 80.0 {
            state.styles.error.fg.unwrap_or(Color::Red)
        } else if mem_percent > 60.0 {
            state.styles.processing.fg.unwrap_or(Color::Yellow)
        } else {
            state.styles.idle.fg.unwrap_or(Color::Green)
        };
        spans.push(Span::styled(
            format!(
                " MEM {} ({:.0}%) ",
                state.system_stats.memory_display(),
                mem_percent
            ),
            Style::default().fg(mem_color),
        ));

        // Time
        spans.push(Span::styled("│", state.styles.dimmed));
        spans.push(Span::styled(format!(" {} ", time), state.styles.dimmed));

        let line = Line::from(spans);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(state.styles.border);

        let paragraph = Paragraph::new(line).block(block);
        frame.render_widget(paragraph, area);
    }
}
