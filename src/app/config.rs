use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

use super::config_override::ConfigOverride;
use super::key_binding::KeyBindings;
use super::session_pattern::SessionPattern;
use ratatui::layout::Constraint;

/// Represents the width of the sidebar (either fixed length or percentage)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum SidebarWidth {
    Fixed(u16),
    Percent(String),
}

impl SidebarWidth {
    pub fn to_constraint(&self) -> Constraint {
        match self {
            SidebarWidth::Fixed(w) => Constraint::Length(*w),
            SidebarWidth::Percent(s) => {
                if let Some(p) = s.strip_suffix('%').and_then(|p| p.parse::<u16>().ok()) {
                    Constraint::Percentage(p.min(100))
                } else {
                    // Fallback for invalid string
                    Constraint::Percentage(35)
                }
            }
        }
    }

    pub fn wider(&mut self) {
        match self {
            SidebarWidth::Fixed(w) => *w = (*w + 2).min(150), // Max 150 chars
            SidebarWidth::Percent(s) => {
                if let Some(p) = s.strip_suffix('%').and_then(|p| p.parse::<u16>().ok()) {
                    let next_p = (p + 5).min(90); // Max 90%
                    *s = format!("{}%", next_p);
                }
            }
        }
    }

    pub fn narrower(&mut self) {
        match self {
            SidebarWidth::Fixed(w) => *w = w.saturating_sub(2).max(5),
            SidebarWidth::Percent(s) => {
                if let Some(p) = s.strip_suffix('%').and_then(|p| p.parse::<u16>().ok()) {
                    let next_p = p.saturating_sub(5).max(5); // Min 5%
                    *s = format!("{}%", next_p);
                }
            }
        }
    }
}

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

    /// Key bindings configuration
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

    /// Generic agent definitions (Merged from defaults + user config)
    #[serde(default)]
    pub agents: Vec<AgentConfig>,

    /// Default color for agent names in the tree
    #[serde(default = "default_agent_name_color")]
    pub agent_name_color: String,

    /// Color for selected item background (cursor)
    #[serde(default = "default_current_item_bg_color")]
    pub current_item_bg_color: String,

    /// Color for multi-selected items background (checked). None = no background change.
    #[serde(default)]
    pub multi_selection_bg_color: Option<String>,

    /// Whether to display TODO from a file instead of parsing pane output
    #[serde(default = "default_todo_from_file")]
    pub todo_from_file: bool,

    /// List of file names/patterns to look for TODO content (first found wins)
    #[serde(default = "default_todo_files")]
    pub todo_files: Vec<String>,

    /// Width of the sidebar (fixed number of characters or percentage like "25%")
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: SidebarWidth,
}

fn default_todo_from_file() -> bool {
    true
}

