use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::config_override::ConfigOverride;
use super::key_binding::KeyBindings;
use super::session_pattern::SessionPattern;

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

    /// Key bindings configuration
    #[serde(default)]
    pub key_bindings: KeyBindings,

    /// Trigger key for popup input dialog (default: "/")
    #[serde(default = "default_popup_trigger_key")]
    pub popup_trigger_key: String,

    /// Sessions to ignore (supports fixed, glob, regex patterns)
    /// - Fixed: "session-name" (exact match)
    /// - Glob: "test-*" (shell wildcards)
    /// - Regex: "/^ssh-\\d+$/" (wrapped in slashes)
    #[serde(default)]
    pub ignore_sessions: Vec<String>,

    /// Auto-ignore the session where tmuxcc itself runs (default: true)
    #[serde(default = "default_ignore_self")]
    pub ignore_self: bool,

    /// Hide bottom input buffer (use modal textarea instead)
    #[serde(default = "default_hide_bottom_input")]
    pub hide_bottom_input: bool,

    /// Whether to log all actions to the status bar (default: true)
    #[serde(default = "default_log_actions")]
    pub log_actions: bool,

    /// Generic agent definitions
    #[serde(default)]
    pub agent_definitions: Vec<AgentDefinition>,
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

fn default_popup_trigger_key() -> String {
    "/".to_string()
}

fn default_ignore_self() -> bool {
    true
}

fn default_hide_bottom_input() -> bool {
    true
}

fn default_log_actions() -> bool {
    true
}



/// Legacy pattern for detecting agent types (deprecated in favor of AgentDefinition)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPattern {
    /// Command pattern to match (regex)
    pub pattern: String,
    /// Name of the agent type
    pub agent_type: String,
}

/// Generic agent definition for flexible configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    /// Name of the agent (e.g. "Claude Code")
    pub name: String,
    
    /// Regex patterns to match command/process lines
    pub match_patterns: Vec<String>,
    
    /// Priority (higher wins)
    #[serde(default)]
    pub priority: u32,
    
    /// State detection rules
    #[serde(default)]
    pub state_rules: Vec<StateRule>,

    /// Approval keys
    #[serde(default)]
    pub approval_keys: Option<String>, 

     /// Rejection keys 
    #[serde(default)]
    pub rejection_keys: Option<String>,
}

/// Rule for detecting agent state from output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateRule {
    /// The target state (e.g., "processing", "awaiting_input", "error")
    pub state: String,
    
    /// Pattern to match in the output
    pub pattern: String,
    
    /// Search mode: "contains", "regex", "ends_with"
    #[serde(default)]
    pub mode: MatchMode,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MatchMode {
    Regex,
    Contains,
    EndsWith,
}

impl Default for MatchMode {
    fn default() -> Self {
        MatchMode::Contains
    }
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
            key_bindings: KeyBindings::default(),
            popup_trigger_key: default_popup_trigger_key(),
            ignore_sessions: Vec::new(),
            ignore_self: default_ignore_self(),
            hide_bottom_input: default_hide_bottom_input(),
            log_actions: default_log_actions(),
            agent_definitions: vec![
                AgentDefinition {
                    name: "Generic Shell".to_string(),
                    match_patterns: vec![
                        // Match common shells (exact match or ending with shell name)
                        r"^(?:.*/)?(bash|zsh|fish|sh|ksh)$".to_string(),
                    ],
                    priority: 0, // Low priority, fallback
                    state_rules: vec![
                        StateRule {
                            state: "awaiting_input".to_string(),
                            // Matches typical prompts: "user@host:path$ ", "sh-5.1$ ", "> "
                            // Regex breakdown:
                            // ^ - start of line (optional, implicit in some modes but good to be explicit if using regex mode)
                            // .* - any characters (user, host, path)
                            // [>$#%] - common prompt characters
                            // \s* - optional trailing whitespace
                            // $ - end of string
                            pattern: r".*[>$#%]\s*$".to_string(),
                            mode: MatchMode::Regex,
                        },
                    ],
                    approval_keys: None,
                    rejection_keys: None,
                }
            ],
        }
    }
}

