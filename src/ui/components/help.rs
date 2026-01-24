use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{Config, KeyAction, KillMethod, NavAction};
use crate::ui::Layout;

/// Help popup widget
pub struct HelpWidget;

impl HelpWidget {
    pub fn render(frame: &mut Frame, area: Rect, config: &Config) {
        let popup_area = Layout::centered_popup(area, 60, 70);

        // Clear the background
        frame.render_widget(Clear, popup_area);

        let key_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        let desc_style = Style::default().fg(Color::White);
        let section_style = Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);

        let kb = &config.key_bindings;
        let mut help_text = Vec::new();

        // Navigation - dynamic from config
        help_text.push(Line::from(vec![Span::styled("Navigation", section_style)]));
        help_text.push(Line::from(vec![]));

        // Find keys for navigation
        let next_keys = kb.keys_for_action(&KeyAction::Navigate(NavAction::NextAgent));
        let prev_keys = kb.keys_for_action(&KeyAction::Navigate(NavAction::PrevAgent));

        if !next_keys.is_empty() {
            let keys_str = format!("  {} / ↓  ", next_keys.join(" / "));
            help_text.push(Line::from(vec![
                Span::styled(format!("{:11}", keys_str), key_style),
                Span::styled("Next agent", desc_style),
            ]));
        } else {
            help_text.push(Line::from(vec![
                Span::styled("  ↓        ", key_style),
                Span::styled("Next agent", desc_style),
            ]));
        }

        if !prev_keys.is_empty() {
            let keys_str = format!("  {} / ↑  ", prev_keys.join(" / "));
            help_text.push(Line::from(vec![
                Span::styled(format!("{:11}", keys_str), key_style),
                Span::styled("Previous agent", desc_style),
            ]));
        } else {
            help_text.push(Line::from(vec![
                Span::styled("  ↑        ", key_style),
                Span::styled("Previous agent", desc_style),
            ]));
        }

        help_text.push(Line::from(vec![
            Span::styled("  Tab      ", key_style),
            Span::styled("Next agent (cycle)", desc_style),
        ]));
        help_text.push(Line::from(vec![]));

        // Selection (hardcoded)
        help_text.push(Line::from(vec![Span::styled("Selection", section_style)]));
        help_text.push(Line::from(vec![]));
        help_text.push(Line::from(vec![
            Span::styled("  Space    ", key_style),
            Span::styled("Toggle selection of current agent", desc_style),
        ]));
        help_text.push(Line::from(vec![
            Span::styled("  Ctrl+a   ", key_style),
            Span::styled("Select all agents", desc_style),
        ]));
        help_text.push(Line::from(vec![
            Span::styled("  Esc      ", key_style),
            Span::styled("Clear selection / Close subagent log", desc_style),
        ]));
        help_text.push(Line::from(vec![]));

        // Actions (dynamic from config)
        help_text.push(Line::from(vec![Span::styled("Actions", section_style)]));
        help_text.push(Line::from(vec![]));

        // Find approval keys
        let approve_keys = kb.keys_for_action(&KeyAction::Approve);
        if !approve_keys.is_empty() {
            let keys_str = format!("  {:9}", approve_keys.join(" / "));
            help_text.push(Line::from(vec![
                Span::styled(keys_str, key_style),
                Span::styled("Approve pending request(s)", desc_style),
            ]));
        }

        let reject_keys = kb.keys_for_action(&KeyAction::Reject);
        if !reject_keys.is_empty() {
            let keys_str = format!("  {:9}", reject_keys.join(" / "));
            help_text.push(Line::from(vec![
                Span::styled(keys_str, key_style),
                Span::styled("Reject pending request(s)", desc_style),
            ]));
        }

        let approve_all_keys = kb.keys_for_action(&KeyAction::ApproveAll);
        if !approve_all_keys.is_empty() {
            let keys_str = format!("  {:9}", approve_all_keys.join(" / "));
            help_text.push(Line::from(vec![
                Span::styled(keys_str, key_style),
                Span::styled("Approve all pending requests", desc_style),
            ]));
        }

