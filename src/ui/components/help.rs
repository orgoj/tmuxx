use crate::app::{Config, KeyAction, KillMethod, NavAction};

/// Help popup widget - deprecated, now using modal textarea
pub struct HelpWidget;

impl HelpWidget {
    /// Generate help text as a single string for ModalTextarea
    pub fn generate_help_text(config: &Config) -> String {
        let kb = &config.key_bindings;
        let mut sections: std::collections::BTreeMap<&str, Vec<String>> =
            std::collections::BTreeMap::new();

        // Helper to add line to section
        let mut add_line = |section: &'static str, line: String| {
            sections.entry(section).or_default().push(line);
        };

        // 1. Process all configured bindings
        // We want to group by Action, not by Key.
        // So first, invert the map: Action -> Vec<Key>
        let mut action_to_keys: std::collections::HashMap<&KeyAction, Vec<&String>> =
            std::collections::HashMap::new();

        for (key, action) in &kb.bindings {
            action_to_keys.entry(action).or_default().push(key);
        }

        // Define known actions and their categories/descriptions
        // We iterate through all known KeyAction variants (conceptually) or handling specific logic
        // But to be truly config driven, we should iterate what we found in the config.

        // We will match on the Action references we found in the config map.
        // To keep order consistent, let's sort the actions by some criteria?
        // Or just categorize them into buckets.

        for (action, keys) in action_to_keys {
            let mut sorted_keys = keys.clone();
            sorted_keys.sort();
            let keys_str: String = sorted_keys
                .iter()
                .map(|k| k.as_str())
                .collect::<Vec<&str>>()
                .join(" / ");

            match action {
                // Navigation
                KeyAction::Navigate(NavAction::NextAgent) => {
                    add_line("Navigation", format!("  {:14} Select next agent", keys_str))
                }
                KeyAction::Navigate(NavAction::PrevAgent) => add_line(
                    "Navigation",
                    format!("  {:14} Select previous agent", keys_str),
                ),

                // Agents / Status
                KeyAction::Approve => {
                    add_line("Actions", format!("  {:14} Approve request(s)", keys_str))
                }
                KeyAction::Reject => {
                    add_line("Actions", format!("  {:14} Reject request(s)", keys_str))
                }
                KeyAction::ApproveAll => {
                    add_line("Actions", format!("  {:14} Approve ALL pending", keys_str))
                }
                KeyAction::SendNumber(n) => {
                    add_line("Actions", format!("  {:14} Send number {}", keys_str, n))
                }

                // View / Filters
                KeyAction::TogglePaneTreeMode => {
                    add_line("View", format!("  {:14} Toggle pane tree mode", keys_str))
                }
                KeyAction::ToggleSubagentLog => {
                    add_line("View", format!("  {:14} Toggle subagent log", keys_str))
                }
                KeyAction::ToggleFilterActive => add_line(
                    "Filters",
                    format!("  {:14} Toggle active filter (Non-Idle)", keys_str),
                ),
                KeyAction::ToggleFilterSelected => add_line(
                    "Filters",
                    format!("  {:14} Toggle selected filter", keys_str),
                ),
                KeyAction::ToggleMenu => {
                    add_line("View", format!("  {:14} Toggle command menu", keys_str))
                }
                KeyAction::Refresh => {
                    add_line("View", format!("  {:14} Redraw / Clear error", keys_str))
                }

                // Commands / Custom
                KeyAction::RenameSession => {
                    add_line("Actions", format!("  {:14} Rename session", keys_str))
                }
                KeyAction::CaptureTestCase => {
                    add_line("Dev", format!("  {:14} Capture test case", keys_str))
                }
                KeyAction::SendKeys(s) => {
                    add_line("Actions", format!("  {:14} Send keys: {}", keys_str, s))
                }
                KeyAction::KillApp { method } => {
                    let m = match method {
                        KillMethod::Sigterm => "SIGTERM",
                        KillMethod::CtrlCCtrlD => "C-c C-d",
                    };
                    add_line("Actions", format!("  {:14} Kill app ({})", keys_str, m));
                }
                KeyAction::ExecuteCommand(cmd) => add_line(
                    "Commands",
                    format!("  {:14} Run: {}", keys_str, cmd.command),
                ),
            }
        }

        // 2. Add hardcoded/system bindings (modifiers etc that aren't in config but handled by App)
        // Navigation (Arrows)
        add_line(
            "Navigation",
            "  Down / ↑       Select next agent".to_string(),
        );
        add_line(
            "Navigation",
            "  Up / ↓         Select previous agent".to_string(),
        );
        add_line("Navigation", "  Tab            Cycle agents".to_string());

        // Selection
        add_line("Selection", "  Space          Toggle selection".to_string());
        add_line("Selection", "  Ctrl+a         Select all".to_string());

        // Input
        add_line(
            "Input",
            format!("  {:14} Quick Filter (Name/ID)", config.popup_trigger_key),
        );
        add_line("Input", "  Shift+I        Multi-line input".to_string());
        add_line(
            "Input",
            "  ← / →          Focus Sidebar / Input".to_string(),
        );

        // General
        add_line("General", "  ?              Toggle Help".to_string());
        add_line("General", "  q              Quit".to_string());

        // 3. Assemble final string in desired order
        let category_order = vec![
            "Navigation",
            "Selection",
            "Filters",
            "Actions",
            "View",
            "Commands",
            "Input",
            "General",
            "Dev",
        ];

        let mut final_text = String::new();

        for category in category_order {
            if let Some(lines) = sections.get(category) {
                if !lines.is_empty() {
                    final_text.push_str(&format!("{}\n\n", category));
                    // Sort lines alphabetically within category for cleanliness? Or keep insertion order?
                    // KeyAction iteration order is random (HashMap). Sorting is better.
                    let mut sorted_lines = lines.clone();
                    sorted_lines.sort();
                    for line in sorted_lines {
                        final_text.push_str(&line);
                        final_text.push('\n');
                    }
                    final_text.push('\n');
                }
            }
        }

        final_text.push_str("\nPress Esc to close help");
        final_text
    }
}
