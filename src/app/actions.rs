use super::key_binding::KillMethod;
use super::state::PopupType;

/// Actions that can be performed in the application
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Quit the application
    Quit,
    /// Navigate to next agent
    NextAgent,
    /// Navigate to previous agent
    PrevAgent,
    /// Toggle selection of current agent
    ToggleSelection,
    /// Select all agents
    SelectAll,
    /// Clear selection
    ClearSelection,
    /// Approve the current/selected request(s)
    Approve,
    /// Reject the current/selected request(s)
    Reject,
    /// Approve all pending requests
    ApproveAll,
    /// Focus on the selected tmux pane
    FocusPane,
    /// Toggle subagent log view
    ToggleSubagentLog,
    /// Toggle summary detail (TODOs and Tools) view
    ToggleSummaryDetail,
    /// Toggle command menu
    ToggleMenu,
    /// Refresh agent list
    Refresh,
    /// Show help
    ShowHelp,
    /// Hide help
    HideHelp,
    /// Focus on input panel
    FocusInput,
    /// Focus on sidebar
    FocusSidebar,
    /// Send input to selected agent
    SendInput,
    /// Clear input buffer
    ClearInput,
    /// Add character to input
    InputChar(char),
    /// Add newline to input
    InputNewline,
    /// Delete last character
    InputBackspace,
    /// Move cursor left
    CursorLeft,
    /// Move cursor right
    CursorRight,
    /// Move cursor to beginning
    CursorHome,
    /// Move cursor to end
    CursorEnd,
    /// Send a specific number (for choice selection)
    SendNumber(u8),
    /// Increase sidebar width
    SidebarWider,
    /// Decrease sidebar width
    SidebarNarrower,
    /// Select agent by index (mouse click)
    SelectAgent(usize),
    /// Scroll up in sidebar
    ScrollUp,
    /// Scroll down in sidebar
    ScrollDown,
    /// Send custom keys to agent pane
    SendKeys(String),
    /// Kill application in agent pane
    KillApp { method: KillMethod },
    /// Show popup input dialog
    ShowPopupInput {
        title: String,
        prompt: String,
        initial: String,
        popup_type: PopupType,
    },
    /// Hide popup without submitting
    HidePopupInput,
    /// Submit popup input (returns text to caller)
    PopupInputSubmit,
    /// Add character to popup buffer
    PopupInputChar(char),
    /// Delete character before cursor (backspace)
    PopupInputBackspace,
    /// Delete character after cursor (delete)
    PopupInputDelete,
    /// Clear popup buffer (Ctrl+U)
    PopupInputClear,
    /// Select all and replace (Ctrl+A)
    PopupInputSelectAll,
    /// Move popup cursor left
    PopupInputCursorLeft,
    /// Move popup cursor right
    PopupInputCursorRight,
    /// Move popup cursor to home
    PopupInputCursorHome,
    /// Move popup cursor to end
    PopupInputCursorEnd,
    /// Execute a shell command with variable expansion
    ExecuteCommand {
        command: String,
        blocking: bool,
        terminal: bool,
    },
    /// Show modal textarea dialog
    ShowModalTextarea {
        title: String,
        prompt: String,
        initial: String,
        single_line: bool,
    },
    /// Hide modal textarea without submitting
    HideModalTextarea,
    /// Submit modal textarea (returns text)
    ModalTextareaSubmit,
    /// Capture current pane content as a test case
    CaptureTestCase,
    /// No action (used for unbound keys)
    None,
    /// Toggle pane tree display mode (compact/full)
    TogglePaneTreeMode,
}

impl Action {
    /// Returns a description of the action for help display
    pub fn description(&self) -> &str {
        match self {
            Action::Quit => "Quit application",
            Action::NextAgent => "Select next agent",
            Action::PrevAgent => "Select previous agent",
            Action::ToggleSelection => "Toggle selection",
            Action::SelectAll => "Select all agents",
            Action::ClearSelection => "Clear selection",
            Action::Approve => "Approve selected request(s)",
            Action::Reject => "Reject selected request(s)",
            Action::ApproveAll => "Approve all pending requests",
            Action::FocusPane => "Focus on selected pane in tmux",
            Action::ToggleSubagentLog => "Toggle subagent log",
            Action::ToggleSummaryDetail => "Toggle TODO/Tools display",
            Action::ToggleMenu => "Toggle command menu",
            Action::Refresh => "Refresh agent list",
            Action::ShowHelp => "Show help",
            Action::HideHelp => "Hide help",
            Action::FocusInput => "Focus input panel",
            Action::FocusSidebar => "Focus sidebar",
            Action::SendInput => "Send input",
            Action::ClearInput => "Clear input",
            Action::InputChar(_) => "Type character",
            Action::InputNewline => "Insert newline",
            Action::InputBackspace => "Delete character",
            Action::CursorLeft => "Move cursor left",
            Action::CursorRight => "Move cursor right",
            Action::CursorHome => "Move cursor to start",
            Action::CursorEnd => "Move cursor to end",
            Action::SendNumber(_) => "Send choice number",
            Action::SidebarWider => "Widen sidebar",
            Action::SidebarNarrower => "Narrow sidebar",
            Action::SelectAgent(_) => "Select agent",
            Action::ScrollUp => "Scroll up",
            Action::ScrollDown => "Scroll down",
            Action::SendKeys(_) => "Send keys to pane",
            Action::KillApp { .. } => "Kill application",
            Action::ShowPopupInput { .. } => "Show popup input",
            Action::HidePopupInput => "Hide popup",
            Action::PopupInputSubmit => "Submit popup input",
            Action::PopupInputChar(_) => "Type in popup",
            Action::PopupInputBackspace => "Delete character (popup)",
            Action::PopupInputDelete => "Delete forward (popup)",
            Action::PopupInputClear => "Clear popup input",
            Action::PopupInputSelectAll => "Select all (popup)",
            Action::PopupInputCursorLeft => "Move cursor left (popup)",
            Action::PopupInputCursorRight => "Move cursor right (popup)",
            Action::PopupInputCursorHome => "Move cursor home (popup)",
            Action::PopupInputCursorEnd => "Move cursor end (popup)",
            Action::ExecuteCommand { .. } => "Execute command",
            Action::ShowModalTextarea { .. } => "Show modal textarea",
            Action::HideModalTextarea => "Hide modal textarea",
            Action::ModalTextareaSubmit => "Submit modal textarea",
            Action::CaptureTestCase => "Capture test case",
            Action::TogglePaneTreeMode => "Toggle pane tree mode (compact/full)",
            Action::None => "",
        }
    }
}
