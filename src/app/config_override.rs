use anyhow::{anyhow, Result};

use super::Config;

/// Represents a configuration override from CLI
#[derive(Debug, Clone)]
pub enum ConfigOverride {
    PollInterval(u64),
    CaptureLines(u32),
    ShowDetachedSessions(bool),
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
            _ => Err(anyhow!(
                "Unknown config key: '{}'. Valid keys: poll_interval_ms, capture_lines, show_detached_sessions",
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
        }
    }
}

/// Normalize a config key: remove underscores, hyphens, convert to lowercase
fn normalize_key(key: &str) -> String {
    key.replace(['_', '-'], "")
        .to_lowercase()
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
        assert_eq!(normalize_key("show_detached_sessions"), "showdetachedsessions");
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
        assert!(result.unwrap_err().to_string().contains("Unknown config key"));
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
    }
}
