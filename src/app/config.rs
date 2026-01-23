use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::config_override::ConfigOverride;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Polling interval in milliseconds
    #[serde(default = "default_poll_interval")]
    pub poll_interval_ms: u64,

    /// Number of lines to capture from pane
    #[serde(default = "default_capture_lines")]
    pub capture_lines: u32,

    /// Whether to show detached tmux sessions
    #[serde(default = "default_show_detached_sessions")]
    pub show_detached_sessions: bool,

    /// Enable extra logging in the TUI
    #[serde(default = "default_debug_mode")]
    pub debug_mode: bool,

    /// Whether to truncate long lines in preview (default: true)
    #[serde(default = "default_truncate_long_lines")]
    pub truncate_long_lines: bool,

    /// Max line width for truncation (None = use terminal width)
    #[serde(default)]
    pub max_line_width: Option<u16>,

    /// Custom agent patterns (command -> agent type mapping)
    #[serde(default)]
    pub agent_patterns: Vec<AgentPattern>,
}

fn default_poll_interval() -> u64 {
    500
}

fn default_capture_lines() -> u32 {
    200
}

fn default_show_detached_sessions() -> bool {
    true
}

fn default_debug_mode() -> bool {
    false
}

fn default_truncate_long_lines() -> bool {
    true
}

/// Pattern for detecting agent types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPattern {
    /// Command pattern to match (regex)
    pub pattern: String,
    /// Name of the agent type
    pub agent_type: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            poll_interval_ms: default_poll_interval(),
            capture_lines: default_capture_lines(),
            show_detached_sessions: default_show_detached_sessions(),
            debug_mode: default_debug_mode(),
            truncate_long_lines: default_truncate_long_lines(),
            max_line_width: None,
            agent_patterns: Vec::new(),
        }
    }
}

impl Config {
    /// Returns the default config file path
    pub fn default_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("tmuxcc").join("config.toml"))
    }

    /// Loads config from the default path or returns defaults
    pub fn load() -> Self {
        Self::default_path()
            .and_then(|path| {
                if path.exists() {
                    Self::load_from(&path).ok()
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }

    /// Loads config from a specific path
    pub fn load_from(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Saves config to the default path
    pub fn save(&self) -> Result<()> {
        if let Some(path) = Self::default_path() {
            self.save_to(&path)?;
        }
        Ok(())
    }

    /// Saves config to a specific path
    pub fn save_to(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Apply a configuration override
    pub fn apply_override(&mut self, key: &str, value: &str) -> Result<()> {
        let override_val = ConfigOverride::parse(key, value)?;
        override_val.apply(self);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.poll_interval_ms, 500);
        assert_eq!(config.capture_lines, 200);
        assert!(config.show_detached_sessions);
        assert!(!config.debug_mode);
        assert!(config.truncate_long_lines);
        assert_eq!(config.max_line_width, None);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.poll_interval_ms, parsed.poll_interval_ms);
        assert_eq!(config.show_detached_sessions, parsed.show_detached_sessions);
    }

    #[test]
    fn test_apply_override() {
        let mut config = Config::default();

        // Test show_detached_sessions override
        config
            .apply_override("show_detached_sessions", "false")
            .unwrap();
        assert!(!config.show_detached_sessions);

        // Test short alias
        config.apply_override("showdetached", "1").unwrap();
        assert!(config.show_detached_sessions);

        // Test poll_interval override
        config.apply_override("poll_interval_ms", "1000").unwrap();
        assert_eq!(config.poll_interval_ms, 1000);

        // Test debug_mode override
        config.apply_override("debug_mode", "true").unwrap();
        assert!(config.debug_mode);

        // Test debug_mode override with short alias
        config.apply_override("debug", "false").unwrap();
        assert!(!config.debug_mode);

        // Test invalid key
        assert!(config.apply_override("invalid_key", "value").is_err());

        // Test invalid value
        assert!(config
            .apply_override("show_detached_sessions", "invalid")
            .is_err());
    }
}
