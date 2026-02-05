use anyhow::{anyhow, Result};

use super::config::NotificationMode;
use super::key_binding::{CommandConfig, KeyAction, KillMethod, NavAction};

use super::Config;

/// Represents a configuration override from CLI
#[derive(Debug, Clone)]
pub enum ConfigOverride {
    PollInterval(u64),
    CaptureLines(u32),
    ShowDetachedSessions(bool),
    DebugMode(bool),
    TruncateLongLines(bool),
    MaxLineWidth(Option<u16>),
    KeyBinding(String, KeyAction),
    PopupTriggerKey(String),
    IgnoreSessions(Vec<String>),
    IgnoreSelf(bool),
    LogActions(bool),
    SidebarWidth(super::config::SidebarWidth),
    TerminalWrapper(Option<String>),
    NotificationCommand(Option<String>),
    NotificationDelayMs(u64),
    NotificationMode(NotificationMode),
    TodoCommandTimeoutMs(u64),
}

impl ConfigOverride {
    /// Parse a KEY=VALUE string into a ConfigOverride
    pub fn parse(key: &str, value: &str) -> Result<Self> {
        // For keybindings, we need to preserve the original key name (case-sensitive)
        // because key names like "C-l" need to match exactly when looking up bindings
        if key.starts_with("keybindings.") || key.starts_with("kb.") {
            let binding_key = if let Some(k) = key.strip_prefix("keybindings.") {
                k
            } else {
                key.strip_prefix("kb.").unwrap()
            };
            let action = parse_key_action(value)?;
            return Ok(ConfigOverride::KeyBinding(binding_key.to_string(), action));
        }

        let normalized_key = normalize_key(key);

        match normalized_key.as_str() {
            "pollintervalms" | "pollinterval" => {
                let val = value.parse::<u64>()
                    .map_err(|_| anyhow!("Invalid value for poll_interval_ms: '{}'. Expected a number in milliseconds.", value))?;
                Ok(ConfigOverride::PollInterval(val))
            }
            "capturelines" => {
                let val = value.parse::<u32>()
                    .map_err(|_| anyhow!("Invalid value for capture_lines: '{}'. Expected a positive number.", value))?;
                Ok(ConfigOverride::CaptureLines(val))
            }
            "showdetachedsessions" | "showdetached" => {
                let val = parse_bool(value)
                    .ok_or_else(|| anyhow!(
                        "Invalid value for show_detached_sessions: '{}'. Expected: true/false, 1/0, yes/no, on/off",
                        value
                    ))?;
                Ok(ConfigOverride::ShowDetachedSessions(val))
            }
            "debugmode" | "debug" => {
                let val = parse_bool(value)
                    .ok_or_else(|| anyhow!(
                        "Invalid value for debug_mode: '{}'. Expected: true/false, 1/0, yes/no, on/off",
                        value
                    ))?;
                Ok(ConfigOverride::DebugMode(val))
            }
            "truncatelonglines" | "truncate" => {
                let val = parse_bool(value)
                    .ok_or_else(|| anyhow!(
                        "Invalid value for truncate_long_lines: '{}'. Expected: true/false, 1/0, yes/no, on/off",
                        value
                    ))?;
                Ok(ConfigOverride::TruncateLongLines(val))
            }
            "maxlinewidth" | "linewidth" => {
                let val = if value == "none" {
                    None
                } else {
                    Some(value.parse::<u16>().map_err(|_| {
                        anyhow!(
                            "Invalid value for max_line_width: '{}'. Expected a number or 'none'.",
                            value
                        )
                    })?)
                };
                Ok(ConfigOverride::MaxLineWidth(val))
            }
            "popuptriggerkey" | "popupkey" => {
                Ok(ConfigOverride::PopupTriggerKey(value.to_string()))
            }
            "ignoresessions" | "ignore_sessions" => {
                let sessions: Vec<String> = value
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                Ok(ConfigOverride::IgnoreSessions(sessions))
            }
            "ignoreself" | "ignore_self" => {
                let val = parse_bool(value).ok_or_else(|| {
                    anyhow!(
                        "Invalid value for ignore_self: '{}'. Expected: true/false, 1/0, yes/no, on/off",
                        value
                    )
                })?;
                Ok(ConfigOverride::IgnoreSelf(val))
            }
            "logactions" | "log" => {
                let val = parse_bool(value).ok_or_else(|| {
                    anyhow!(
                        "Invalid value for log_actions: '{}'. Expected: true/false, 1/0, yes/no, on/off",
                        value
                    )
                })?;
                Ok(ConfigOverride::LogActions(val))
            }
            "sidebarwidth" | "sidebar" => {
                let val = if value.contains('%') {
                    super::config::SidebarWidth::Percent(value.to_string())
                } else {
                    let w = value.parse::<u16>().map_err(|_| {
                        anyhow!(
                            "Invalid value for sidebar_width: '{}'. Expected a number or percentage like '25%'.",
                            value
                        )
                    })?;
                    super::config::SidebarWidth::Fixed(w)
                };
                Ok(ConfigOverride::SidebarWidth(val))
            }
            "terminalwrapper" | "wrapper" => {
                let val = if value.is_empty() { None } else { Some(value.to_string()) };
                Ok(ConfigOverride::TerminalWrapper(val))
            }
            "notificationcommand" | "notifycmd" => {
                let val = if value.is_empty() {
                    None
                } else {
                    Some(value.to_string())
                };
                Ok(ConfigOverride::NotificationCommand(val))
            }
            "notificationdelayms" | "notifydelay" => {
                let val = value.parse::<u64>().map_err(|_| {
                    anyhow!(
                        "Invalid value for notification_delay_ms: '{}'. Expected milliseconds.",
                        value
                    )
                })?;
                Ok(ConfigOverride::NotificationDelayMs(val))
            }
            "notificationmode" | "notifymode" => {
                let mode = match value.to_lowercase().as_str() {
                    "first" => NotificationMode::First,
                    "each" => NotificationMode::Each,
                    _ => {
                        return Err(anyhow!(
                            "Invalid notification_mode: '{}'. Use 'first' or 'each'.",
                            value
                        ))
                    }
                };
                Ok(ConfigOverride::NotificationMode(mode))
            }
            "todocommandtimeoutms" | "todotimeout" => {
                let val = value.parse::<u64>().map_err(|_| {
                    anyhow!(
                        "Invalid value for todo_command_timeout_ms: '{}'. Expected milliseconds.",
                        value
                    )
                })?;
                Ok(ConfigOverride::TodoCommandTimeoutMs(val))
            }
            _ => Err(anyhow!(
                "Unknown config key: '{}'. Valid keys: poll_interval_ms, capture_lines, show_detached_sessions, debug_mode, truncate_long_lines, max_line_width, popup_trigger_key, ignore_sessions, ignore_self, log_actions, sidebar_width, terminal_wrapper, notification_command, notification_delay_ms, notification_mode, todo_command_timeout_ms, keybindings.KEY (or kb.KEY)",
                key
            )),
        }
    }