        // Number keys
        let mut number_keys = Vec::new();
        for i in 0..=9 {
            if kb.get_action(&i.to_string()).is_some() {
                number_keys.push(i.to_string());
            }
        }
        if !number_keys.is_empty() {
            help_text.push(Line::from(vec![
                Span::styled("  0-9      ", key_style),
                Span::styled("Send number choice to agent", desc_style),
            ]));
        }

        // SendKeys actions
        for (key, action) in &kb.bindings {
            if let KeyAction::SendKeys(keys) = action {
                let keys_str = format!("  {:9}", key);
                help_text.push(Line::from(vec![
                    Span::styled(keys_str, key_style),
                    Span::styled(format!("Send {} to agent", keys), desc_style),
                ]));
            }
        }

        // Kill actions
        for (key, action) in &kb.bindings {
            if let KeyAction::KillApp { method } = action {
                let method_str = match method {
                    KillMethod::Sigterm => "SIGTERM",
                    KillMethod::CtrlCCtrlD => "Ctrl-C+Ctrl-D",
                };
                let keys_str = format!("  {:9}", key);
                help_text.push(Line::from(vec![
                    Span::styled(keys_str, key_style),
                    Span::styled(format!("Kill app ({})", method_str), desc_style),
                ]));
            }
        }

        // Rename session
        let rename_keys = kb.keys_for_action(&KeyAction::RenameSession);
        if !rename_keys.is_empty() {
            let keys_str = format!("  {:9}", rename_keys.join(" / "));
            help_text.push(Line::from(vec![
                Span::styled(keys_str, key_style),
                Span::styled("Rename current session", desc_style),
            ]));
        }

        // ExecuteCommand actions
        for (key, action) in &kb.bindings {
            if let KeyAction::ExecuteCommand { command, .. } = action {
                let keys_str = format!("  {:9}", key);
                help_text.push(Line::from(vec![
                    Span::styled(keys_str, key_style),
                    Span::styled(format!("Execute: {}", command), desc_style),
                ]));
            }
        }

        help_text.push(Line::from(vec![
            Span::styled("  ← / →    ", key_style),
            Span::styled("Switch focus (Sidebar / Input)", desc_style),
        ]));
        help_text.push(Line::from(vec![
            Span::styled("  f / F    ", key_style),
            Span::styled("Focus on selected pane in tmux", desc_style),
        ]));
        help_text.push(Line::from(vec![
            Span::styled(format!("  {:9}", config.popup_trigger_key), key_style),
            Span::styled("Show popup input dialog", desc_style),
        ]));
        help_text.push(Line::from(vec![]));

        // View (hardcoded)
        help_text.push(Line::from(vec![Span::styled("View", section_style)]));
        help_text.push(Line::from(vec![]));
        help_text.push(Line::from(vec![
            Span::styled("  s / S    ", key_style),
            Span::styled("Toggle subagent log", desc_style),
        ]));
        help_text.push(Line::from(vec![
            Span::styled("  t / T    ", key_style),
            Span::styled("Toggle TODO/Tools display", desc_style),
        ]));
        // Refresh - show configured key or default
        let refresh_keys = kb.keys_for_action(&KeyAction::Refresh);
        if !refresh_keys.is_empty() {
            let keys_str = format!("  {:9}", refresh_keys.join(" / "));
            help_text.push(Line::from(vec![
                Span::styled(keys_str, key_style),
                Span::styled("Refresh / clear error", desc_style),
            ]));
        }
        help_text.push(Line::from(vec![]));

        // General (hardcoded)
        help_text.push(Line::from(vec![Span::styled("General", section_style)]));
        help_text.push(Line::from(vec![]));
        help_text.push(Line::from(vec![
            Span::styled("  h / ?    ", key_style),
            Span::styled("Toggle this help", desc_style),
        ]));
        help_text.push(Line::from(vec![
            Span::styled("  q        ", key_style),
            Span::styled("Quit", desc_style),
        ]));
        help_text.push(Line::from(vec![]));
        help_text.push(Line::from(vec![Span::styled(
            "  Press any key to close this help",
            Style::default().fg(Color::DarkGray),
        )]));

        let block = Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black));

        let paragraph = Paragraph::new(help_text).block(block);

        frame.render_widget(paragraph, popup_area);
    }
}
