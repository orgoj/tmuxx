use std::io;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;

use crate::app::{Action, AppState, Config};
use crate::monitor::MonitorTask;
use crate::tmux::TmuxClient;
use crate::parsers::ParserRegistry;

use super::components::{
    AgentTreeWidget, FooterWidget, HelpWidget, PanePreviewWidget, SubagentLogWidget,
};
use super::Layout;

/// Runs the main application loop
pub async fn run_app(config: Config) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize state
    let mut state = AppState::new();

    // Create tmux client and parser registry
    let tmux_client = Arc::new(TmuxClient::with_capture_lines(config.capture_lines));
    let parser_registry = Arc::new(ParserRegistry::new());

    // Check if tmux is available
    if !tmux_client.is_available() {
        state.set_error("tmux is not running".to_string());
    }

    // Create channel for monitor updates
    let (tx, mut rx) = mpsc::channel(32);

    // Start monitor task
    let monitor = MonitorTask::new(
        tmux_client.clone(),
        parser_registry.clone(),
        tx,
        Duration::from_millis(config.poll_interval_ms),
    );
    let monitor_handle = tokio::spawn(async move {
        monitor.run().await;
    });

    // Main loop
    let result = run_loop(&mut terminal, &mut state, &mut rx, &tmux_client).await;

    // Cleanup
    monitor_handle.abort();
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut AppState,
    rx: &mut mpsc::Receiver<crate::monitor::MonitorUpdate>,
    tmux_client: &TmuxClient,
) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|frame| {
            let size = frame.area();
            let main_chunks = Layout::main_layout(size);

            // Content area - 2 column layout
            if state.show_subagent_log {
                let (left, preview, subagent_log) = Layout::content_layout_with_log(main_chunks[0], state.sidebar_width);
                AgentTreeWidget::render(frame, left, state);
                PanePreviewWidget::render_detailed(frame, preview, state);
                SubagentLogWidget::render(frame, subagent_log, state);
            } else {
                let (left, right) = Layout::content_layout(main_chunks[0], state.sidebar_width);
                AgentTreeWidget::render(frame, left, state);
                PanePreviewWidget::render_detailed(frame, right, state);
            }

            // Footer
            FooterWidget::render(frame, main_chunks[1], state);

            // Help overlay
            if state.show_help {
                HelpWidget::render(frame, size);
            }
        })?;

        // Handle events with timeout to allow monitor updates
        let timeout = Duration::from_millis(100);

        tokio::select! {
            // Handle monitor updates
            Some(update) = rx.recv() => {
                state.agents = update.agents;
                // Ensure selected index is valid
                if state.selected_index >= state.agents.root_agents.len() {
                    state.selected_index = state.agents.root_agents.len().saturating_sub(1);
                }
                // Clean up invalid selections
                let max_idx = state.agents.root_agents.len();
                state.selected_agents.retain(|&idx| idx < max_idx);
            }

            // Handle keyboard events
            _ = tokio::time::sleep(timeout) => {
                if event::poll(Duration::from_millis(0))? {
                    if let Event::Key(key) = event::read()? {
                        let action = map_key_to_action(key.code, key.modifiers, state);

                        match action {
                            Action::Quit => {
                                state.should_quit = true;
                            }
                            Action::NextAgent => {
                                state.select_next();
                            }
                            Action::PrevAgent => {
                                state.select_prev();
                            }
                            Action::ToggleSelection => {
                                state.toggle_selection();
                            }
                            Action::SelectAll => {
                                state.select_all();
                            }
                            Action::ClearSelection => {
                                state.clear_selection();
                            }
                            Action::Approve => {
                                let indices = state.get_operation_indices();
                                for idx in indices {
                                    if let Some(agent) = state.agents.get_agent(idx) {
                                        if agent.status.needs_attention() {
                                            let target = agent.target.clone();
                                            if let Err(e) = tmux_client.send_keys(&target, "y") {
                                                state.set_error(format!("Failed to approve: {}", e));
                                                break;
                                            }
                                            if let Err(e) = tmux_client.send_keys(&target, "Enter") {
                                                state.set_error(format!("Failed to send Enter: {}", e));
                                                break;
                                            }
                                        }
                                    }
                                }
                                state.clear_selection();
                            }
                            Action::Reject => {
                                let indices = state.get_operation_indices();
                                for idx in indices {
                                    if let Some(agent) = state.agents.get_agent(idx) {
                                        if agent.status.needs_attention() {
                                            let target = agent.target.clone();
                                            if let Err(e) = tmux_client.send_keys(&target, "n") {
                                                state.set_error(format!("Failed to reject: {}", e));
                                                break;
                                            }
                                            if let Err(e) = tmux_client.send_keys(&target, "Enter") {
                                                state.set_error(format!("Failed to send Enter: {}", e));
                                                break;
                                            }
                                        }
                                    }
                                }
                                state.clear_selection();
                            }
                            Action::ApproveAll => {
                                for agent in &state.agents.root_agents {
                                    if agent.status.needs_attention() {
                                        if let Err(e) = tmux_client.send_keys(&agent.target, "y") {
                                            state.set_error(format!("Failed to approve {}: {}", agent.target, e));
                                            break;
                                        }
                                        if let Err(e) = tmux_client.send_keys(&agent.target, "Enter") {
                                            state.set_error(format!("Failed to send Enter to {}: {}", agent.target, e));
                                            break;
                                        }
                                    }
                                }
                            }
                            Action::FocusPane => {
                                if let Some(agent) = state.selected_agent() {
                                    let target = agent.target.clone();
                                    if let Err(e) = tmux_client.focus_pane(&target) {
                                        state.set_error(format!("Failed to focus: {}", e));
                                    }
                                }
                            }
                            Action::ToggleSubagentLog => {
                                state.toggle_subagent_log();
                            }
                            Action::Refresh => {
                                state.clear_error();
                            }
                            Action::ShowHelp => {
                                state.toggle_help();
                            }
                            Action::HideHelp => {
                                state.show_help = false;
                            }
                            Action::EnterInputMode => {
                                state.enter_input_mode();
                            }
                            Action::CancelInput => {
                                state.exit_input_mode();
                            }
                            Action::InputChar(c) => {
                                state.input_char(c);
                            }
                            Action::InputBackspace => {
                                state.input_backspace();
                            }
                            Action::SendInput => {
                                if let Some((input, target_idx)) = state.take_input() {
                                    if let Some(agent) = state.agents.get_agent(target_idx) {
                                        let target = agent.target.clone();
                                        // Send the input text
                                        if let Err(e) = tmux_client.send_keys(&target, &input) {
                                            state.set_error(format!("Failed to send input: {}", e));
                                        } else if let Err(e) = tmux_client.send_keys(&target, "Enter") {
                                            state.set_error(format!("Failed to send Enter: {}", e));
                                        }
                                    }
                                }
                            }
                            Action::SendNumber(num) => {
                                if let Some(agent) = state.selected_agent() {
                                    let target = agent.target.clone();
                                    let num_str = num.to_string();
                                    if let Err(e) = tmux_client.send_keys(&target, &num_str) {
                                        state.set_error(format!("Failed to send number: {}", e));
                                    } else if let Err(e) = tmux_client.send_keys(&target, "Enter") {
                                        state.set_error(format!("Failed to send Enter: {}", e));
                                    }
                                }
                            }
                            Action::SidebarWider => {
                                state.sidebar_width = (state.sidebar_width + 5).min(70);
                            }
                            Action::SidebarNarrower => {
                                state.sidebar_width = state.sidebar_width.saturating_sub(5).max(15);
                            }
                            Action::None => {}
                        }
                    }
                }
            }
        }

        if state.should_quit {
            break;
        }
    }

    Ok(())
}