    /// Apply this override to a Config
    pub fn apply(self, config: &mut Config) {
        match self {
            ConfigOverride::PollInterval(val) => config.poll_interval_ms = val,
            ConfigOverride::CaptureLines(val) => config.capture_lines = val,
            ConfigOverride::ShowDetachedSessions(val) => config.show_detached_sessions = val,
            ConfigOverride::DebugMode(val) => config.debug_mode = val,
            ConfigOverride::TruncateLongLines(val) => config.truncate_long_lines = val,
            ConfigOverride::MaxLineWidth(val) => config.max_line_width = val,
            ConfigOverride::KeyBinding(key, action) => {
                config.key_bindings.bindings.insert(key, action);
            }
            ConfigOverride::PopupTriggerKey(val) => config.popup_trigger_key = val,
            ConfigOverride::IgnoreSessions(sessions) => config.ignore_sessions = sessions,
            ConfigOverride::IgnoreSelf(val) => config.ignore_self = val,
            ConfigOverride::LogActions(val) => config.log_actions = val,
            ConfigOverride::SidebarWidth(val) => config.sidebar_width = val,
            ConfigOverride::TerminalWrapper(val) => config.terminal_wrapper = val,
            ConfigOverride::NotificationCommand(val) => config.notification_command = val,
            ConfigOverride::NotificationDelayMs(val) => config.notification_delay_ms = val,
            ConfigOverride::NotificationMode(val) => config.notification_mode = val,
            ConfigOverride::TodoCommandTimeoutMs(val) => config.todo_command_timeout_ms = val,
        }
    }
}

