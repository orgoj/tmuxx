use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Navigation actions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavAction {
    NextAgent,
    PrevAgent,
}

/// Method for killing applications in tmux panes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KillMethod {
    /// Send SIGTERM to process (graceful shutdown)
    Sigterm,
    /// Send Ctrl-C then Ctrl-D sequence (forced interrupt)
    CtrlCCtrlD,
}

/// Actions that can be bound to keys
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum KeyAction {
    /// Navigate in UI
    Navigate(NavAction),
    /// Approve current/selected agent(s)
    Approve,
    /// Reject current/selected agent(s)
    Reject,
    /// Approve all pending requests
    ApproveAll,
    /// Send a number choice (0-9)
    SendNumber(u8),
    /// Send raw text/keys to tmux pane
    SendKeys(String),
    /// Kill the application in target pane
    KillApp { method: KillMethod },
    /// Rename current session
    RenameSession,
    /// Capture current pane content as a test case
    CaptureTestCase,
    /// Refresh/redraw the screen
    Refresh,
    /// Execute a shell command with variable expansion
    ExecuteCommand(CommandConfig),
    /// Toggle command menu
    ToggleMenu,
    /// Toggle subagent log display
    ToggleSubagentLog,
}

/// Configuration for command execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandConfig {
    pub command: String,
    #[serde(default)]
    pub blocking: bool,
    #[serde(default)]
    pub terminal: bool,
}

/// Holds all key binding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    #[serde(flatten)]
    pub bindings: HashMap<String, KeyAction>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        // All default key bindings are now defined in defaults.toml
        // to check config structure, see src/config/defaults.toml
        let bindings = HashMap::new();
        Self { bindings }
    }
}

impl KeyBindings {
    /// Get action for a key press (case-sensitive)
    pub fn get_action(&self, key: &str) -> Option<&KeyAction> {
        self.bindings.get(key)
    }