fn map_key_to_action(code: KeyCode, modifiers: KeyModifiers, state: &AppState) -> Action {
    // If help is shown, any key closes it
    if state.show_help {
        return Action::HideHelp;
    }

    // If in input mode, handle input-specific keys
    if state.is_input_mode() {
        return match code {
            KeyCode::Esc => Action::CancelInput,
            KeyCode::Enter => Action::SendInput,
            KeyCode::Backspace => Action::InputBackspace,
            KeyCode::Char(c) => Action::InputChar(c),
            _ => Action::None,
        };
    }

    match code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,

        KeyCode::Char('j') | KeyCode::Down => Action::NextAgent,
        KeyCode::Char('k') | KeyCode::Up => Action::PrevAgent,
        KeyCode::Tab => Action::NextAgent,

        // Multi-selection
        KeyCode::Char(' ') => Action::ToggleSelection,
        KeyCode::Char('a') if modifiers.contains(KeyModifiers::CONTROL) => Action::SelectAll,

        // Approval
        KeyCode::Char('y') | KeyCode::Char('Y') => Action::Approve,
        KeyCode::Char('n') | KeyCode::Char('N') => Action::Reject,
        KeyCode::Char('a') | KeyCode::Char('A') => Action::ApproveAll,

        // Number keys for quick choice selection (1-9)
        KeyCode::Char(c @ '1'..='9') => {
            let num = c.to_digit(10).unwrap() as u8;
            Action::SendNumber(num)
        }

        // Focus pane with 'f'
        KeyCode::Char('f') | KeyCode::Char('F') => Action::FocusPane,

        // Enter input mode with 'i'
        KeyCode::Char('i') | KeyCode::Char('I') => Action::EnterInputMode,

        KeyCode::Char('s') | KeyCode::Char('S') => Action::ToggleSubagentLog,
        KeyCode::Char('r') => Action::Refresh,

        // Sidebar resize
        KeyCode::Char('<') | KeyCode::Left => Action::SidebarNarrower,
        KeyCode::Char('>') | KeyCode::Right => Action::SidebarWider,

        KeyCode::Char('h') | KeyCode::Char('?') => Action::ShowHelp,

        KeyCode::Esc => {
            if !state.selected_agents.is_empty() {
                Action::ClearSelection
            } else if state.show_subagent_log {
                Action::ToggleSubagentLog
            } else {
                Action::None
            }
        }

        _ => Action::None,
    }
}