/// Parse a key action from a string value
fn parse_key_action(value: &str) -> Result<KeyAction> {
    match value {
        "approve" => Ok(KeyAction::Approve),
        "reject" => Ok(KeyAction::Reject),
        "approve_all" => Ok(KeyAction::ApproveAll),
        "rename_session" => Ok(KeyAction::RenameSession),
        "refresh" => Ok(KeyAction::Refresh),
        s if s.starts_with("send_number:") => {
            let num = s
                .strip_prefix("send_number:")
                .unwrap()
                .parse::<u8>()
                .map_err(|_| anyhow!("Invalid number for send_number"))?;
            if num > 9 {
                return Err(anyhow!("send_number must be 0-9"));
            }
            Ok(KeyAction::SendNumber(num))
        }
        s if s.starts_with("send_keys:") => {
            let keys = s.strip_prefix("send_keys:").unwrap().to_string();
            Ok(KeyAction::SendKeys(keys))
        }
        s if s.starts_with("kill_app:") => {
            let method = match s.strip_prefix("kill_app:").unwrap() {
                "sigterm" => KillMethod::Sigterm,
                "ctrlc_ctrld" => KillMethod::CtrlCCtrlD,
                _ => return Err(anyhow!("Invalid kill method, use 'sigterm' or 'ctrlc_ctrld'")),
            };
            Ok(KeyAction::KillApp { method })
        }
        s if s.starts_with("navigate:") => {
            let nav = match s.strip_prefix("navigate:").unwrap() {
                "next_agent" => NavAction::NextAgent,
                "prev_agent" => NavAction::PrevAgent,
                _ => return Err(anyhow!("Invalid navigation action, use 'next_agent' or 'prev_agent'")),
            };
            Ok(KeyAction::Navigate(nav))
        }
        s if s.starts_with("command:") => {
            // Format: command:CMD[:blocking][:terminal][:external][:active]
            let cmd_part = s.strip_prefix("command:").unwrap();
            let parts: Vec<&str> = cmd_part.split(':').collect();

            if parts.len() > 1 {
                // Last parts might be flags
                let mut blocking = false;
                let mut terminal = false;
                let mut external_terminal = false;
                let mut active_in_tmux = false;

                // Everything except known flags at the end is part of the command
                // This is still a bit ambiguous if command contains colons
                // So we work backwards from the end
                let mut i = parts.len() - 1;
                let mut flags_found = true;
                while i > 0 && flags_found {
                    match parts[i] {
                        "blocking" => blocking = true,
                        "terminal" => terminal = true,
                        "external" => external_terminal = true,
                        "active" => active_in_tmux = true,
                        _ => {
                            flags_found = false;
                            continue;
                        }
                    }
                    i -= 1;
                }

                let command = parts[0..=i].join(":");
                Ok(KeyAction::ExecuteCommand(CommandConfig {
                    command,
                    blocking,
                    terminal,
                    external_terminal,
                    active_in_tmux,
                }))
            } else {
                Ok(KeyAction::ExecuteCommand(CommandConfig {
                    command: cmd_part.to_string(),
                    blocking: false,
                    terminal: false,
                    external_terminal: false,
                    active_in_tmux: false,
                }))
            }
        }
        _ => Err(anyhow!(
            "Invalid key action: '{}'. Valid formats: approve, reject, approve_all, rename_session, refresh, send_number:N, send_keys:KEYS, kill_app:METHOD, navigate:ACTION, command:CMD[:blocking]",
            value
        )),
    }
}

/// Normalize a config key: remove underscores, hyphens, convert to lowercase
fn normalize_key(key: &str) -> String {
    key.replace(['_', '-'], "").to_lowercase()
}