    /// Get all keys mapped to a specific action (for help display)
    /// Returns keys sorted: lowercase first, then uppercase, then others
    pub fn keys_for_action(&self, target: &KeyAction) -> Vec<String> {
        let mut keys: Vec<String> = self
            .bindings
            .iter()
            .filter(|(_, action)| *action == target)
            .map(|(key, _)| key.clone())
            .collect();

        // Sort: lowercase first (a-z), then uppercase (A-Z), then others
        keys.sort_by(|a, b| {
            let a_chars = a.chars().next();
            let b_chars = b.chars().next();

            match (a_chars, b_chars) {
                (Some(ac), Some(bc)) => {
                    // Both lowercase or both uppercase - natural order
                    if (ac.is_lowercase() && bc.is_lowercase())
                        || (ac.is_uppercase() && bc.is_uppercase())
                    {
                        a.cmp(b)
                    }
                    // Lowercase before uppercase
                    else if ac.is_lowercase() && bc.is_uppercase() {
                        std::cmp::Ordering::Less
                    } else if ac.is_uppercase() && bc.is_lowercase() {
                        std::cmp::Ordering::Greater
                    }
                    // Everything else by natural order
                    else {
                        a.cmp(b)
                    }
                }
                _ => a.cmp(b),
            }
        });

        keys
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keys_for_action() {
        let mut bindings = HashMap::new();
        bindings.insert("y".to_string(), KeyAction::Approve);
        bindings.insert("Y".to_string(), KeyAction::Approve);
        let kb = KeyBindings { bindings };

        let approve_keys = kb.keys_for_action(&KeyAction::Approve);
        assert!(approve_keys.contains(&"y".to_string()));
        assert!(approve_keys.contains(&"Y".to_string()));
        assert_eq!(approve_keys.len(), 2);
    }

    #[test]
    fn test_keys_for_action_sorted() {
        let mut bindings = HashMap::new();
        bindings.insert("y".to_string(), KeyAction::Approve);
        bindings.insert("Y".to_string(), KeyAction::Approve);
        bindings.insert("n".to_string(), KeyAction::Reject);
        bindings.insert("N".to_string(), KeyAction::Reject);
        bindings.insert("a".to_string(), KeyAction::ApproveAll);
        bindings.insert("A".to_string(), KeyAction::ApproveAll);
        let kb = KeyBindings { bindings };

        // Test that lowercase comes before uppercase
        let approve_keys = kb.keys_for_action(&KeyAction::Approve);
        assert_eq!(approve_keys, vec!["y".to_string(), "Y".to_string()]);

        let reject_keys = kb.keys_for_action(&KeyAction::Reject);
        assert_eq!(reject_keys, vec!["n".to_string(), "N".to_string()]);

        let approve_all_keys = kb.keys_for_action(&KeyAction::ApproveAll);
        assert_eq!(approve_all_keys, vec!["a".to_string(), "A".to_string()]);
    }

    #[test]
    fn test_toml_roundtrip() {
        let mut bindings = HashMap::new();
        bindings.insert("j".to_string(), KeyAction::Navigate(NavAction::NextAgent));
        bindings.insert("E".to_string(), KeyAction::SendKeys("Escape".to_string()));
        bindings.insert("y".to_string(), KeyAction::Approve);
        bindings.insert(
            "K".to_string(),
            KeyAction::KillApp {
                method: KillMethod::Sigterm,
            },
        );

        let kb = KeyBindings { bindings };

        // Serialize to TOML
        let toml_str = toml::to_string_pretty(&kb).unwrap();
        println!("Serialized TOML:\n{}", toml_str);

        // Deserialize back
        let parsed: KeyBindings = toml::from_str(&toml_str).unwrap();

        // Verify round-trip
        assert_eq!(kb.bindings.len(), parsed.bindings.len());
        assert_eq!(
            parsed.get_action("j"),
            Some(&KeyAction::Navigate(NavAction::NextAgent))
        );
        assert_eq!(
            parsed.get_action("E"),
            Some(&KeyAction::SendKeys("Escape".to_string()))
        );
        assert_eq!(parsed.get_action("y"), Some(&KeyAction::Approve));
    }

    #[test]
    fn test_execute_command_toml() {
        let mut bindings = HashMap::new();
        bindings.insert(
            "z".to_string(),
            KeyAction::ExecuteCommand(CommandConfig {
                command: "zede ${SESSION_DIR}".to_string(),
                blocking: false,
                terminal: false,
            }),
        );
        bindings.insert(
            "M-t".to_string(),
            KeyAction::ExecuteCommand(CommandConfig {
                command: "wezterm cli attach ${SESSION_NAME}".to_string(),
                blocking: true,
                terminal: false,
            }),
        );

        let kb = KeyBindings { bindings };

        // Serialize to TOML
        let toml_str = toml::to_string_pretty(&kb).unwrap();
        println!("Serialized TOML:\n{}", toml_str);

        // Deserialize back
        let parsed: KeyBindings = toml::from_str(&toml_str).unwrap();

        // Verify round-trip
        assert_eq!(kb.bindings.len(), parsed.bindings.len());
        match parsed.get_action("z") {
            Some(KeyAction::ExecuteCommand(CommandConfig {
                command,
                blocking,
                terminal,
            })) => {
                assert_eq!(command, "zede ${SESSION_DIR}");
                assert!(!blocking);
                assert!(!terminal);
            }
            _ => panic!("Expected ExecuteCommand action"),
        }
        match parsed.get_action("M-t") {
            Some(KeyAction::ExecuteCommand(CommandConfig {
                command,
                blocking,
                terminal,
            })) => {
                assert_eq!(command, "wezterm cli attach ${SESSION_NAME}");
                assert!(blocking);
                assert!(!terminal);
            }
            _ => panic!("Expected ExecuteCommand action"),
        }
    }

    #[test]
    fn test_invalid_command_format_rejected() {
        // Test that using 'command' instead of 'execute_command' causes an error
        // This ensures users get immediate feedback instead of silent failure
        let toml_str = r#"
            [key_bindings]
            z = { command = "zede ${SESSION_DIR}" }
        "#;

        let result: std::result::Result<KeyBindings, _> = toml::from_str(toml_str);
        assert!(
            result.is_err(),
            "Expected error for invalid 'command' format, got: {:?}",
            result
        );
    }
}
