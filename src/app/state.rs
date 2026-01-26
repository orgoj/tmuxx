use crate::agents::MonitoredAgent;
use crate::monitor::SystemStats;
use crate::ui::components::{MenuTreeState, ModalTextareaState};
// use ratatui::style::{Color, Style};
use std::collections::HashSet;
use std::time::Instant;

use super::config::SidebarWidth;
use super::Config;

/// Which panel is currently focused
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusedPanel {
    /// Agent list sidebar is focused
    #[default]
    Sidebar,
    /// Input area is focused
    Input,
}

/// Type of popup dialog
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum PopupType {
    /// Filter agents/sessions
    #[default]
    Filter,
    /// General text input to send to agent
    GeneralInput,
    /// Rename a tmux session
    RenameSession {
        /// The current session name to rename
        session: String,
    },
    /// Capture test case with expected status
    CaptureStatus {
        /// The content captured at the moment of keypress
        content: String,
    },
}

/// State for popup input dialog
#[derive(Debug, Clone)]
pub struct PopupInputState {
    /// Dialog title
    pub title: String,
    /// Prompt text to display
    pub prompt: String,
    /// Input buffer
    pub buffer: String,
    /// Cursor position (byte offset)
    pub cursor: usize,
    /// Type of popup dialog
    pub popup_type: PopupType,
}

/// Tree structure containing all monitored agents
#[derive(Debug, Clone, Default)]
pub struct AgentTree {
    /// Root agents (directly in tmux panes)
    pub root_agents: Vec<MonitoredAgent>,
}

impl AgentTree {
    /// Creates an empty agent tree
    pub fn new() -> Self {
        Self {
            root_agents: Vec::new(),
        }
    }

    /// Returns the total number of agents (including subagents)
    pub fn total_count(&self) -> usize {
        self.root_agents.iter().map(|a| 1 + a.subagents.len()).sum()
    }

    /// Returns the number of active agents (those needing attention)
    pub fn active_count(&self) -> usize {
        self.root_agents
            .iter()
            .filter(|a| a.status.needs_attention())
            .count()
    }

    /// Returns the total number of running subagents
    pub fn running_subagent_count(&self) -> usize {
        use crate::agents::SubagentStatus;
        self.root_agents
            .iter()
            .flat_map(|a| &a.subagents)
            .filter(|s| matches!(s.status, SubagentStatus::Running))
            .count()
    }

    /// Returns the number of processing agents
    pub fn processing_count(&self) -> usize {
        use crate::agents::AgentStatus;
        self.root_agents
            .iter()
            .filter(|a| matches!(a.status, AgentStatus::Processing { .. }))
            .count()
    }

    /// Gets an agent by index (for selection)
    pub fn get_agent(&self, index: usize) -> Option<&MonitoredAgent> {
        self.root_agents.get(index)
    }

    /// Gets a mutable agent by index
    pub fn get_agent_mut(&mut self, index: usize) -> Option<&mut MonitoredAgent> {
        self.root_agents.get_mut(index)
    }
}

/// Spinner frames for animation
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Main application state
#[derive(Debug)]
pub struct AppState {
    /// Application configuration
    pub config: Config,
    /// Tree of monitored agents
    pub agents: AgentTree,
    /// Currently selected agent index (cursor position)
    pub selected_index: usize,
    /// Multi-selected agent indices
    pub selected_agents: HashSet<usize>,
    /// Which panel is focused
    pub focused_panel: FocusedPanel,
    /// Input buffer (always available)
    pub input_buffer: String,
    /// Cursor position within input buffer (byte offset)
    pub cursor_position: usize,
    /// Whether help is being shown
    pub show_help: bool,
    /// Popup input dialog state (None = not shown)
    pub popup_input: Option<PopupInputState>,
    /// Modal textarea dialog state (None = not shown)
    pub modal_textarea: Option<ModalTextareaState>,
    /// Current filter pattern (None = no filter, Some("") = show all, Some("text") = filter)
    pub filter_pattern: Option<String>,
    /// Whether subagent log is shown
    pub show_subagent_log: bool,
    /// Whether summary detail (TODOs and Tools) is shown
    pub show_summary_detail: bool,
    /// Whether the application should quit
    pub should_quit: bool,
    /// Last error message (if any)
    pub last_error: Option<String>,
    /// Sidebar width (fixed or percentage)
    pub sidebar_width: SidebarWidth,
    /// Animation tick counter
    pub tick: usize,
    /// Last tick time for animation throttling
    last_tick: Instant,
    /// System resource statistics
    pub system_stats: SystemStats,
    /// Whether the terminal likely supports TrueColor
    pub truecolor_supported: bool,
    /// Content of the project TODO file for the currently selected agent
    pub current_todo: Option<String>,
    /// Whether the command menu is shown
    /// Whether the command menu is shown
    pub show_menu: bool,
    /// State for the command menu widget
    pub menu_tree: MenuTreeState,
    /// Whether the prompts menu is shown
    pub show_prompts: bool,
    /// State for the prompts menu widget
    pub prompts_tree: MenuTreeState,
    /// Filter: Show only active (non-idle) agents
    pub filter_active: bool,
    /// Filter: Show only selected agents
    pub filter_selected: bool,
    /// Cached projection: Indices of agents currently visible in the UI
    pub visible_indices: Vec<usize>,
}