/// Parse a boolean value from various string formats
fn parse_bool(value: &str) -> Option<bool> {
    match value.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Macro to reduce boilerplate for testing ConfigOverride parsing
    macro_rules! test_parse_override {
        // Test that parsing succeeds and produces expected variant
        ($name:ident, $key:literal, $value:literal, $check:expr) => {
            #[test]
            fn $name() {
                let mut config = Config::default();
                config.apply_override($key, $value).unwrap();
                assert!($check(&config));
            }
        };
    }

    // Macro for testing that parsing fails
    macro_rules! test_parse_fails {
        ($name:ident, $key:literal, $value:literal) => {
            #[test]
            fn $name() {
                assert!(ConfigOverride::parse($key, $value).is_err());
            }
        };
    }

    #[test]
    fn test_normalize_key() {
        assert_eq!(normalize_key("poll_interval_ms"), "pollintervalms");
        assert_eq!(normalize_key("PollIntervalMs"), "pollintervalms");
        assert_eq!(normalize_key("poll-interval-ms"), "pollintervalms");
        assert_eq!(
            normalize_key("show_detached_sessions"),
            "showdetachedsessions"
        );
    }

    #[test]
    fn test_parse_bool() {
        // True variants
        for val in ["true", "TRUE", "1", "yes", "YES", "on", "ON"] {
            assert_eq!(parse_bool(val), Some(true), "Expected true for '{}'", val);
        }
        // False variants
        for val in ["false", "FALSE", "0", "no", "NO", "off", "OFF"] {
            assert_eq!(parse_bool(val), Some(false), "Expected false for '{}'", val);
        }
        // Invalid
        assert_eq!(parse_bool("invalid"), None);
        assert_eq!(parse_bool("2"), None);
    }

    // Poll interval tests
    test_parse_override!(
        test_poll_interval,
        "poll_interval_ms",
        "1000",
        |c: &Config| c.poll_interval_ms == 1000
    );
    test_parse_override!(
        test_poll_interval_alias,
        "pollinterval",
        "2000",
        |c: &Config| c.poll_interval_ms == 2000
    );
    test_parse_fails!(test_poll_interval_invalid, "poll_interval_ms", "invalid");

    // Capture lines tests
    test_parse_override!(test_capture_lines, "capture_lines", "500", |c: &Config| c
        .capture_lines
        == 500);
    test_parse_fails!(test_capture_lines_invalid, "capture_lines", "invalid");

    // Show detached sessions tests
    test_parse_override!(
        test_show_detached_false,
        "show_detached_sessions",
        "false",
        |c: &Config| !c.show_detached_sessions
    );
    test_parse_override!(
        test_show_detached_alias,
        "showdetached",
        "0",
        |c: &Config| !c.show_detached_sessions
    );
    test_parse_override!(
        test_show_detached_true,
        "showdetached",
        "true",
        |c: &Config| c.show_detached_sessions
    );
    test_parse_override!(
        test_show_detached_yes,
        "showdetached",
        "yes",
        |c: &Config| c.show_detached_sessions
    );
    test_parse_fails!(
        test_show_detached_invalid,
        "show_detached_sessions",
        "invalid"
    );

    // Debug mode tests
    test_parse_override!(test_debug_mode_true, "debug_mode", "true", |c: &Config| c
        .debug_mode);
    test_parse_override!(
        test_debug_mode_alias_false,
        "debug",
        "false",
        |c: &Config| !c.debug_mode
    );
    test_parse_override!(test_debug_mode_yes, "debug", "yes", |c: &Config| c
        .debug_mode);
    test_parse_fails!(test_debug_mode_invalid, "debug_mode", "invalid");

    // Log actions tests
    test_parse_override!(
        test_log_actions_false,
        "log_actions",
        "false",
        |c: &Config| !c.log_actions
    );
    test_parse_override!(test_log_actions_alias, "log", "1", |c: &Config| c
        .log_actions);

    // Invalid key test
    test_parse_fails!(test_invalid_key, "invalid_key", "value");

    // Notification config tests
    test_parse_override!(
        test_notification_delay,
        "notification_delay_ms",
        "30000",
        |c: &Config| c.notification_delay_ms == 30000
    );
    test_parse_override!(
        test_notification_mode_first,
        "notification_mode",
        "first",
        |c: &Config| c.notification_mode == NotificationMode::First
    );
    test_parse_override!(
        test_notification_mode_each,
        "notifymode",
        "each",
        |c: &Config| c.notification_mode == NotificationMode::Each
    );
    test_parse_fails!(
        test_notification_mode_invalid,
        "notification_mode",
        "invalid"
    );
    test_parse_fails!(
        test_notification_delay_invalid,
        "notification_delay_ms",
        "not_a_number"
    );

    #[test]
    fn test_notification_command() {
        let override_val =
            ConfigOverride::parse("notification_command", "notify-send '{message}'").unwrap();
        match override_val {
            ConfigOverride::NotificationCommand(Some(cmd)) => {
                assert_eq!(cmd, "notify-send '{message}'");
            }
            _ => panic!("Expected NotificationCommand"),
        }

        // Test empty notification_command (disables notifications)
        let override_val = ConfigOverride::parse("notifycmd", "").unwrap();
        assert!(matches!(
            override_val,
            ConfigOverride::NotificationCommand(None)
        ));
    }

    #[test]
    fn test_parse_command_action() {
        // Test non-blocking command
        let override_val = ConfigOverride::parse("kb.z", "command:echo test").unwrap();
        match override_val {
            ConfigOverride::KeyBinding(
                key,
                KeyAction::ExecuteCommand(CommandConfig {
                    command,
                    blocking,
                    terminal,
                    active_in_tmux,
                    ..
                }),
            ) => {
                assert_eq!(key, "z");
                assert_eq!(command, "echo test");
                assert!(!blocking);
                assert!(!terminal);
                assert!(!active_in_tmux);
            }
            _ => panic!("Expected ExecuteCommand action"),
        }

        // Test blocking command
        let override_val = ConfigOverride::parse("kb.x", "command:ls -la:blocking").unwrap();
        match override_val {
            ConfigOverride::KeyBinding(
                key,
                KeyAction::ExecuteCommand(CommandConfig {
                    command,
                    blocking,
                    terminal,
                    active_in_tmux,
                    ..
                }),
            ) => {
                assert_eq!(key, "x");
                assert_eq!(command, "ls -la");
                assert!(blocking);
                assert!(!terminal);
                assert!(!active_in_tmux);
            }
            _ => panic!("Expected ExecuteCommand action with blocking=true"),
        }

        // Test command with multiple flags
        let override_val =
            ConfigOverride::parse("kb.t", "command:wezterm:terminal:active").unwrap();
        match override_val {
            ConfigOverride::KeyBinding(
                _,
                KeyAction::ExecuteCommand(CommandConfig {
                    command,
                    blocking,
                    terminal,
                    active_in_tmux,
                    ..
                }),
            ) => {
                assert_eq!(command, "wezterm");
                assert!(!blocking);
                assert!(terminal);
                assert!(active_in_tmux);
            }
            _ => panic!("Expected ExecuteCommand action with flags"),
        }

        // Test command with colons in the command itself
        let override_val = ConfigOverride::parse(
            "kb.y",
            "command:wezterm cli attach-session ${SESSION_NAME}:blocking",
        )
        .unwrap();
        match override_val {
            ConfigOverride::KeyBinding(
                key,
                KeyAction::ExecuteCommand(CommandConfig {
                    command,
                    blocking,
                    terminal,
                    ..
                }),
            ) => {
                assert_eq!(key, "y");
                assert_eq!(command, "wezterm cli attach-session ${SESSION_NAME}");
                assert!(blocking);
                assert!(!terminal);
            }
            _ => panic!("Expected ExecuteCommand action"),
        }
    }

    #[test]
    fn test_parse_simple_actions() {
        // Test rename_session
        let override_val = ConfigOverride::parse("kb.r", "rename_session").unwrap();
        assert!(matches!(
            override_val,
            ConfigOverride::KeyBinding(ref key, KeyAction::RenameSession) if key == "r"
        ));

        // Test refresh (preserves original key name)
        let override_val = ConfigOverride::parse("kb.C-l", "refresh").unwrap();
        assert!(matches!(
            override_val,
            ConfigOverride::KeyBinding(ref key, KeyAction::Refresh) if key == "C-l"
        ));
    }
}