fn default_todo_files() -> Vec<String> {
    vec![
        "TODO.md".to_string(),
        "NOTES.md".to_string(),
        "TASKS.md".to_string(),
        "README.md".to_string(),
    ]
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

fn default_agent_name_color() -> String {
    "#000000".to_string()
}

fn default_current_item_bg_color() -> String {
    "#4a4a4a".to_string()
}

fn default_sidebar_width() -> SidebarWidth {
    SidebarWidth::Fixed(24)
}

/// Configurable Agent Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Unique ID for merging/overriding (e.g., "claude")
    pub id: String,

    /// Display Name
    pub name: String,

    /// Agent color theme (e.g. "magenta", "blue", "green")
    #[serde(default)]
    pub color: Option<String>,

    /// Agent background color (e.g. "black", "red")
    #[serde(default)]
    pub background_color: Option<String>,

    /// Priority (higher wins)
    #[serde(default)]
    pub priority: u32,

    /// How to detect this agent
    #[serde(default)]
    pub matchers: Vec<MatcherConfig>,

    /// How to detect state
    #[serde(default)]
    pub state_rules: Vec<StateRule>,

    /// Specific patterns in title that indicate 'Processing'
    #[serde(default)]
    pub title_indicators: Option<Vec<String>>,

    /// Status to return if no rules match (default: "processing")
    #[serde(default)]
    pub default_status: Option<String>,

    /// How to detect subagents
    #[serde(default)]
    pub subagent_rules: Option<SubagentRules>,

    /// Configuration for parsing output regions (separating body from footer)
    #[serde(default)]
    pub layout: Option<LayoutConfig>,

    /// Key bindings
    #[serde(default)]
    pub keys: AgentKeys,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    /// Regex identifying the separator for the footer (content after this is ignored)
    pub footer_separator: Option<String>,
    /// Regex identifying the separator for the header (content before this is ignored)
    pub header_separator: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MatcherConfig {
    #[serde(rename = "command")]
    Command { pattern: String },

    #[serde(rename = "ancestor")]
    Ancestor { pattern: String },

    #[serde(rename = "title")]
    Title { pattern: String },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentKeys {
    pub approve: Option<String>,
    pub reject: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateRule {
    pub status: String,
    pub pattern: String,
    /// Explicit approval type if status is 'awaiting_approval'
    pub approval_type: Option<String>,
    /// If set, only search within the last N lines
    pub last_lines: Option<usize>,
    /// Refine the status based on capture groups in the pattern
    #[serde(default)]
    pub refinements: Vec<Refinement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Refinement {
    pub group: String,
    pub pattern: String,
    pub status: String,
    pub approval_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentRules {
    pub start: String,
    pub running: String,
    pub complete: String,
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

            key_bindings: KeyBindings::default(),
            popup_trigger_key: default_popup_trigger_key(),
            ignore_sessions: Vec::new(),
            ignore_self: default_ignore_self(),
            hide_bottom_input: default_hide_bottom_input(),
            log_actions: default_log_actions(),
            agents: Vec::new(),
            agent_name_color: default_agent_name_color(),
            current_item_bg_color: default_current_item_bg_color(),
            multi_selection_bg_color: None,
            todo_from_file: default_todo_from_file(),
            todo_files: default_todo_files(),
            sidebar_width: default_sidebar_width(),
        }
    }
}

impl Config {
    /// Loads configuration, merging embedded defaults with user settings
    pub fn load_merged() -> Self {
        // 1. Load Defaults
        let default_toml = include_str!("../config/defaults.toml");
        #[derive(Deserialize)]
        struct Defaults {
            #[serde(default)]
            agents: Vec<AgentConfig>,
            #[serde(default)]
            key_bindings: Option<KeyBindings>,
            poll_interval_ms: Option<u64>,
            capture_lines: Option<u32>,
            show_detached_sessions: Option<bool>,
            debug_mode: Option<bool>,
            truncate_long_lines: Option<bool>,
            max_line_width: Option<Option<u16>>,
            popup_trigger_key: Option<String>,
            ignore_sessions: Option<Vec<String>>,
            ignore_self: Option<bool>,
            hide_bottom_input: Option<bool>,
            log_actions: Option<bool>,
            agent_name_color: Option<String>,
            current_item_bg_color: Option<String>,
            multi_selection_bg_color: Option<Option<String>>,
            todo_from_file: Option<bool>,
            todo_files: Option<Vec<String>>,
            sidebar_width: Option<SidebarWidth>,
        }
        let defaults: Defaults = toml::from_str(default_toml).unwrap_or_else(|e| {
            eprintln!("Internal Error: Failed to parse default config: {}", e);
            Defaults {
                agents: Vec::new(),
                key_bindings: None,
                poll_interval_ms: None,
                capture_lines: None,
                show_detached_sessions: None,
                debug_mode: None,
                truncate_long_lines: None,
                max_line_width: None,
                popup_trigger_key: None,
                ignore_sessions: None,
                ignore_self: None,
                hide_bottom_input: None,
                log_actions: None,
                agent_name_color: None,
                current_item_bg_color: None,
                multi_selection_bg_color: None,
                todo_from_file: None,
                todo_files: None,
                sidebar_width: None,
            }
        });

        // Start with hardcoded defaults, then apply values from embedded defaults.toml
        let mut base_config = Config::default();
        if let Some(v) = defaults.poll_interval_ms {
            base_config.poll_interval_ms = v;
        }
        if let Some(v) = defaults.capture_lines {
            base_config.capture_lines = v;
        }
        if let Some(v) = defaults.show_detached_sessions {
            base_config.show_detached_sessions = v;
        }
        if let Some(v) = defaults.debug_mode {
            base_config.debug_mode = v;
        }
        if let Some(v) = defaults.truncate_long_lines {
            base_config.truncate_long_lines = v;
        }
        if let Some(v) = defaults.max_line_width {
            base_config.max_line_width = v;
        }
        if let Some(v) = defaults.popup_trigger_key {
            base_config.popup_trigger_key = v;
        }
        if let Some(v) = defaults.ignore_sessions {
            base_config.ignore_sessions = v;
        }
        if let Some(v) = defaults.ignore_self {
            base_config.ignore_self = v;
        }
        if let Some(v) = defaults.hide_bottom_input {
            base_config.hide_bottom_input = v;
        }
        if let Some(v) = defaults.log_actions {
            base_config.log_actions = v;
        }
        if let Some(v) = defaults.agent_name_color {
            base_config.agent_name_color = v;
        }
        if let Some(v) = defaults.current_item_bg_color {
            base_config.current_item_bg_color = v;
        }
        if let Some(v) = defaults.multi_selection_bg_color {
            base_config.multi_selection_bg_color = v;
        }
        if let Some(v) = defaults.todo_from_file {
            base_config.todo_from_file = v;
        }
        if let Some(v) = defaults.todo_files {
            base_config.todo_files = v;
        }
        if let Some(v) = defaults.sidebar_width {
            base_config.sidebar_width = v;
        }

        // 2. Load User Config (if exists)
        // We load into PartialConfig first to identify which fields are actually present
        // in the user config file. This prevents missing fields from overwriting
        // defaults.toml values with hardcoded defaults.
        let mut user_config_agents = Vec::new();
        let mut user_config_bindings = None;

        let mut config = base_config.clone();

        if let Some(path) = Self::default_path() {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(user_partial) = toml::from_str::<Defaults>(&content) {
                        // Merge scalar fields
                        if let Some(v) = user_partial.poll_interval_ms {
                            config.poll_interval_ms = v;
                        }
                        if let Some(v) = user_partial.capture_lines {
                            config.capture_lines = v;
                        }
                        if let Some(v) = user_partial.show_detached_sessions {
                            config.show_detached_sessions = v;
                        }
                        if let Some(v) = user_partial.debug_mode {
                            config.debug_mode = v;
                        }
                        if let Some(v) = user_partial.truncate_long_lines {
                            config.truncate_long_lines = v;
                        }
                        if let Some(v) = user_partial.max_line_width {
                            config.max_line_width = v;
                        }
                        if let Some(v) = user_partial.popup_trigger_key {
                            config.popup_trigger_key = v;
                        }
                        if let Some(v) = user_partial.ignore_sessions {
                            config.ignore_sessions = v;
                        }
                        if let Some(v) = user_partial.ignore_self {
                            config.ignore_self = v;
                        }
                        if let Some(v) = user_partial.hide_bottom_input {
                            config.hide_bottom_input = v;
                        }
                        if let Some(v) = user_partial.log_actions {
                            config.log_actions = v;
                        }
                        if let Some(v) = user_partial.agent_name_color {
                            config.agent_name_color = v;
                        }
                        if let Some(v) = user_partial.current_item_bg_color {
                            config.current_item_bg_color = v;
                        }
                        if let Some(v) = user_partial.multi_selection_bg_color {
                            config.multi_selection_bg_color = v;
                        }
                        if let Some(v) = user_partial.todo_from_file {
                            config.todo_from_file = v;
                        }
                        if let Some(v) = user_partial.todo_files {
                            config.todo_files = v;
                        }
                        if let Some(v) = user_partial.sidebar_width {
                            config.sidebar_width = v;
                        }

                        // Save complex fields for merging logic below
                        user_config_agents = user_partial.agents;
                        user_config_bindings = user_partial.key_bindings;
                    } else {
                        eprintln!("Warning: Failed to parse user config file. Using defaults.");
                    }
                }
            }
        }

        // 2a. Merge Key Bindings
        // Start with default bindings (from defaults.toml)
        let mut final_bindings = if let Some(default_bindings) = defaults.key_bindings {
            default_bindings.bindings
        } else {
            KeyBindings::default().bindings
        };

        // Override with user bindings
        if let Some(user_bindings) = user_config_bindings {
            final_bindings.extend(user_bindings.bindings);
        }
        config.key_bindings.bindings = final_bindings;

        // 3. Merge Agents
        // Logic: User agents with same 'id' replace default. New ones append.
        let mut final_agents = Vec::new();
        let mut user_ids: std::collections::HashSet<String> = HashSet::new();

        // Index user agents
        for agent in &user_config_agents {
            user_ids.insert(agent.id.clone());
        }

        // Add defaults that are NOT overridden
        for agent in defaults.agents {
            if !user_ids.contains(&agent.id) {
                final_agents.push(agent);
            }
        }

        // Add user agents (overrides + new)
        final_agents.extend(user_config_agents);

        // Sort by priority (descending)
        final_agents.sort_by_key(|a| std::cmp::Reverse(a.priority));

        config.agents = final_agents;
        config
    }

    /// Loads only the embedded default configuration (ignores user config)
    pub fn load_defaults() -> Self {
        let default_toml = include_str!("../config/defaults.toml");
        #[derive(Deserialize)]
        struct Defaults {
            #[serde(default)]
            agents: Vec<AgentConfig>,
            #[serde(default)]
            key_bindings: Option<KeyBindings>,
            poll_interval_ms: Option<u64>,
            capture_lines: Option<u32>,
            show_detached_sessions: Option<bool>,
            debug_mode: Option<bool>,
            truncate_long_lines: Option<bool>,
            max_line_width: Option<Option<u16>>,
            popup_trigger_key: Option<String>,
            ignore_sessions: Option<Vec<String>>,
            ignore_self: Option<bool>,
            hide_bottom_input: Option<bool>,
            log_actions: Option<bool>,
            agent_name_color: Option<String>,
            current_item_bg_color: Option<String>,
            multi_selection_bg_color: Option<Option<String>>,
            todo_from_file: Option<bool>,
            todo_files: Option<Vec<String>>,
            sidebar_width: Option<SidebarWidth>,
        }
        let defaults: Defaults = toml::from_str(default_toml).unwrap_or_else(|e| {
            eprintln!("Internal Error: Failed to parse default config: {}", e);
            Defaults {
                agents: Vec::new(),
                key_bindings: None,
                poll_interval_ms: None,
                capture_lines: None,
                show_detached_sessions: None,
                debug_mode: None,
                truncate_long_lines: None,
                max_line_width: None,
                popup_trigger_key: None,
                ignore_sessions: None,
                ignore_self: None,
                hide_bottom_input: None,
                log_actions: None,
                agent_name_color: None,
                current_item_bg_color: None,
                multi_selection_bg_color: None,
                todo_from_file: None,
                todo_files: None,
                sidebar_width: None,
            }
        });

        let mut config = Config {
            agents: defaults.agents,
            ..Default::default()
        };
        if let Some(kb) = defaults.key_bindings {
            config.key_bindings = kb;
        }
        if let Some(v) = defaults.poll_interval_ms {
            config.poll_interval_ms = v;
        }
        if let Some(v) = defaults.capture_lines {
            config.capture_lines = v;
        }
        if let Some(v) = defaults.show_detached_sessions {
            config.show_detached_sessions = v;
        }
        if let Some(v) = defaults.debug_mode {
            config.debug_mode = v;
        }
        if let Some(v) = defaults.truncate_long_lines {
            config.truncate_long_lines = v;
        }
        if let Some(v) = defaults.max_line_width {
            config.max_line_width = v;
        }
        if let Some(v) = defaults.popup_trigger_key {
            config.popup_trigger_key = v;
        }
        if let Some(v) = defaults.ignore_sessions {
            config.ignore_sessions = v;
        }
        if let Some(v) = defaults.ignore_self {
            config.ignore_self = v;
        }
        if let Some(v) = defaults.hide_bottom_input {
            config.hide_bottom_input = v;
        }
        if let Some(v) = defaults.log_actions {
            config.log_actions = v;
        }
        if let Some(v) = defaults.agent_name_color {
            config.agent_name_color = v;
        }
        if let Some(v) = defaults.current_item_bg_color {
            config.current_item_bg_color = v;
        }
        if let Some(v) = defaults.multi_selection_bg_color {
            config.multi_selection_bg_color = v;
        }
        if let Some(v) = defaults.todo_from_file {
            config.todo_from_file = v;
        }
        if let Some(v) = defaults.todo_files {
            config.todo_files = v;
        }
        if let Some(v) = defaults.sidebar_width {
            config.sidebar_width = v;
        }

        config
    }

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
        let config = Config::load_defaults();
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
