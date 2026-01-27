use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

use super::config_override::ConfigOverride;
use super::key_binding::KeyBindings;
use super::menu_config::MenuConfig;
use super::session_pattern::SessionPattern;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Polling interval in milliseconds
    #[serde(default)]
    pub poll_interval_ms: u64,

    /// Number of lines to capture from pane
    #[serde(default)]
    pub capture_lines: u32,

    /// Whether to show detached tmux sessions
    #[serde(default)]
    pub show_detached_sessions: bool,

    /// Enable extra logging in the TUI
    #[serde(default)]
    pub debug_mode: bool,

    /// Whether to truncate long lines in preview (default: true)
    #[serde(default)]
    pub truncate_long_lines: bool,

    /// Max line width for truncation (None = use terminal width)
    #[serde(default)]
    pub max_line_width: Option<u16>,

    /// Size of the buffer to capture from pane (default: 16384)
    #[serde(default = "default_buffer_size")]
    pub capture_buffer_size: usize,

    /// Whether navigation in lists is cyclic (default: true)
    #[serde(default = "default_true")]
    pub cyclic_navigation: bool,

    /// Key bindings configuration
    #[serde(default)]
    pub key_bindings: KeyBindings,

    /// Trigger key for popup input dialog (default: "/")
    #[serde(default)]
    pub popup_trigger_key: String,

    /// Sessions to ignore (supports fixed, glob, regex patterns)
    #[serde(default)]
    pub ignore_sessions: Vec<String>,

    /// Auto-ignore the session where tmuxx itself runs (default: true)
    #[serde(default)]
    pub ignore_self: bool,

    /// Hide bottom input buffer (use modal textarea instead)
    #[serde(default)]
    pub hide_bottom_input: bool,

    /// Whether to log all actions to the status bar (default: true)
    #[serde(default)]
    pub log_actions: bool,

    /// Whether to show TODO section at full width (hiding activity panel)
    #[serde(default = "default_true")]
    pub todo_full_width: bool,

    /// Generic agent definitions (Merged from defaults + user config)
    #[serde(default)]
    pub agents: Vec<AgentConfig>,

    /// Default color for agent names in the tree
    #[serde(default)]
    pub agent_name_color: String,

    /// Color for selected item background (cursor)
    #[serde(default)]
    pub current_item_bg_color: String,

    /// Color for multi-selected items background (checked). None = no background change.
    #[serde(default)]
    pub multi_selection_bg_color: Option<String>,

    /// Whether to display TODO from a file instead of parsing pane output
    #[serde(default)]
    pub todo_from_file: bool,

    /// List of file names/patterns to look for TODO content (first found wins)
    #[serde(default)]
    pub todo_files: Vec<String>,

    /// Width of the sidebar (fixed number of characters or percentage like "25%")
    #[serde(default)]
    pub sidebar_width: SidebarWidth,

    /// Tree menu configuration
    #[serde(default)]
    pub menu: MenuConfig,

    /// Prompts menu configuration
    #[serde(default)]
    pub prompts: MenuConfig,

    /// Pane tree configuration
    #[serde(default)]
    pub pane_tree: PaneTreeConfig,

    /// Global status indicators
    #[serde(default)]
    pub indicators: StatusIndicators,

    /// Animation and polling settings
    #[serde(default)]
    pub timing: TimingConfig,

    /// UI message templates
    #[serde(default)]
    pub messages: MessageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingConfig {
    /// Animation tick interval in milliseconds
    #[serde(default = "default_tick_interval")]
    pub tick_interval_ms: u64,
    /// Status hysteresis in milliseconds
    #[serde(default = "default_hysteresis")]
    pub hysteresis_ms: u64,
}

fn default_tick_interval() -> u64 {
    80
}
fn default_hysteresis() -> u64 {
    2000
}

