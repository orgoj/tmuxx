use crate::agents::MonitoredAgent;
use crate::monitor::SystemStats;
use crate::ui::components::ModalTextareaState;
// use ratatui::style::{Color, Style};
use std::collections::HashSet;
use std::time::Instant;

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
            .filter(|a| {
                matches!(
                    a.status,
                    AgentStatus::Processing { .. } | AgentStatus::Tui { .. }
                )
            })
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
    /// Sidebar width in percentage (15-70)
    pub sidebar_width: u16,
    /// Animation tick counter
    pub tick: usize,
    /// Last tick time for animation throttling
    last_tick: Instant,
    /// System resource statistics
    pub system_stats: SystemStats,
    /// Whether the terminal likely supports TrueColor
    pub truecolor_supported: bool,
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
            sidebar_width: 35,
            tick: 0,
            last_tick: Instant::now(),
            system_stats: SystemStats::new(),
            truecolor_supported,
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

    /// Selects the next visible agent (respects filter)
    pub fn select_next(&mut self) {
        let visible = self.visible_agent_indices();
        if visible.is_empty() {
            return; // No visible agents, no-op
        }

        // Find current position in visible list
        if let Some(current_pos) = visible.iter().position(|&idx| idx == self.selected_index) {
            // Move to next visible (wrap around)
            let next_pos = (current_pos + 1) % visible.len();
            self.selected_index = visible[next_pos];
        } else {
            // Current not visible, jump to first visible
            self.selected_index = visible[0];
        }
    }

    /// Selects the previous visible agent (respects filter)
    pub fn select_prev(&mut self) {
        let visible = self.visible_agent_indices();
        if visible.is_empty() {
            return; // No visible agents, no-op
        }

        // Find current position in visible list
        if let Some(current_pos) = visible.iter().position(|&idx| idx == self.selected_index) {
            // Move to previous visible (wrap around)
            let prev_pos = if current_pos == 0 {
                visible.len() - 1
            } else {
                current_pos - 1
            };
            self.selected_index = visible[prev_pos];
        } else {
            // Current not visible, jump to first visible
            self.selected_index = visible[0];
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
        let visible = self.visible_agent_indices();
        if !visible.contains(&self.selected_index) {
            return; // Current agent is hidden, no-op
        }

        if self.selected_agents.contains(&self.selected_index) {
            self.selected_agents.remove(&self.selected_index);
        } else {
            self.selected_agents.insert(self.selected_index);
        }
    }

    /// Selects all visible agents (respects filter)
    pub fn select_all(&mut self) {
        let visible = self.visible_agent_indices();
        for i in visible {
            self.selected_agents.insert(i);
        }
    }

    /// Clears all selections
    pub fn clear_selection(&mut self) {
        self.selected_agents.clear();
    }

    /// Returns visible indices to operate on (selected agents, or current if none selected)
    /// Only returns indices that are currently visible (match filter)
    pub fn get_operation_indices(&self) -> Vec<usize> {
        let visible = self.visible_agent_indices();

        if self.selected_agents.is_empty() {
            // If current is visible, return it; otherwise empty
            if visible.contains(&self.selected_index) {
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
                .filter(|idx| visible.contains(idx))
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

    /// Sets an error message
    pub fn set_error(&mut self, message: String) {
        self.last_error = Some(message);
    }

    /// Set a status message (non-error, displayed differently)
    pub fn set_status(&mut self, message: String) {
        self.last_error = Some(format!("✓ {}", message));
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

    /// Get filtered agent list
    /// Returns Vec for flexibility. If performance becomes an issue,
    /// consider caching or using iterator instead.
    pub fn filtered_agents(&self) -> Vec<&MonitoredAgent> {
        self.agents
            .root_agents
            .iter()
            .filter(|agent| self.matches_filter(agent))
            .collect()
    }

    /// Returns indices of agents that are currently visible (match filter)
    /// These are indices into the unfiltered root_agents list
    pub fn visible_agent_indices(&self) -> Vec<usize> {
        self.agents
            .root_agents
            .iter()
            .enumerate()
            .filter(|(_, agent)| self.matches_filter(agent))
            .map(|(idx, _)| idx)
            .collect()
    }

    /// Ensures the current selection points to a visible agent.
    /// If current agent is hidden by filter, jumps to first visible.
    /// Also removes hidden agents from multi-selection.
    pub fn ensure_visible_selection(&mut self) {
        let visible = self.visible_agent_indices();

        // Remove hidden agents from multi-selection
        self.selected_agents.retain(|idx| visible.contains(idx));

        // If current cursor is not visible, jump to first visible
        if !visible.is_empty() && !visible.contains(&self.selected_index) {
            self.selected_index = visible[0];
        }
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
            "magenta".to_string(),
            "main:0.0".to_string(),
            "main".to_string(),
            0,
            "code".to_string(),
            0,
            "/home/user/project1".to_string(),
            AgentType::ClaudeCode,
            1000,
        ));
        state.agents.root_agents.push(MonitoredAgent::new(
            "2".to_string(),
            "OpenCode".to_string(),
            "blue".to_string(),
            "main:0.1".to_string(),
            "main".to_string(),
            0,
            "code".to_string(),
            1,
            "/home/user/project2".to_string(),
            AgentType::OpenCode,
            1001,
        ));

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
            "cyan".to_string(),
            format!("{}:0.{}", session, pane_index),
            session.to_string(),
            0,
            "code".to_string(),
            pane_index,
            format!("/home/user/{}", session),
            AgentType::ClaudeCode,
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

        // Filter to hide session-b (index 1)
        state.filter_pattern = Some("session-a".to_string());

        // Start at first visible (session-a, idx 0)
        state.selected_index = 0;

        // Only session-a matches, so next should stay at 0 (wrap around to same)
        state.select_next();
        assert_eq!(state.selected_index, 0);

        // Previous should also stay at 0
        state.select_prev();
        assert_eq!(state.selected_index, 0);

        // Now filter to show a and c (hide b)
        state.filter_pattern = Some("session-".to_string()); // matches all
        state.filter_pattern = None; // show all first
        state.filter_pattern = Some("session-a".to_string());

        // Verify we only see session-a
        let visible = state.visible_agent_indices();
        assert_eq!(visible, vec![0]);
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

        // Filter to show only "test" sessions (indices 0 and 2)
        state.filter_pattern = Some("test".to_string());

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

        // Filter that matches nothing
        state.filter_pattern = Some("nonexistent".to_string());

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

        // Filter to show only "test" sessions
        state.filter_pattern = Some("test".to_string());

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

        // Select all agents first (no filter)
        state.select_all();
        assert_eq!(state.selected_agents.len(), 3);

        // Set cursor to prod-b (index 1)
        state.selected_index = 1;

        // Now filter to hide prod-b
        state.filter_pattern = Some("test".to_string());
        state.ensure_visible_selection();

        // Cursor should move to first visible (index 0)
        assert_eq!(state.selected_index, 0);

        // Multi-selection should only contain visible indices
        assert!(state.selected_agents.contains(&0));
        assert!(!state.selected_agents.contains(&1)); // Hidden, removed
        assert!(state.selected_agents.contains(&2));
        assert_eq!(state.selected_agents.len(), 2);
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

        // Select all before filter
        state.select_all();

        // Apply filter to hide prod-b
        state.filter_pattern = Some("test".to_string());

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

        // Filter to hide prod-b
        state.filter_pattern = Some("test".to_string());

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
}
