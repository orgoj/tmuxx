use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{AppState, Config, KeyAction};

/// Button definitions for footer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FooterButton {
    Approve,
    Reject,
    ApproveAll,
    ToggleSelect,
    Focus,
    Help,
    Quit,
}

/// Footer widget showing clickable buttons (single line, no border)
pub struct FooterWidget;

impl FooterWidget {
    /// Button layout: returns (label, start_col, end_col, button_type)
    pub fn get_button_layout(
        state: &AppState,
        config: &Config,
    ) -> Vec<(String, u16, u16, FooterButton)> {
        let mut buttons = Vec::new();
        let mut col: u16 = 0;

        if state.is_input_focused() {
            return buttons;
        }

        let kb = &config.key_bindings;

        // Find first key for each action
        let approve_keys = kb.keys_for_action(&KeyAction::Approve);
        let approve_key = approve_keys.first().map(|s| s.as_str()).unwrap_or("Y");

        let reject_keys = kb.keys_for_action(&KeyAction::Reject);
        let reject_key = reject_keys.first().map(|s| s.as_str()).unwrap_or("N");

        let approve_all_keys = kb.keys_for_action(&KeyAction::ApproveAll);
        let approve_all_key = approve_all_keys.first().map(|s| s.as_str()).unwrap_or("A");

        let items = vec![
            (format!(" {} ", approve_key), FooterButton::Approve),
            (format!(" {} ", reject_key), FooterButton::Reject),
            (format!(" {} ", approve_all_key), FooterButton::ApproveAll),
            (" ☐ ".to_string(), FooterButton::ToggleSelect), // Hardcoded
            (" F ".to_string(), FooterButton::Focus),        // Hardcoded
            (" ? ".to_string(), FooterButton::Help),         // Hardcoded
            (" Q ".to_string(), FooterButton::Quit),         // Hardcoded
        ];

        for (label, btn_type) in items {
            buttons.push((label.clone(), col, col + label.len() as u16, btn_type));
            col += label.len() as u16 + 1;
        }

        buttons
    }

    /// Check if a click at (x, y) hits a button
    pub fn hit_test(
        x: u16,
        y: u16,
        area: Rect,
        state: &AppState,
        config: &Config,
    ) -> Option<FooterButton> {
        if y != area.y {
            return None;
        }
        if x < area.x || x >= area.x + area.width {
            return None;
        }

        let rel_x = x - area.x;
        let buttons = Self::get_button_layout(state, config);

        for (_, start, end, button) in buttons {
            if rel_x >= start && rel_x < end {
                return Some(button);
            }
        }

        None
    }

    pub fn render(frame: &mut Frame, area: Rect, state: &AppState, config: &Config) {
        let btn_y = Style::default().fg(Color::Black).bg(Color::Green);
        let btn_n = Style::default().fg(Color::Black).bg(Color::Red);
        let btn_a = Style::default().fg(Color::Black).bg(Color::Yellow);
        let btn_sel = Style::default().fg(Color::Black).bg(Color::Cyan);
        let btn_def = Style::default().fg(Color::Black).bg(Color::Gray);
        let sep = Style::default().fg(Color::DarkGray);
        let key = Style::default().fg(Color::Yellow);
        let txt = Style::default().fg(Color::White);

        let line: Line = if state.is_input_focused() {
            Line::from(vec![
                Span::styled(
                    " INPUT ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("│", sep),
                Span::styled(" Enter", key),
                Span::styled(":Send ", txt),
                Span::styled("S-Enter", key),
                Span::styled(":NL ", txt),
                Span::styled("Esc", key),
                Span::styled(":Back ", txt),
                Span::styled("←→", key),
                Span::styled(":Move", txt),
            ])
        } else {
            let buttons = Self::get_button_layout(state, config);
            let mut spans = Vec::new();

            for (label, _, _, btn_type) in buttons {
                let style = match btn_type {
                    FooterButton::Approve => btn_y,
                    FooterButton::Reject => btn_n,
                    FooterButton::ApproveAll => btn_a,
                    FooterButton::ToggleSelect => btn_sel,
                    _ => btn_def,
                };
                spans.push(Span::styled(label, style));
                spans.push(Span::styled(" ", sep));
            }

            if !state.selected_agents.is_empty() {
                spans.push(Span::styled(
                    format!(" ({}sel)", state.selected_agents.len()),
                    Style::default().fg(Color::Cyan),
                ));
            }

            if let Some(msg) = &state.last_error {
                spans.push(Span::styled(" │ ", sep));
                // Check if it's a status message (starts with ✓) or error
                // Note: ✓ is multi-byte UTF-8, so we strip by chars not bytes
                if msg.starts_with("✓ ") {
                    let text = msg.chars().skip(2).collect::<String>();
                    spans.push(Span::styled(
                        format!("✓ {}", truncate_error(&text, 30)),
                        Style::default().fg(Color::Green),
                    ));
                } else {
                    spans.push(Span::styled(
                        format!("✗ {}", truncate_error(msg, 30)),
                        Style::default().fg(Color::Red),
                    ));
                };
            }

            Line::from(spans)
        };

        let paragraph = Paragraph::new(line);
        frame.render_widget(paragraph, area);
    }
}

fn truncate_error(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max_len - 1).collect::<String>())
    }
}