impl Config {
    /// Returns the default config file path
    pub fn default_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("tmuxcc").join("config.toml"))
    }

    /// Loads config from the default path or returns defaults
    ///
    /// # Panics
    /// Panics if config file exists but contains invalid TOML or unknown fields.
    /// This ensures users get immediate feedback on configuration errors.
    pub fn load() -> Self {
        if let Some(path) = Self::default_path() {
            if path.exists() {
                return Self::load_from(&path).unwrap_or_else(|e| {
                    eprintln!("Error loading config from {}: {}", path.display(), e);
                    eprintln!("\nHint: Check if all key bindings use valid format:");
                    eprintln!("  - execute_command = {{ command = \"...\" }}");
                    eprintln!("  - kill_app = {{ method = \"sigterm\" }}");
                    eprintln!("  - send_keys = \"...\"");
                    eprintln!("  - navigate: next_agent or prev_agent");
                    std::process::exit(1);
                });
            }
        }
        Self::default()
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

    /// Check if a session should be ignored based on configuration.
    ///
    /// A session is ignored if:
    /// 1. `ignore_self` is true AND session matches current_session
    /// 2. Session matches any pattern in `ignore_sessions`
    ///
    /// # Arguments
    /// * `session` - The session name to check
    /// * `current_session` - The session where tmuxcc is running (for ignore_self)
    pub fn should_ignore_session(&self, session: &str, current_session: Option<&str>) -> bool {
        // Check ignore_self
        if self.ignore_self {
            if let Some(current) = current_session {
                if session == current {
                    return true;
                }
            }
        }

        // Check ignore_sessions patterns
        for pattern_str in &self.ignore_sessions {
            match SessionPattern::parse(pattern_str) {
                Ok(pattern) => {
                    if pattern.matches(session) {
                        return true;
                    }
                }
                Err(e) => {
                    // Log warning but continue (invalid patterns are skipped)
                    tracing::warn!("Invalid ignore_sessions pattern '{}': {}", pattern_str, e);
                }
            }
        }

        false
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
        assert!(config.log_actions);
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

        // Test log_actions override
        config.apply_override("log_actions", "false").unwrap();
        assert!(!config.log_actions);
        config.apply_override("log", "1").unwrap();
        assert!(config.log_actions);

        // Test invalid key
        assert!(config.apply_override("invalid_key", "value").is_err());

        // Test invalid value
        assert!(config
            .apply_override("show_detached_sessions", "invalid")
            .is_err());
    }

    #[test]
    fn test_key_bindings_included() {
        let config = Config::default();
        // Verify key_bindings field exists and has defaults
        assert!(config.key_bindings.get_action("y").is_some());
        assert!(config.key_bindings.get_action("n").is_some());
    }

    #[test]
    fn test_should_ignore_session_default() {
        let config = Config::default();

        // Default: ignore_self=true, empty ignore_sessions
        // Should ignore own session
        assert!(config.should_ignore_session("my-session", Some("my-session")));
        // Should NOT ignore other sessions
        assert!(!config.should_ignore_session("other", Some("my-session")));
        // When not inside tmux (current_session=None), nothing is ignored
        assert!(!config.should_ignore_session("my-session", None));
    }

    #[test]
    fn test_should_ignore_session_patterns() {
        let mut config = Config::default();
        config.ignore_self = false; // Disable to test patterns only
        config.ignore_sessions = vec![
            "prod-*".to_string(),       // glob
            "/^vpn-\\d+$/".to_string(), // regex
            "ssh-tunnel".to_string(),   // fixed
        ];

        // Fixed match
        assert!(config.should_ignore_session("ssh-tunnel", None));
        assert!(!config.should_ignore_session("ssh-tunnel-2", None));

        // Glob match
        assert!(config.should_ignore_session("prod-main", None));
        assert!(config.should_ignore_session("prod-backup", None));
        assert!(!config.should_ignore_session("dev-prod", None));

        // Regex match
        assert!(config.should_ignore_session("vpn-123", None));
        assert!(!config.should_ignore_session("vpn-abc", None));
        assert!(!config.should_ignore_session("my-vpn-1", None));

        // Non-matching
        assert!(!config.should_ignore_session("dev-session", None));
    }

    #[test]
    fn test_should_ignore_session_combined() {
        let mut config = Config::default();
        config.ignore_self = true;
        config.ignore_sessions = vec!["test-*".to_string()];

        // Both ignore_self and patterns work together
        assert!(config.should_ignore_session("tmuxcc", Some("tmuxcc"))); // ignore_self
        assert!(config.should_ignore_session("test-1", Some("tmuxcc"))); // pattern
        assert!(!config.should_ignore_session("dev", Some("tmuxcc"))); // neither
    }

    #[test]
    fn test_ignore_sessions_override() {
        let mut config = Config::default();

        // Test ignore_sessions override
        config
            .apply_override("ignore_sessions", "prod-*,ssh-tunnel")
            .unwrap();
        assert_eq!(config.ignore_sessions.len(), 2);
        assert_eq!(config.ignore_sessions[0], "prod-*");
        assert_eq!(config.ignore_sessions[1], "ssh-tunnel");

        // Test ignore_self override
        config.apply_override("ignore_self", "false").unwrap();
        assert!(!config.ignore_self);
        config.apply_override("ignore_self", "true").unwrap();
        assert!(config.ignore_self);
    }
}