impl Default for TimingConfig {
    fn default() -> Self {
        Self {
            tick_interval_ms: default_tick_interval(),
            hysteresis_ms: default_hysteresis(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageConfig {
    /// Message shown when awaiting approval
    #[serde(default = "default_approval_msg")]
    pub approval_prompt: String,
    /// Welcome message shown in status bar
    #[serde(default = "default_welcome_msg")]
    pub welcome: String,

    /// UI Labels
    #[serde(default = "default_label_todo")]
    pub label_todo: String,
    #[serde(default = "default_label_tasks")]
    pub label_tasks: String,
    #[serde(default = "default_label_tools")]
    pub label_tools: String,

    /// Agent Tree Header Labels
    #[serde(default = "default_label_sel")]
    pub label_sel: String,
    #[serde(default = "default_label_pending")]
    pub label_pending: String,
    #[serde(default = "default_label_subs")]
    pub label_subs: String,
    #[serde(default = "default_label_agents")]
    pub label_agents: String,
}

fn default_approval_msg() -> String {
    "⚠ {agent_type} wants: {approval_type}\n\nDetails: {details}\n\nPress {approve_key} to approve or {reject_key} to reject".to_string()
}
fn default_welcome_msg() -> String {
    "tmuxx v{version} [{color_mode}] - Press ? for help".to_string()
}
fn default_label_todo() -> String {
    "Project TODO:".to_string()
}
fn default_label_tasks() -> String {
    "Tasks:".to_string()
}
fn default_label_tools() -> String {
    "Tools:".to_string()
}
fn default_label_sel() -> String {
    "sel".to_string()
}
fn default_label_pending() -> String {
    "pending".to_string()
}
fn default_label_subs() -> String {
    "subs".to_string()
}
fn default_label_agents() -> String {
    "agents".to_string()
}

impl Default for MessageConfig {
    fn default() -> Self {
        Self {
            approval_prompt: default_approval_msg(),
            welcome: default_welcome_msg(),
            label_todo: default_label_todo(),
            label_tasks: default_label_tasks(),
            label_tools: default_label_tools(),
            label_sel: default_label_sel(),
            label_pending: default_label_pending(),
            label_subs: default_label_subs(),
            label_agents: default_label_agents(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusIndicators {
    #[serde(default = "default_idle_indicator")]
    pub idle: String,
    #[serde(default = "default_processing_indicator")]
    pub processing: String,
    #[serde(default = "default_approval_indicator")]
    pub approval: String,
    #[serde(default = "default_error_indicator")]
    pub error: String,
    #[serde(default = "default_unknown_indicator")]
    pub unknown: String,
    #[serde(default = "default_subagent_running_indicator")]
    pub subagent_running: String,
    #[serde(default = "default_subagent_completed_indicator")]
    pub subagent_completed: String,
    #[serde(default = "default_subagent_failed_indicator")]
    pub subagent_failed: String,
    /// Animation frames for processing status
    #[serde(default = "default_spinner_frames")]
    pub spinner: Vec<String>,
}

fn default_idle_indicator() -> String {
    "●".to_string()
}
fn default_processing_indicator() -> String {
    "◐".to_string()
}
fn default_approval_indicator() -> String {
    "⚠".to_string()
}
fn default_error_indicator() -> String {
    "✗".to_string()
}
fn default_unknown_indicator() -> String {
    "?".to_string()
}
fn default_subagent_running_indicator() -> String {
    "▶".to_string()
}
fn default_subagent_completed_indicator() -> String {
    "✓".to_string()
}
fn default_subagent_failed_indicator() -> String {
    "✗".to_string()
}
fn default_spinner_frames() -> Vec<String> {
    vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]
        .into_iter()
        .map(|s| s.to_string())
        .collect()
}

impl Default for StatusIndicators {
    fn default() -> Self {
        Self {
            idle: default_idle_indicator(),
            processing: default_processing_indicator(),
            approval: default_approval_indicator(),
            error: default_error_indicator(),
            unknown: default_unknown_indicator(),
            subagent_running: default_subagent_running_indicator(),
            subagent_completed: default_subagent_completed_indicator(),
            subagent_failed: default_subagent_failed_indicator(),
            spinner: default_spinner_frames(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_buffer_size() -> usize {
    16384
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct PartialConfig {
    poll_interval_ms: Option<u64>,
    capture_lines: Option<u32>,
    show_detached_sessions: Option<bool>,
    debug_mode: Option<bool>,
    truncate_long_lines: Option<bool>,
    max_line_width: Option<u16>,
    key_bindings: Option<KeyBindings>,
    popup_trigger_key: Option<String>,
    ignore_sessions: Option<Vec<String>>,
    ignore_self: Option<bool>,
    hide_bottom_input: Option<bool>,
    log_actions: Option<bool>,
    todo_full_width: Option<bool>,
    agents: Vec<AgentConfig>,
    agent_name_color: Option<String>,
    current_item_bg_color: Option<String>,
    multi_selection_bg_color: Option<String>,
    todo_from_file: Option<bool>,
    todo_files: Option<Vec<String>>,
    sidebar_width: Option<SidebarWidth>,
    capture_buffer_size: Option<usize>,
    cyclic_navigation: Option<bool>,

    menu: Option<MenuConfig>,
    prompts: Option<MenuConfig>,
    pane_tree: Option<PaneTreeConfig>,
    indicators: Option<StatusIndicators>,
    timing: Option<TimingConfig>,
    messages: Option<MessageConfig>,
}

impl PartialConfig {
    fn apply(self, config: &mut Config) {
        if let Some(v) = self.poll_interval_ms {
            config.poll_interval_ms = v;
        }
        if let Some(v) = self.capture_lines {
            config.capture_lines = v;
        }
        if let Some(v) = self.show_detached_sessions {
            config.show_detached_sessions = v;
        }
        if let Some(v) = self.debug_mode {
            config.debug_mode = v;
        }
        if let Some(v) = self.truncate_long_lines {
            config.truncate_long_lines = v;
        }
        if let Some(v) = self.max_line_width {
            config.max_line_width = Some(v);
        }
        if let Some(v) = self.popup_trigger_key {
            config.popup_trigger_key = v;
        }
        if let Some(v) = self.ignore_sessions {
            config.ignore_sessions = v;
        }
        if let Some(v) = self.ignore_self {
            config.ignore_self = v;
        }
        if let Some(v) = self.hide_bottom_input {
            config.hide_bottom_input = v;
        }
        if let Some(v) = self.log_actions {
            config.log_actions = v;
        }
        if let Some(v) = self.todo_full_width {
            config.todo_full_width = v;
        }
        if let Some(v) = self.agent_name_color {
            config.agent_name_color = v;
        }
        if let Some(v) = self.current_item_bg_color {
            config.current_item_bg_color = v;
        }
        if let Some(v) = self.multi_selection_bg_color {
            config.multi_selection_bg_color = Some(v);
        }
        if let Some(v) = self.todo_from_file {
            config.todo_from_file = v;
        }
        if let Some(v) = self.todo_files {
            config.todo_files = v;
        }
        if let Some(v) = self.sidebar_width {
            config.sidebar_width = v;
        }
        if let Some(v) = self.capture_buffer_size {
            config.capture_buffer_size = v;
        }
        if let Some(v) = self.cyclic_navigation {
            config.cyclic_navigation = v;
        }
        if let Some(mut v) = self.menu {
            if v.merge_with_defaults {
                config.menu.items.append(&mut v.items);
            } else {
                config.menu = v;
            }
        }
        if let Some(mut v) = self.prompts {
            if v.merge_with_defaults {
                config.prompts.items.append(&mut v.items);
            } else {
                config.prompts = v;
            }
        }
        if let Some(v) = self.pane_tree {
            config.pane_tree = v;
        }
        if let Some(v) = self.indicators {
            config.indicators = v;
        }
        if let Some(v) = self.timing {
            config.timing = v;
        }
        if let Some(v) = self.messages {
            config.messages = v;
        }

        if !self.agents.is_empty() {
            // Logic: User agents with same 'id' replace default. New ones append.
            let mut final_agents = Vec::new();
            let mut user_ids: std::collections::HashSet<String> = HashSet::new();

            for agent in &self.agents {
                user_ids.insert(agent.id.clone());
            }

            // Add existing that are NOT overridden
            for agent in config.agents.drain(..) {
                if !user_ids.contains(&agent.id) {
                    final_agents.push(agent);
                }
            }

            // Add user agents (overrides + new)
            final_agents.extend(self.agents);

            // Sort by priority (descending)
            final_agents.sort_by_key(|a| std::cmp::Reverse(a.priority));
            config.agents = final_agents;
        }

        if let Some(user_bindings) = self.key_bindings {
            config.key_bindings.bindings.extend(user_bindings.bindings);
        }
    }
}
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

impl Default for SidebarWidth {
    fn default() -> Self {
        SidebarWidth::Fixed(24)
    }
}

/// Configurable Agent Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

    /// Explicit type for the default status
    #[serde(rename = "default_type", default)]
    pub default_type: Option<RuleType>,

    /// How to detect subagents
    #[serde(default)]
    pub subagent_rules: Option<SubagentRules>,

    /// Configuration for parsing output regions (separating body from footer)
    #[serde(default)]
    pub layout: Option<LayoutConfig>,

    /// Process indicators to show next to agent name
    #[serde(default)]
    pub process_indicators: Vec<ProcessIndicator>,

    /// Rules for generating summary view
    #[serde(default)]
    pub summary_rules: Option<SummaryRules>,

    /// Rules for syntax highlighting in detailed preview
    #[serde(default)]
    pub highlight_rules: Vec<HighlightRule>,

    /// Key bindings
    #[serde(default)]
    pub keys: AgentKeys,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryRules {
    pub activity: Option<String>,
    pub task_pending: Option<String>,
    pub task_completed: Option<String>,
    pub tool_use: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightRule {
    pub pattern: String,
    pub color: String,
    #[serde(default)]
    pub modifiers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    /// Regex identifying the separator for the footer (content after this is ignored)
    pub footer_separator: Option<String>,
    /// Regex identifying the separator for the header (content before this is ignored)
    pub header_separator: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessIndicator {
    pub ancestor_pattern: String, // regex pro ps -o comm=
    pub icon: String,             // emoji/text k zobrazení
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

    #[serde(rename = "content")]
    Content { pattern: String },
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
    /// Explicit categorization of the status
    #[serde(rename = "type")]
    pub kind: Option<RuleType>,
    /// Explicit approval type if status is 'awaiting_approval'
    pub approval_type: Option<String>,
    /// If set, only search within the last N lines
    pub last_lines: Option<usize>,
    /// Refine the status based on capture groups in the pattern
    #[serde(default)]
    pub refinements: Vec<Refinement>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuleType {
    Idle,
    Working,
    Approval,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Refinement {
    pub group: String,
    pub pattern: String,
    pub status: String,
    #[serde(rename = "type")]
    pub kind: Option<RuleType>,
    pub approval_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentRules {
    pub start: String,
    pub running: String,
    pub complete: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneTreeConfig {
    #[serde(default = "default_pane_tree_mode")]
    pub mode: String,

    #[serde(default)]
    pub compact_template: String,

    #[serde(default)]
    pub full_template: String,

    #[serde(default)]
    pub header_template: String,

    #[serde(default = "default_header_fg")]
    pub session_header_fg_color: String,

    #[serde(default)]
    pub session_header_bg_color: Option<String>,
}

fn default_pane_tree_mode() -> String {
    "full".to_string()
}

fn default_header_fg() -> String {
    "cyan".to_string()
}

impl Default for PaneTreeConfig {
    fn default() -> Self {
        Self {
            mode: "full".to_string(),
            compact_template: "  {selection}{window_id}:{window_name} │ {status_char} {name} {status_text}".to_string(),
            full_template: "  {selection}{status_char} {name}\n    {status_text} | pid:{pid} | {uptime}\n    {path} {context}\n{subagents}".to_string(),
            header_template: " ▼ {session}".to_string(),
            session_header_fg_color: "cyan".to_string(),
            session_header_bg_color: Some("darkgray".to_string()),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::load_defaults()
    }
}

impl Config {
    /// Loads configuration, merging embedded defaults with user settings
    pub fn try_load_merged() -> Result<Self> {
        let mut config = Self::load_defaults();

        // 2. Load User Config (if exists)
        if let Some(path) = Self::default_path() {
            if path.exists() {
                let content = std::fs::read_to_string(&path)?;
                let user_partial: PartialConfig = toml::from_str(&content).map_err(|e| {
                    anyhow::anyhow!("Error parsing config file {}: {}", path.display(), e)
                })?;
                user_partial.apply(&mut config);
            }
        }

        // 3. Merge Project Config (if exists)
        if let Some(project_menu) = Self::load_project_menu_config()? {
            config.menu.items.extend(project_menu.items);
        }

        // 4. Merge Project Prompts
        if let Some(project_prompts) = Self::load_project_prompts_config()? {
            config.prompts.items.extend(project_prompts.items);
        }

        // 5. Load prompts from directories
        // User directory: ~/.config/tmuxx/prompts
        if let Some(config_dir) = dirs::config_dir() {
            let user_prompts_dir = config_dir.join("tmuxx").join("prompts");
            if let Some(dir_prompts) = Self::load_prompts_from_dir(&user_prompts_dir)? {
                config.prompts.items.extend(dir_prompts.items);
            }
        }

        // Project directory: ./.tmuxx/prompts
        let project_prompts_dir = PathBuf::from(".tmuxx").join("prompts");
        if let Some(dir_prompts) = Self::load_prompts_from_dir(&project_prompts_dir)? {
            config.prompts.items.extend(dir_prompts.items);
        }

        Ok(config)
    }

    /// Loads configuration, merging embedded defaults with user settings.
    /// Exits the process if the configuration is invalid.
    pub fn load_merged() -> Self {
        Self::try_load_merged().unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(1);
        })
    }

    /// Attempts to load project-specific prompts configuration
    fn load_project_prompts_config() -> Result<Option<MenuConfig>> {
        let project_config_path = PathBuf::from(".tmuxx.toml");
        if project_config_path.exists() {
            let content = std::fs::read_to_string(&project_config_path)?;
            #[derive(Deserialize)]
            struct ProjectConfig {
                prompts: Option<MenuConfig>,
            }
            let proj: ProjectConfig = toml::from_str(&content)
                .map_err(|e| anyhow::anyhow!("Error parsing .tmuxx.toml (prompts): {}", e))?;
            return Ok(proj.prompts);
        }
        Ok(None)
    }

    /// Recursively load prompts from a directory
    fn load_prompts_from_dir(path: &PathBuf) -> Result<Option<MenuConfig>> {
        if !path.exists() || !path.is_dir() {
            return Ok(None);
        }

        let mut items = Vec::new();
        let entries = std::fs::read_dir(path)?;
        let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
        // Sort to ensure consistent order
        entries.sort_by_key(|e| e.path());

        for entry in entries {
            let path = entry.path();
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            if path.is_dir() {
                if let Some(subdir_config) = Self::load_prompts_from_dir(&path)? {
                    if !subdir_config.items.is_empty() {
                        items.push(super::menu_config::MenuItem {
                            name,
                            description: None,
                            execute_command: None,
                            text: None,
                            items: subdir_config.items,
                        });
                    }
                }
            } else if path.is_file() {
                let content = std::fs::read_to_string(&path)?;
                // Only include if content is not empty? Or trim?
                let content = content.trim().to_string();
                if !content.is_empty() {
                    items.push(super::menu_config::MenuItem {
                        name,
                        description: None,
                        execute_command: None,
                        text: Some(content),
                        items: Vec::new(),
                    });
                }
            }
        }

        if items.is_empty() {
            Ok(None)
        } else {
            Ok(Some(MenuConfig {
                items,
                merge_with_defaults: false,
            }))
        }
    }

    /// Attempts to load project-specific configuration (.tmuxx.toml)
    fn load_project_menu_config() -> Result<Option<MenuConfig>> {
        // Look for .tmuxx.toml in current directory (where tmuxx is running)
        // Ideally this should be the session root, but for now current dir is a good proxy if run from root.
        // We can also check specific paths if needed.
        let project_config_path = PathBuf::from(".tmuxx.toml");
        if project_config_path.exists() {
            let content = std::fs::read_to_string(&project_config_path)?;
            #[derive(Deserialize)]
            struct ProjectConfig {
                menu: Option<MenuConfig>,
                menu_items: Option<Vec<super::menu_config::MenuItem>>,
            }

            let proj: ProjectConfig = toml::from_str(&content)
                .map_err(|e| anyhow::anyhow!("Error parsing .tmuxx.toml (menu): {}", e))?;
            if let Some(menu) = proj.menu {
                return Ok(Some(menu));
            }
            if let Some(items) = proj.menu_items {
                return Ok(Some(MenuConfig {
                    items,
                    merge_with_defaults: false,
                }));
            }
        }
        Ok(None)
    }

    /// Loads only the embedded default configuration (ignores user config)
    pub fn load_defaults() -> Self {
        let default_toml = include_str!("../config/defaults.toml");
        toml::from_str(default_toml).unwrap_or_else(|e| {
            eprintln!("Internal Error: Failed to parse default config: {}", e);
            // Absolute fallback if parsing fails completely
            std::process::exit(1);
        })
    }

    /// Returns the default config file path
    pub fn default_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("tmuxx").join("config.toml"))
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
    /// * `current_session` - The session where tmuxx is running (for ignore_self)
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
        assert!(config.should_ignore_session("tmuxx", Some("tmuxx"))); // ignore_self
        assert!(config.should_ignore_session("test-1", Some("tmuxx"))); // pattern
        assert!(!config.should_ignore_session("dev", Some("tmuxx"))); // neither
    }

    #[test]
    fn test_try_load_merged_invalid_toml() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, "invalid = toml = format").unwrap();

        // We need to mock default_path() or use load_from for testing the parsing error
        let result = Config::load_from(&config_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid"));
    }
}
