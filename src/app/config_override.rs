use anyhow::{anyhow, Result};

use super::key_binding::{KeyAction, KillMethod, NavAction};
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
}

impl ConfigOverride {
    /// Parse a KEY=VALUE string into a ConfigOverride
    pub fn parse(key: &str, value: &str) -> Result<Self> {
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
            s if s.starts_with("keybindings.") || s.starts_with("kb.") => {
                let key = if let Some(k) = normalized_key.strip_prefix("keybindings.") {
                    k
                } else {
                    normalized_key.strip_prefix("kb.").unwrap()
                };

                let action = parse_key_action(value)?;
                Ok(ConfigOverride::KeyBinding(key.to_string(), action))
            }
            _ => Err(anyhow!(
                "Unknown config key: '{}'. Valid keys: poll_interval_ms, capture_lines, show_detached_sessions, debug_mode, truncate_long_lines, max_line_width, keybindings.KEY (or kb.KEY)",
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
        }
    }
}

/// Parse a key action from a string value
fn parse_key_action(value: &str) -> Result<KeyAction> {
    match value {
        "approve" => Ok(KeyAction::Approve),
        "reject" => Ok(KeyAction::Reject),
        "approve_all" => Ok(KeyAction::ApproveAll),
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
        _ => Err(anyhow!(
            "Invalid key action: '{}'. Valid formats: approve, reject, approve_all, send_number:N, send_keys:KEYS, kill_app:METHOD, navigate:ACTION",
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
        assert_eq!(parse_bool("true"), Some(true));
        assert_eq!(parse_bool("TRUE"), Some(true));
        assert_eq!(parse_bool("1"), Some(true));
        assert_eq!(parse_bool("yes"), Some(true));
        assert_eq!(parse_bool("YES"), Some(true));
        assert_eq!(parse_bool("on"), Some(true));
        assert_eq!(parse_bool("ON"), Some(true));

        // False variants
        assert_eq!(parse_bool("false"), Some(false));
        assert_eq!(parse_bool("FALSE"), Some(false));
        assert_eq!(parse_bool("0"), Some(false));
        assert_eq!(parse_bool("no"), Some(false));
        assert_eq!(parse_bool("NO"), Some(false));
        assert_eq!(parse_bool("off"), Some(false));
        assert_eq!(parse_bool("OFF"), Some(false));

        // Invalid
        assert_eq!(parse_bool("invalid"), None);
        assert_eq!(parse_bool("2"), None);
    }

    #[test]
    fn test_parse_poll_interval() {
        let override_val = ConfigOverride::parse("poll_interval_ms", "1000").unwrap();
        match override_val {
            ConfigOverride::PollInterval(val) => assert_eq!(val, 1000),
            _ => panic!("Wrong variant"),
        }

        // Test alias
        let override_val = ConfigOverride::parse("pollinterval", "2000").unwrap();
        match override_val {
            ConfigOverride::PollInterval(val) => assert_eq!(val, 2000),
            _ => panic!("Wrong variant"),
        }

        // Test invalid value
        assert!(ConfigOverride::parse("poll_interval_ms", "invalid").is_err());
    }

    #[test]
    fn test_parse_capture_lines() {
        let override_val = ConfigOverride::parse("capture_lines", "500").unwrap();
        match override_val {
            ConfigOverride::CaptureLines(val) => assert_eq!(val, 500),
            _ => panic!("Wrong variant"),
        }

        // Test invalid value
        assert!(ConfigOverride::parse("capture_lines", "invalid").is_err());
    }

    #[test]
    fn test_parse_show_detached_sessions() {
        // Full name
        let override_val = ConfigOverride::parse("show_detached_sessions", "false").unwrap();
        match override_val {
            ConfigOverride::ShowDetachedSessions(val) => assert!(!val),
            _ => panic!("Wrong variant"),
        }

        // Short alias
        let override_val = ConfigOverride::parse("showdetached", "0").unwrap();
        match override_val {
            ConfigOverride::ShowDetachedSessions(val) => assert!(!val),
            _ => panic!("Wrong variant"),
        }

        // Various true formats
        for true_val in &["true", "1", "yes", "on"] {
            let override_val = ConfigOverride::parse("showdetached", true_val).unwrap();
            match override_val {
                ConfigOverride::ShowDetachedSessions(val) => assert!(val),
                _ => panic!("Wrong variant"),
            }
        }

        // Invalid value
        assert!(ConfigOverride::parse("show_detached_sessions", "invalid").is_err());
    }

    #[test]
    fn test_parse_invalid_key() {
        let result = ConfigOverride::parse("invalid_key", "value");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown config key"));
    }

    #[test]
    fn test_parse_debug_mode() {
        // Full name
        let override_val = ConfigOverride::parse("debug_mode", "true").unwrap();
        match override_val {
            ConfigOverride::DebugMode(val) => assert!(val),
            _ => panic!("Wrong variant"),
        }

        // Short alias
        let override_val = ConfigOverride::parse("debug", "false").unwrap();
        match override_val {
            ConfigOverride::DebugMode(val) => assert!(!val),
            _ => panic!("Wrong variant"),
        }

        // Various true formats
        for true_val in &["true", "1", "yes", "on"] {
            let override_val = ConfigOverride::parse("debug", true_val).unwrap();
            match override_val {
                ConfigOverride::DebugMode(val) => assert!(val),
                _ => panic!("Wrong variant"),
            }
        }

        // Invalid value
        assert!(ConfigOverride::parse("debug_mode", "invalid").is_err());
    }

    #[test]
    fn test_apply_overrides() {
        let mut config = Config::default();

        // Apply poll interval
        let override_val = ConfigOverride::parse("poll_interval_ms", "1000").unwrap();
        override_val.apply(&mut config);
        assert_eq!(config.poll_interval_ms, 1000);

        // Apply capture lines
        let override_val = ConfigOverride::parse("capture_lines", "500").unwrap();
        override_val.apply(&mut config);
        assert_eq!(config.capture_lines, 500);

        // Apply show detached sessions
        let override_val = ConfigOverride::parse("showdetached", "false").unwrap();
        override_val.apply(&mut config);
        assert!(!config.show_detached_sessions);

        // Apply debug mode
        let override_val = ConfigOverride::parse("debug_mode", "true").unwrap();
        override_val.apply(&mut config);
        assert!(config.debug_mode);
    }
}
