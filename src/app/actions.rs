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
    /// Refresh agent list
    Refresh,
    /// Show help
    ShowHelp,
    /// Hide help
    HideHelp,
    /// Enter input mode
    EnterInputMode,
    /// Send input and exit input mode
    SendInput,
    /// Cancel input mode
    CancelInput,
    /// Add character to input
    InputChar(char),
    /// Delete last character
    InputBackspace,
    /// Send a specific number (for choice selection)
    SendNumber(u8),
    /// Increase sidebar width
    SidebarWider,
    /// Decrease sidebar width
    SidebarNarrower,
    /// No action (used for unbound keys)
    None,
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
            Action::Refresh => "Refresh agent list",
            Action::ShowHelp => "Show help",
            Action::HideHelp => "Hide help",
            Action::EnterInputMode => "Enter input mode",
            Action::SendInput => "Send input",
            Action::CancelInput => "Cancel input",
            Action::InputChar(_) => "Type character",
            Action::InputBackspace => "Delete character",
            Action::SendNumber(_) => "Send choice number",
            Action::SidebarWider => "Widen sidebar",
            Action::SidebarNarrower => "Narrow sidebar",
            Action::None => "",
        }
    }
}
