use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, BorderType, Borders, Clear},
    Frame,
};

use crate::app::{Config, KeyAction, KillMethod, NavAction};
use crate::ui::Layout;

/// Help popup widget - deprecated, now using modal textarea
pub struct HelpWidget;

impl HelpWidget {
    /// Generate help text as a single string for modal textarea
    pub fn generate_help_text(config: &Config) -> String {
        let kb = &config.key_bindings;
        let mut lines: Vec<String> = Vec::new();

        // Navigation - dynamic from config
        lines.push(String::from("Navigation"));
        lines.push(String::from(""));

        // Find keys for navigation
        let next_keys = kb.keys_for_action(&KeyAction::Navigate(NavAction::NextAgent));
        let prev_keys = kb.keys_for_action(&KeyAction::Navigate(NavAction::PrevAgent));

        if !next_keys.is_empty() {
            lines.push(format!(
                "  {} / ↓  Next agent",
                next_keys.join(" / ")
            ));
        } else {
            lines.push(String::from("  j / ↓  Next agent"));
        }

        if !prev_keys.is_empty() {
            lines.push(format!(
                "  {} / ↑  Previous agent",
                prev_keys.join(" / ")
            ));
        } else {
            lines.push(String::from("  k / ↑  Previous agent"));
        }

        lines.push(String::from("  Tab      Next agent (cycle)"));
        lines.push(String::from(""));

        // Selection (hardcoded)
        lines.push(String::from("Selection"));
        lines.push(String::from(""));
        lines.push(String::from("  Space    Toggle selection"));
        lines.push(String::from("  Ctrl+a   Select all"));
        lines.push(String::from("  Ctrl+d   Deselect all"));
        lines.push(String::from(""));

        // Actions (dynamic from config)
        lines.push(String::from("Actions"));
        lines.push(String::from(""));

        // Find approval keys
        let approve_keys = kb.keys_for_action(&KeyAction::Approve);
        if !approve_keys.is_empty() {
            lines.push(format!(
                "  {:9}Approve pending request(s)",
                approve_keys.join(" / ")
            ));
        }

        let reject_keys = kb.keys_for_action(&KeyAction::Reject);
        if !reject_keys.is_empty() {
            lines.push(format!(
                "  {:9}Reject pending request(s)",
                reject_keys.join(" / ")
            ));
        }

        let approve_all_keys = kb.keys_for_action(&KeyAction::ApproveAll);
        if !approve_all_keys.is_empty() {
            lines.push(format!(
                "  {:9}Approve all pending requests",
                approve_all_keys.join(" / ")
            ));
        }

        // Number keys
        let mut number_keys = Vec::new();
        for i in 0..=9 {
            let key_str = i.to_string();
            if kb.get_action(key_str.as_str()).is_some() {
                number_keys.push(i);
            }
        }
        if !number_keys.is_empty() {
            lines.push(format!("  0-9      Quick approve (1=yes, 2=no)"));
        }

        // SendKeys actions
        for (key, action) in &kb.bindings {
            if let KeyAction::SendKeys(keys) = action {
                lines.push(format!("  {:9}Send {} to agent", key, keys));
            }
        }

        // Kill actions
        for (key, action) in &kb.bindings {
            if let KeyAction::KillApp { method } = action {
                let method_str = match method {
                    KillMethod::Sigterm => "SIGTERM",
                    KillMethod::CtrlCCtrlD => "Ctrl-C+Ctrl-D",
                };
                lines.push(format!("  {:9}Kill app ({})", key, method_str));
            }
        }

        // Rename session
        let rename_keys = kb.keys_for_action(&KeyAction::RenameSession);
        if !rename_keys.is_empty() {
            lines.push(format!(
                "  {:9}Rename current session",
                rename_keys.join(" / ")
            ));
        }

        // ExecuteCommand actions
        for (key, action) in &kb.bindings {
            if let KeyAction::ExecuteCommand { command, .. } = action {
                lines.push(format!("  {:9}Execute: {}", key, command));
            }
        }

        lines.push(String::from("  ← / →    Switch focus (Sidebar / Input)"));
        lines.push(String::from(""));
        lines.push(format!(
            "  {:9}Show popup input dialog",
            config.popup_trigger_key
        ));
        lines.push(String::from("  Shift+I  Open multi-line input modal"));
        lines.push(String::from(""));

        // View (hardcoded)
        lines.push(String::from("View"));
        lines.push(String::from(""));
        lines.push(String::from("  ?        Show/hide help"));
        lines.push(String::from(""));

        // Refresh - show configured key or default
        let refresh_keys = kb.keys_for_action(&KeyAction::Refresh);
        if !refresh_keys.is_empty() {
            lines.push(format!(
                "  {:9}Refresh / clear error",
                refresh_keys.join(" / ")
            ));
        }
        lines.push(String::from(""));

        // General (hardcoded)
        lines.push(String::from("General"));
        lines.push(String::from(""));
        lines.push(String::from("  q / Esc  Quit"));
        lines.push(String::from("  /        Toggle filter mode"));
        lines.push(String::from(""));
        lines.push(String::from("Press Esc to close help"));

        lines.join("\n")
    }
}