impl AppState {
    /// Creates a new AppState with the given config
    pub fn new(config: Config) -> Self {
        let truecolor_supported = std::env::var("COLORTERM")
            .map(|v| v.to_lowercase().contains("truecolor") || v.contains("24bit"))
            .unwrap_or(false)
            || std::env::var("TERM")
                .map(|v| v.contains("truecolor"))
                .unwrap_or(false);

        let sidebar_width = config.sidebar_width.clone();
        Self {
            config,
            agents: AgentTree::new(),
            selected_index: 0,
            selected_agents: HashSet::new(),
            focused_panel: FocusedPanel::Sidebar,
            input_buffer: String::new(),
            cursor_position: 0,
            show_help: false,
            popup_input: None,
            modal_textarea: None,
            filter_pattern: None,
            show_subagent_log: false,
            show_summary_detail: true,
            should_quit: false,
            last_error: Some(format!(
                "tmuxcc v{} [{}] - Press ? for help",
                env!("CARGO_PKG_VERSION"),
                if truecolor_supported { "tc" } else { "256" }
            )),
            sidebar_width,
            tick: 0,
            last_tick: Instant::now(),
            system_stats: SystemStats::new(),
            truecolor_supported,
            current_todo: None,
            show_menu: false,
            menu_tree: MenuTreeState::new(),
            show_prompts: false,
            prompts_tree: MenuTreeState::new(),
            filter_active: false,
            filter_selected: false,
            visible_indices: Vec::new(),
        }
    }

    /// Advance the animation tick (throttled to ~10fps for spinner)
    pub fn tick(&mut self) {
        const TICK_INTERVAL_MS: u128 = 80; // ~12fps for smooth spinner
        if self.last_tick.elapsed().as_millis() >= TICK_INTERVAL_MS {
            self.tick = self.tick.wrapping_add(1);
            self.last_tick = Instant::now();
        }
    }

    /// Get the current spinner frame
    pub fn spinner_frame(&self) -> &'static str {
        SPINNER_FRAMES[self.tick % SPINNER_FRAMES.len()]
    }

    /// Check if input panel is focused
    pub fn is_input_focused(&self) -> bool {
        self.focused_panel == FocusedPanel::Input
    }

    /// Focus on the input panel
    pub fn focus_input(&mut self) {
        self.focused_panel = FocusedPanel::Input;
    }

    /// Focus on the sidebar
    pub fn focus_sidebar(&mut self) {
        self.focused_panel = FocusedPanel::Sidebar;
    }

    /// Toggle focus between panels
    pub fn toggle_focus(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Sidebar => FocusedPanel::Input,
            FocusedPanel::Input => FocusedPanel::Sidebar,
        };
    }

    /// Add a character to the input buffer at cursor position
    pub fn input_char(&mut self, c: char) {
        self.input_buffer.insert(self.cursor_position, c);
        self.cursor_position += c.len_utf8();
    }

    /// Add a newline to the input buffer at cursor position
    pub fn input_newline(&mut self) {
        self.input_buffer.insert(self.cursor_position, '\n');
        self.cursor_position += 1;
    }

    /// Delete the character before the cursor
    pub fn input_backspace(&mut self) {
        if self.cursor_position > 0 {
            // Find the previous character boundary
            let prev_boundary = self.input_buffer[..self.cursor_position]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.input_buffer.remove(prev_boundary);
            self.cursor_position = prev_boundary;
        }
    }

    /// Get the current input buffer
    pub fn get_input(&self) -> &str {
        &self.input_buffer
    }

    /// Get the current cursor position
    pub fn get_cursor_position(&self) -> usize {
        self.cursor_position
    }

    /// Take and clear the input buffer
    pub fn take_input(&mut self) -> String {
        self.cursor_position = 0;
        std::mem::take(&mut self.input_buffer)
    }

    /// Move cursor left by one character
    pub fn cursor_left(&mut self) {
        if self.cursor_position > 0 {
            // Find the previous character boundary
            self.cursor_position = self.input_buffer[..self.cursor_position]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    /// Move cursor right by one character
    pub fn cursor_right(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            // Find the next character boundary
            if let Some(c) = self.input_buffer[self.cursor_position..].chars().next() {
                self.cursor_position += c.len_utf8();
            }
        }
    }

    /// Move cursor to the beginning of the input
    pub fn cursor_home(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to the end of the input
    pub fn cursor_end(&mut self) {
        self.cursor_position = self.input_buffer.len();
    }

    /// Returns the currently selected agent
    pub fn selected_agent(&self) -> Option<&MonitoredAgent> {
        self.agents.get_agent(self.selected_index)
    }

    /// Returns the currently selected agent mutably
    pub fn selected_agent_mut(&mut self) -> Option<&mut MonitoredAgent> {
        self.agents.get_agent_mut(self.selected_index)
    }

    /// Returns the currently selected agent ONLY if it is currently visible
    pub fn selected_visible_agent(&self) -> Option<&MonitoredAgent> {
        if self.visible_indices.contains(&self.selected_index) {
            self.agents.get_agent(self.selected_index)
        } else {
            None
        }
    }

    /// Selects the next visible agent (respects filter)
    pub fn select_next(&mut self) {
        if self.visible_indices.is_empty() {
            return; // No visible agents, no-op
        }

        // Find current position in visible list
        if let Some(current_pos) = self
            .visible_indices
            .iter()
            .position(|&idx| idx == self.selected_index)
        {
            // Move to next visible (wrap around)
            let next_pos = (current_pos + 1) % self.visible_indices.len();
            self.selected_index = self.visible_indices[next_pos];
        } else {
            // Current not visible, jump to first visible
            self.selected_index = self.visible_indices[0];
        }
    }

    /// Selects the previous visible agent (respects filter)
    pub fn select_prev(&mut self) {
        if self.visible_indices.is_empty() {
            return; // No visible agents, no-op
        }

        // Find current position in visible list
        if let Some(current_pos) = self
            .visible_indices
            .iter()
            .position(|&idx| idx == self.selected_index)
        {
            // Move to previous visible (wrap around)
            let prev_pos = if current_pos == 0 {
                self.visible_indices.len() - 1
            } else {
                current_pos - 1
            };
            self.selected_index = self.visible_indices[prev_pos];
        } else {
            // Current not visible, jump to first visible
            self.selected_index = self.visible_indices[0];
        }
    }

    /// Selects an agent by index
    pub fn select_agent(&mut self, index: usize) {
        if index < self.agents.root_agents.len() {
            self.selected_index = index;
        }
    }

    /// Toggles selection of the current agent (only if visible)
    pub fn toggle_selection(&mut self) {
        // Only allow toggling selection of visible agents
        if !self.visible_indices.contains(&self.selected_index) {
            return; // Current agent is hidden, no-op
        }

        if self.selected_agents.contains(&self.selected_index) {
            self.selected_agents.remove(&self.selected_index);
        } else {
            self.selected_agents.insert(self.selected_index);
        }

        // If "Selected Only" filter is active, updating selection might change visibility
        if self.filter_selected {
            self.update_visible_indices();
            self.ensure_visible_selection();
        }
    }

    /// Selects all visible agents (respects filter)
    pub fn select_all(&mut self) {
        // We use a clone to avoid immutable/mutable borrow issues if sub-methods are called
        let visible = self.visible_indices.clone();
        for i in visible {
            self.selected_agents.insert(i);
        }

        if self.filter_selected {
            self.update_visible_indices();
        }
    }

    /// Clears all selections
    pub fn clear_selection(&mut self) {
        self.selected_agents.clear();
        if self.filter_selected {
            self.update_visible_indices();
            self.ensure_visible_selection();
        }
    }

    /// Returns visible indices to operate on (selected agents, or current if none selected)
    /// Only returns indices that are currently visible (match filter)
    pub fn get_operation_indices(&self) -> Vec<usize> {
        if self.selected_agents.is_empty() {
            // If current is visible, return it; otherwise empty
            if self.visible_indices.contains(&self.selected_index) {
                vec![self.selected_index]
            } else {
                vec![]
            }
        } else {
            // Filter selected_agents to only visible ones
            let mut indices: Vec<usize> = self
                .selected_agents
                .iter()
                .copied()
                .filter(|idx| self.visible_indices.contains(idx))
                .collect();
            indices.sort();
            indices
        }
    }

    /// Check if an agent is in multi-selection
    pub fn is_multi_selected(&self, index: usize) -> bool {
        self.selected_agents.contains(&index)
    }

    /// Toggles help display
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        // Create help modal textarea when opening
        if self.show_help && self.modal_textarea.is_none() {
            use crate::ui::components::ModalTextareaState;
            let help_text = ModalTextareaState::new(
                "Help (Readonly)".to_string(),
                "".to_string(),
                crate::ui::components::HelpWidget::generate_help_text(&self.config),
                false, // not single_line
                true,  // readonly!
            );
            self.modal_textarea = Some(help_text);
        }
        // Clear modal textarea when closing help
        if !self.show_help {
            self.modal_textarea = None;
        }
    }

    /// Toggles subagent log display
    pub fn toggle_subagent_log(&mut self) {
        self.show_subagent_log = !self.show_subagent_log;
    }

    /// Toggles summary detail (TODOs and Tools) display
    pub fn toggle_summary_detail(&mut self) {
        self.show_summary_detail = !self.show_summary_detail;
    }

    /// Toggles command menu display
    pub fn toggle_menu(&mut self) {
        self.show_menu = !self.show_menu;
        // Close prompts if menu opens
        if self.show_menu {
            self.show_prompts = false;
        } else {
            // Clear filter when closing?
            self.menu_tree.filter.clear();
        }
    }

    /// Toggles prompts menu display
    pub fn toggle_prompts(&mut self) {
        self.show_prompts = !self.show_prompts;
        // Close menu if prompts opens
        if self.show_prompts {
            self.show_menu = false;
        } else {
            self.prompts_tree.filter.clear();
        }
    }

    /// Sets an error message
    pub fn set_error(&mut self, message: String) {
        self.last_error = Some(message);
    }

    /// Set a status message (non-error, displayed differently)
    pub fn set_status(&mut self, message: String) {
        self.last_error = None;
        self.set_error(message);
    }

    /// Sets the filter pattern and updates visibility projection
    pub fn set_filter_pattern(&mut self, pattern: Option<String>) {
        self.filter_pattern = pattern;
        self.update_visible_indices();
        self.ensure_visible_selection();
    }

    /// Clears the error message
    pub fn clear_error(&mut self) {
        self.last_error = None;
    }

    /// Logs an action to the status bar if log_actions is enabled
    pub fn log_action(&mut self, action: &super::Action) {
        if !self.config.log_actions {
            return;
        }

        use super::Action;
        match action {
            Action::None
            | Action::InputChar(_)
            | Action::InputNewline
            | Action::InputBackspace
            | Action::CursorLeft
            | Action::CursorRight
            | Action::CursorHome
            | Action::CursorEnd
            | Action::PopupInputChar(_)
            | Action::PopupInputBackspace
            | Action::PopupInputDelete
            | Action::PopupInputCursorLeft
            | Action::PopupInputCursorRight
            | Action::PopupInputCursorHome
            | Action::PopupInputCursorEnd => {}
            _ => {
                let desc = action.description();
                if !desc.is_empty() {
                    self.set_status(desc.to_string());
                }
            }
        }
    }

    /// Check if an agent matches the current filter pattern
    pub fn matches_filter(&self, agent: &MonitoredAgent) -> bool {
        // Find index of this agent (fallback, O(N))
        let index = self
            .agents
            .root_agents
            .iter()
            .position(|a| std::ptr::eq(a, agent));
        if let Some(idx) = index {
            self.matches_filter_impl(idx, agent)
        } else {
            // Should happen only if agent is not in root_agents??
            // Just use text pattern matching as fallback if index not found
            // This is safer than unwrapping
            match &self.filter_pattern {
                None => true,
                Some(pattern) if pattern.is_empty() => true,
                Some(pattern) => {
                    let pattern_lower = pattern.to_lowercase();
                    agent
                        .agent_type
                        .to_string()
                        .to_lowercase()
                        .contains(&pattern_lower)
                        || agent.session.to_lowercase().contains(&pattern_lower)
                        || agent.window_name.to_lowercase().contains(&pattern_lower)
                        || agent.target.to_lowercase().contains(&pattern_lower)
                        || agent.path.to_lowercase().contains(&pattern_lower)
                }
            }
        }
    }

    /// Check if an agent matches the current filter pattern (internal logic)
    pub fn matches_filter_impl(&self, index: usize, agent: &MonitoredAgent) -> bool {
        // Boolean filters (Active / Selected)
        // Treated as OR (Union): If any are enabled, agent must match AT LEAST ONE of the enabled filters.
        if self.filter_active || self.filter_selected {
            let mut matched = false;

            if self.filter_active {
                // Active = Not Idle
                if !matches!(agent.status, crate::agents::AgentStatus::Idle { .. }) {
                    matched = true;
                }
            }

            if self.filter_selected && self.selected_agents.contains(&index) {
                matched = true;
            }

            if !matched {
                return false;
            }
        }

        // Text pattern (Text filter is always AND logic with the above)
        match &self.filter_pattern {
            None => true,                                // No filter = show all
            Some(pattern) if pattern.is_empty() => true, // Empty = show all
            Some(pattern) => {
                let pattern_lower = pattern.to_lowercase();

                // Match against multiple fields
                agent
                    .agent_type
                    .to_string()
                    .to_lowercase()
                    .contains(&pattern_lower)
                    || agent.session.to_lowercase().contains(&pattern_lower)
                    || agent.window_name.to_lowercase().contains(&pattern_lower)
                    || agent.target.to_lowercase().contains(&pattern_lower)
                    || agent.path.to_lowercase().contains(&pattern_lower)
            }
        }
    }

    /// Update the cached projection of visible agents.
    /// This should be called whenever agents or filters change.
    pub fn update_visible_indices(&mut self) {
        self.visible_indices = self
            .agents
            .root_agents
            .iter()
            .enumerate()
            .filter(|(idx, agent)| self.matches_filter_impl(*idx, agent))
            .map(|(idx, _)| idx)
            .collect();
    }

    // ... (rest of methods)

    // ... (in tests module)

    /// Get filtered agent list
    pub fn filtered_agents(&self) -> Vec<&MonitoredAgent> {
        self.visible_indices
            .iter()
            .filter_map(|&idx| self.agents.root_agents.get(idx))
            .collect()
    }

    /// Get filtered agent list with their original indices
    pub fn filtered_agents_with_indices(&self) -> Vec<(usize, &MonitoredAgent)> {
        self.visible_indices
            .iter()
            .filter_map(|&idx| self.agents.root_agents.get(idx).map(|agent| (idx, agent)))
            .collect()
    }

    /// Returns indices of agents that are currently visible (match filter)
    pub fn visible_agent_indices(&self) -> Vec<usize> {
        self.visible_indices.clone()
    }

    /// Ensures the current selection points to a visible agent.
    /// If actual current agent is hidden, tries to find the nearest visible neighbor.
    pub fn ensure_visible_selection(&mut self) {
        if self.visible_indices.is_empty() {
            return;
        }

        if self.visible_indices.contains(&self.selected_index) {
            return;
        }

        let mut nearest_idx = self.visible_indices[0];
        let mut min_diff = (self.selected_index as isize - nearest_idx as isize).abs();

        for &idx in &self.visible_indices {
            let diff = (self.selected_index as isize - idx as isize).abs();
            if diff < min_diff {
                min_diff = diff;
                nearest_idx = idx;
            }
        }
        self.selected_index = nearest_idx;
    }

    pub fn toggle_filter_active(&mut self) {
        self.filter_active = !self.filter_active;
        self.update_visible_indices();
        self.ensure_visible_selection();
    }

    pub fn toggle_filter_selected(&mut self) {
        self.filter_selected = !self.filter_selected;
        self.update_visible_indices();
        self.ensure_visible_selection();
    }

    /// Refresh the current project TODO content based on the selected agent's path
    pub fn refresh_project_todo(&mut self) {
        if !self.config.todo_from_file {
            self.current_todo = None;
            return;
        }

        let path = if let Some(agent) = self.selected_agent() {
            &agent.path
        } else {
            self.current_todo = None;
            return;
        };

        if path.is_empty() {
            self.current_todo = None;
            return;
        }

        let mut todo_content = None;
        for file in &self.config.todo_files {
            let full_path = std::path::Path::new(path).join(file);
            if full_path.exists() && full_path.is_file() {
                // Read the first few lines
                if let Ok(content) = std::fs::read_to_string(full_path) {
                    // Limit to first 20 lines to keep it reasonable
                    let lines: Vec<&str> = content.lines().take(20).collect();
                    todo_content = Some(lines.join("\n"));
                    break;
                }
            }
        }
        self.current_todo = todo_content;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::AgentType;

    #[test]
    fn test_app_state_navigation() {
        let config = Config::default();
        let mut state = AppState::new(config);

        // Add some agents
        state.agents.root_agents.push(MonitoredAgent::new(
            "1".to_string(),
            "Claude".to_string(),
            Some("magenta".to_string()),
            "main:0.0".to_string(),
            "main".to_string(),
            0,
            "code".to_string(),
            0,
            "/home/user/project1".to_string(),
            AgentType::Named("Claude".to_string()),
            None,
            1000,
        ));
        state.agents.root_agents.push(MonitoredAgent::new(
            "2".to_string(),
            "OpenCode".to_string(),
            Some("blue".to_string()),
            "main:0.1".to_string(),
            "main".to_string(),
            0,
            "code".to_string(),
            1,
            "/home/user/project2".to_string(),
            AgentType::Named("OpenCode".to_string()),
            None,
            1001,
        ));
        state.update_visible_indices();

        assert_eq!(state.selected_index, 0);
        state.select_next();
        assert_eq!(state.selected_index, 1);
        state.select_next();
        assert_eq!(state.selected_index, 0); // Wraps around
        state.select_prev();
        assert_eq!(state.selected_index, 1); // Wraps around
    }

    /// Helper to create a test agent with the given session name
    fn create_test_agent(id: &str, session: &str, pane_index: u32) -> MonitoredAgent {
        MonitoredAgent::new(
            id.to_string(),
            "Test Agent".to_string(),
            Some("cyan".to_string()),
            format!("{}:0.{}", session, pane_index),
            session.to_string(),
            0,
            "code".to_string(),
            pane_index,
            format!("/home/user/{}", session),
            AgentType::Named("Agent".to_string()),
            None,
            1000 + pane_index,
        )
    }

    #[test]
    fn test_navigation_with_filter_skips_hidden() {
        let mut state = AppState::default();

        // Add 3 agents: session-a, session-b, session-c
        state
            .agents
            .root_agents
            .push(create_test_agent("1", "session-a", 0));
        state
            .agents
            .root_agents
            .push(create_test_agent("2", "session-b", 1));
        state
            .agents
            .root_agents
            .push(create_test_agent("3", "session-c", 2));
        state.update_visible_indices();

        // Filter to hide session-b (index 1)
        state.set_filter_pattern(Some("session-a".to_string()));

        // Start at first visible (session-a, idx 0)
        state.selected_index = 0;

        // Only session-a matches, so next should stay at 0 (wrap around to same)
        state.select_next();
        assert_eq!(state.selected_index, 0);

        // Previous should also stay at 0
        state.select_prev();
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_navigation_active_filter_skips_idle() {
        let mut state = AppState::default();
        use crate::agents::AgentStatus;

        // Add 3 agents: Idle, Processing, Idle
        let mut a1 = create_test_agent("1", "idle1", 0);
        a1.status = AgentStatus::Idle { label: None };
        state.agents.root_agents.push(a1);

        let mut a2 = create_test_agent("2", "working", 1);
        a2.status = AgentStatus::Processing {
            activity: "work".to_string(),
        };
        state.agents.root_agents.push(a2);

        let mut a3 = create_test_agent("3", "idle2", 2);
        a3.status = AgentStatus::Idle { label: None };
        state.agents.root_agents.push(a3);
        state.update_visible_indices();

        // Verify all 3 present
        assert_eq!(state.visible_agent_indices(), vec![0, 1, 2]);

        // Enable Active Filter (should hide 0 and 2)
        state.toggle_filter_active();
        assert!(state.filter_active);

        // Only 1 should be visible
        assert_eq!(state.visible_agent_indices(), vec![1]);

        // Current selection should account for visibility (ensure_visible_selection called by toggle)
        // Default select was 0. 0 is hidden. Nearest visible is 1.
        assert_eq!(state.selected_index, 1);

        // Next should loop to 1
        state.select_next();
        assert_eq!(state.selected_index, 1);

        // Prev should loop to 1
        state.select_prev();
        assert_eq!(state.selected_index, 1);

        if let Some(agent) = state.agents.get_agent_mut(2) {
            agent.status = AgentStatus::Processing {
                activity: "work".to_string(),
            };
        }
        state.update_visible_indices();

        // Now 1 and 2 visible
        assert_eq!(state.visible_agent_indices(), vec![1, 2]);

        // Navigation: 1 -> 2 -> 1
        assert_eq!(state.selected_index, 1);
        state.select_next();
        assert_eq!(state.selected_index, 2);
        state.select_next();
        assert_eq!(state.selected_index, 1);
    }

    #[test]
    fn test_navigation_with_filter_multiple_visible() {
        let mut state = AppState::default();

        // Add 3 agents
        state
            .agents
            .root_agents
            .push(create_test_agent("1", "test-a", 0));
        state
            .agents
            .root_agents
            .push(create_test_agent("2", "prod-b", 1));
        state
            .agents
            .root_agents
            .push(create_test_agent("3", "test-c", 2));
        state.update_visible_indices();

        // Filter to show only "test" sessions (indices 0 and 2)
        state.set_filter_pattern(Some("test".to_string()));

        // Start at index 0
        state.selected_index = 0;

        // Next should jump to index 2 (skipping hidden index 1)
        state.select_next();
        assert_eq!(state.selected_index, 2);

        // Next should wrap to index 0
        state.select_next();
        assert_eq!(state.selected_index, 0);

        // Prev from 0 should go to 2
        state.select_prev();
        assert_eq!(state.selected_index, 2);

        // Prev from 2 should go to 0
        state.select_prev();
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_navigation_empty_filter_result() {
        let mut state = AppState::default();

        // Add agents
        state
            .agents
            .root_agents
            .push(create_test_agent("1", "session-a", 0));
        state
            .agents
            .root_agents
            .push(create_test_agent("2", "session-b", 1));
        state.update_visible_indices();

        // Filter that matches nothing
        state.set_filter_pattern(Some("nonexistent".to_string()));

        // Store original index
        let original = state.selected_index;

        // Navigation should be no-op
        state.select_next();
        assert_eq!(state.selected_index, original);

        state.select_prev();
        assert_eq!(state.selected_index, original);
    }

    #[test]
    fn test_select_all_with_filter() {
        let mut state = AppState::default();

        // Add 3 agents
        state
            .agents
            .root_agents
            .push(create_test_agent("1", "test-a", 0));
        state
            .agents
            .root_agents
            .push(create_test_agent("2", "prod-b", 1));
        state
            .agents
            .root_agents
            .push(create_test_agent("3", "test-c", 2));
        state.update_visible_indices();

        // Filter to show only "test" sessions
        state.set_filter_pattern(Some("test".to_string()));

        // Select all should only select visible (indices 0 and 2)
        state.select_all();

        assert!(state.selected_agents.contains(&0));
        assert!(!state.selected_agents.contains(&1)); // Hidden, not selected
        assert!(state.selected_agents.contains(&2));
        assert_eq!(state.selected_agents.len(), 2);
    }

    #[test]
    fn test_ensure_visible_selection() {
        let mut state = AppState::default();

        // Add 3 agents
        state
            .agents
            .root_agents
            .push(create_test_agent("1", "test-a", 0));
        state
            .agents
            .root_agents
            .push(create_test_agent("2", "prod-b", 1));
        state
            .agents
            .root_agents
            .push(create_test_agent("3", "test-c", 2));
        state.update_visible_indices();

        // Select all agents first (no filter)
        state.select_all();
        assert_eq!(state.selected_agents.len(), 3);

        // Set cursor to prod-b (index 1)
        state.selected_index = 1;

        // Now filter to hide prod-b
        state.set_filter_pattern(Some("test".to_string()));

        // Cursor should move to first visible (index 0)
        assert_eq!(state.selected_index, 0);

        // Multi-selection should still contain hidden indices (design choice: preserve selection across filters)
        assert!(state.selected_agents.contains(&0));
        assert!(state.selected_agents.contains(&1)); // Hidden, BUT preserved in set
        assert!(state.selected_agents.contains(&2));
        assert_eq!(state.selected_agents.len(), 3);
    }

    #[test]
    fn test_get_operation_indices_with_filter() {
        let mut state = AppState::default();

        // Add 3 agents
        state
            .agents
            .root_agents
            .push(create_test_agent("1", "test-a", 0));
        state
            .agents
            .root_agents
            .push(create_test_agent("2", "prod-b", 1));
        state
            .agents
            .root_agents
            .push(create_test_agent("3", "test-c", 2));
        state.update_visible_indices();

        // Select all before filter
        state.select_all();

        // Apply filter to hide prod-b
        state.set_filter_pattern(Some("test".to_string()));

        // get_operation_indices should only return visible ones
        let indices = state.get_operation_indices();
        assert_eq!(indices, vec![0, 2]); // Only visible indices, sorted
    }

    #[test]
    fn test_toggle_selection_hidden_agent() {
        let mut state = AppState::default();

        // Add agents
        state
            .agents
            .root_agents
            .push(create_test_agent("1", "test-a", 0));
        state
            .agents
            .root_agents
            .push(create_test_agent("2", "prod-b", 1));
        state.update_visible_indices();

        // Filter to hide prod-b
        state.set_filter_pattern(Some("test".to_string()));

        // Set cursor to hidden agent (shouldn't happen normally, but test edge case)
        state.selected_index = 1;

        // Toggle should be no-op
        state.toggle_selection();
        assert!(!state.selected_agents.contains(&1));

        // Move to visible agent
        state.selected_index = 0;
        state.toggle_selection();
        assert!(state.selected_agents.contains(&0));
    }

    #[test]
    fn test_filters_union() {
        let mut state = AppState::default();
        use crate::agents::AgentStatus;

        // 0: Idle, Selected
        let mut a0 = create_test_agent("0", "idle-sel", 0);
        a0.status = AgentStatus::Idle { label: None };
        state.agents.root_agents.push(a0);

        // 1: Working, Selected
        let mut a1 = create_test_agent("1", "work-sel", 1);
        a1.status = AgentStatus::Processing {
            activity: "work".to_string(),
        };
        state.agents.root_agents.push(a1);

        // 2: Working, Not Selected
        let mut a2 = create_test_agent("2", "work-unsel", 2);
        a2.status = AgentStatus::Processing {
            activity: "work".to_string(),
        };
        state.agents.root_agents.push(a2);

        // 3: Idle, Not Selected
        let mut a3 = create_test_agent("3", "idle-unsel", 3);
        a3.status = AgentStatus::Idle { label: None };
        state.agents.root_agents.push(a3);
        state.update_visible_indices();

        // Select 0 and 1
        state.selected_agents.insert(0);
        state.selected_agents.insert(1);
        state.update_visible_indices();

        // 1. No filters -> All visible
        assert_eq!(state.visible_agent_indices(), vec![0, 1, 2, 3]);

        // 2. Active Only -> 1, 2 (Working)
        state.toggle_filter_active();
        assert_eq!(state.visible_agent_indices(), vec![1, 2]);

        // 3. Selected Only (Active still ON) -> Union (Active OR Selected)
        // Active match: 1, 2
        // Selected match: 0, 1
        // Union: 0, 1, 2
        state.toggle_filter_selected();
        assert_eq!(state.visible_agent_indices(), vec![0, 1, 2]);

        // 4. Active OFF, Selected Only -> 0, 1
        state.toggle_filter_active();
        assert_eq!(state.visible_agent_indices(), vec![0, 1]);
    }
}
