use std::io;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton,
        MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;

use crate::app::{Action, AppState, Config, KeyAction, NavAction};
use crate::monitor::{MonitorTask, SystemStatsCollector};
use crate::parsers::ParserRegistry;
use crate::tmux::TmuxClient;

use super::components::{
    AgentTreeWidget, FooterWidget, HeaderWidget, HelpWidget, InputWidget, PanePreviewWidget,
    PopupInputWidget, SubagentLogWidget,
};
use super::Layout;

/// Runs the main application loop
pub async fn run_app(config: Config) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize state
    let mut state = AppState::new(config.clone());

    // Create tmux client and parser registry
    let tmux_client = Arc::new(TmuxClient::from_config(&config));
    let parser_registry = Arc::new(ParserRegistry::with_config(&config));

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
        config.clone(),
    );
    let monitor_handle = tokio::spawn(async move {
        monitor.run().await;
    });

    // Create system stats collector
    let mut system_stats = SystemStatsCollector::new();

    // Main loop
    let result = run_loop(
        &mut terminal,
        &mut state,
        &mut rx,
        &tmux_client,
        &mut system_stats,
    )
    .await;

    // Cleanup
    monitor_handle.abort();
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut AppState,
    rx: &mut mpsc::Receiver<crate::monitor::MonitorUpdate>,
    tmux_client: &TmuxClient,
    system_stats: &mut SystemStatsCollector,
) -> Result<()> {
    loop {
        // Advance animation tick
        state.tick();

        // Update system stats
        system_stats.refresh();
        state.system_stats = system_stats.stats().clone();

        // Draw UI
        terminal.draw(|frame| {
            let size = frame.area();
            let main_chunks = Layout::main_layout(size);

            // Header
            HeaderWidget::render(frame, main_chunks[0], state);

            // Always show input widget at bottom of right column
            let input_height = InputWidget::calculate_height(state.get_input(), 6);

            if state.show_subagent_log {
                // With subagent log: sidebar | summary+preview+input | subagent_log
                let (left, preview, subagent_log) =
                    Layout::content_layout_with_log(main_chunks[1], state.sidebar_width);
                AgentTreeWidget::render(frame, left, state);

                // Split preview area for summary, preview, and input
                let preview_chunks = ratatui::layout::Layout::default()
                    .direction(ratatui::layout::Direction::Vertical)
                    .constraints([
                        ratatui::layout::Constraint::Length(15),
                        ratatui::layout::Constraint::Min(5),
                        ratatui::layout::Constraint::Length(input_height + 2),
                    ])
                    .split(preview);
                PanePreviewWidget::render_summary(frame, preview_chunks[0], state);
                PanePreviewWidget::render_detailed(frame, preview_chunks[1], state);
                InputWidget::render(frame, preview_chunks[2], state);
                SubagentLogWidget::render(frame, subagent_log, state);
            } else {
                // Normal: sidebar | summary+preview+input
                let (left, summary, preview, input_area) = Layout::content_layout_with_input(
                    main_chunks[1],
                    state.sidebar_width,
                    input_height,
                    state.show_summary_detail,
                );
                AgentTreeWidget::render(frame, left, state);
                if state.show_summary_detail {
                    PanePreviewWidget::render_summary(frame, summary, state);
                }
                PanePreviewWidget::render_detailed(frame, preview, state);
                InputWidget::render(frame, input_area, state);
            }

            // Footer
            FooterWidget::render(frame, main_chunks[2], state, &state.config);

            // Popup input (before help)
            if let Some(popup_state) = &state.popup_input {
                PopupInputWidget::render(frame, size, popup_state);
            }

            // Help overlay (highest priority - render last)
            if state.show_help {
                HelpWidget::render(frame, size, &state.config);
            }
        })?;

        // Handle events with short timeout for responsive UI (~60fps)
        let timeout = Duration::from_millis(16);

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

            // Handle keyboard and mouse events
            _ = tokio::time::sleep(timeout) => {
                // Process all pending events to avoid input lag
                while event::poll(Duration::from_millis(0))? {
                    let event = event::read()?;

                    // Handle mouse events
                    if let Event::Mouse(mouse) = event {
                        let size = terminal.size()?;
                        let area = ratatui::layout::Rect::new(0, 0, size.width, size.height);
                        let main_chunks = Layout::main_layout(area);
                        let footer_area = main_chunks[2];
                        let (sidebar, _, _, input_area) = Layout::content_layout_with_input(
                            main_chunks[1], state.sidebar_width, 3, state.show_summary_detail
                        );

                        match mouse.kind {
                            MouseEventKind::Down(MouseButton::Left) => {
                                let x = mouse.column;
                                let y = mouse.row;

                                // Check footer button clicks first
                                if let Some(button) = FooterWidget::hit_test(x, y, footer_area, state, &state.config) {
                                    use super::components::FooterButton;
                                    match button {
                                        FooterButton::Approve => {
                                            let indices = state.get_operation_indices();
                                            for idx in indices {
                                                if let Some(agent) = state.agents.get_agent(idx) {
                                                    if agent.status.needs_attention() {
                                                        let target = agent.target.clone();
                                                        let _ = tmux_client.send_keys(&target, "y");
                                                        let _ = tmux_client.send_keys(&target, "Enter");
                                                    }
                                                }
                                            }
                                            state.clear_selection();
                                        }
                                        FooterButton::Reject => {
                                            let indices = state.get_operation_indices();
                                            for idx in indices {
                                                if let Some(agent) = state.agents.get_agent(idx) {
                                                    if agent.status.needs_attention() {
                                                        let target = agent.target.clone();
                                                        let _ = tmux_client.send_keys(&target, "n");
                                                        let _ = tmux_client.send_keys(&target, "Enter");
                                                    }
                                                }
                                            }
                                            state.clear_selection();
                                        }
                                        FooterButton::ApproveAll => {
                                            for agent in &state.agents.root_agents {
                                                if agent.status.needs_attention() {
                                                    let _ = tmux_client.send_keys(&agent.target, "y");
                                                    let _ = tmux_client.send_keys(&agent.target, "Enter");
                                                }
                                            }
                                        }
                                        FooterButton::ToggleSelect => {
                                            state.toggle_selection();
                                        }
                                        FooterButton::Focus => {
                                            if let Some(agent) = state.selected_agent() {
                                                let target = agent.target.clone();
                                                let _ = tmux_client.focus_pane(&target);
                                            }
                                        }
                                        FooterButton::Help => {
                                            state.toggle_help();
                                        }
                                        FooterButton::Quit => {
                                            state.should_quit = true;
                                        }
                                    }
                                }
                                // Check if click is in sidebar - try to select agent
                                else if x >= sidebar.x && x < sidebar.x + sidebar.width
                                    && y >= sidebar.y && y < sidebar.y + sidebar.height
                                {
                                    state.focus_sidebar();
                                    // Calculate which agent was clicked based on row
                                    // Each agent takes ~4 lines in the tree view (varies)
                                    // Simple heuristic: use relative row position
                                    let rel_y = (y - sidebar.y).saturating_sub(1) as usize;
                                    let agents_count = state.agents.root_agents.len();
                                    if agents_count > 0 {
                                        // Estimate ~4 lines per agent (header + info + status)
                                        let estimated_idx = rel_y / 4;
                                        if estimated_idx < agents_count {
                                            state.select_agent(estimated_idx);
                                        }
                                    }
                                }
                                // Check if click is in input area
                                else if x >= input_area.x && x < input_area.x + input_area.width
                                    && y >= input_area.y && y < input_area.y + input_area.height
                                {
                                    state.focus_input();
                                }
                            }
                            MouseEventKind::ScrollUp => {
                                state.select_prev();
                            }
                            MouseEventKind::ScrollDown => {
                                state.select_next();
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // Handle keyboard events
                    if let Event::Key(key) = event {
                        let action = map_key_to_action(key.code, key.modifiers, state, &state.config);

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
                            Action::ToggleSummaryDetail => {
                                state.toggle_summary_detail();
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
                            Action::FocusInput => {
                                state.focus_input();
                            }
                            Action::FocusSidebar => {
                                state.focus_sidebar();
                            }
                            Action::ClearInput => {
                                state.take_input();
                            }
                            Action::InputChar(c) => {
                                state.input_char(c);
                            }
                            Action::InputNewline => {
                                state.input_newline();
                            }
                            Action::InputBackspace => {
                                state.input_backspace();
                            }
                            Action::CursorLeft => {
                                state.cursor_left();
                            }
                            Action::CursorRight => {
                                state.cursor_right();
                            }
                            Action::CursorHome => {
                                state.cursor_home();
                            }
                            Action::CursorEnd => {
                                state.cursor_end();
                            }
                            Action::SendInput => {
                                let input = state.take_input();
                                if !input.is_empty() {
                                    if let Some(agent) = state.selected_agent() {
                                        let target = agent.target.clone();
                                        // Send the input text
                                        if let Err(e) = tmux_client.send_keys(&target, &input) {
                                            state.set_error(format!("Failed to send input: {}", e));
                                        } else if let Err(e) = tmux_client.send_keys(&target, "Enter") {
                                            state.set_error(format!("Failed to send Enter: {}", e));
                                        }
                                    }
                                }
                                // Stay in input mode for consecutive inputs
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
                            Action::SelectAgent(idx) => {
                                state.select_agent(idx);
                            }
                            Action::ScrollUp => {
                                state.select_prev();
                            }
                            Action::ScrollDown => {
                                state.select_next();
                            }
                            Action::SendKeys(keys) => {
                                let indices = state.get_operation_indices();
                                for idx in indices {
                                    if let Some(agent) = state.agents.get_agent(idx) {
                                        let target = agent.target.clone();
                                        if let Err(e) = tmux_client.send_keys(&target, &keys) {
                                            state.set_error(format!("Failed to send keys: {}", e));
                                            break;
                                        }
                                    }
                                }
                                state.clear_selection();
                            }
                            Action::KillApp { method } => {
                                let indices = state.get_operation_indices();
                                for idx in indices {
                                    if let Some(agent) = state.agents.get_agent(idx) {
                                        let target = agent.target.clone();
                                        if let Err(e) = tmux_client.kill_application(&target, &method) {
                                            state.set_error(format!("Failed to kill app: {}", e));
                                            break;
                                        }
                                    }
                                }
                                state.clear_selection();
                            }
                            Action::ShowPopupInput {
                                title,
                                prompt,
                                initial,
                                popup_type,
                            } => {
                                use crate::app::PopupInputState;
                                state.popup_input = Some(PopupInputState {
                                    title,
                                    prompt,
                                    buffer: initial,
                                    cursor: 0,
                                    popup_type,
                                });
                            }
                            Action::HidePopupInput => {
                                state.popup_input = None;
                            }
                            Action::PopupInputSubmit => {
                                use crate::app::PopupType;
                                if let Some(popup) = state.popup_input.take() {
                                    match popup.popup_type {
                                        PopupType::Filter => {
                                            // Apply filter
                                            if popup.buffer.is_empty() {
                                                state.filter_pattern = None; // Clear filter
                                            } else {
                                                state.filter_pattern = Some(popup.buffer);
                                            }
                                            // Ensure selection points to visible agent and clean up selections
                                            state.ensure_visible_selection();
                                        }
                                        PopupType::GeneralInput => {
                                            // Send to selected agent
                                            let text = popup.buffer;
                                            if let Some(agent) = state.selected_agent() {
                                                if let Err(e) = tmux_client.send_keys(&agent.target, &text)
                                                {
                                                    state.set_error(format!("Failed to send input: {}", e));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Action::PopupInputChar(c) => {
                                if let Some(popup) = &mut state.popup_input {
                                    popup.buffer.insert(popup.cursor, c);
                                    popup.cursor += c.len_utf8();
                                }
                            }
                            Action::PopupInputBackspace => {
                                if let Some(popup) = &mut state.popup_input {
                                    if popup.cursor > 0 {
                                        let prev = popup.buffer[..popup.cursor]
                                            .char_indices()
                                            .last()
                                            .map(|(i, _)| i)
                                            .unwrap_or(0);
                                        popup.buffer.remove(prev);
                                        popup.cursor = prev;
                                    }
                                }
                            }
                            Action::PopupInputDelete => {
                                if let Some(popup) = &mut state.popup_input {
                                    if popup.cursor < popup.buffer.len() {
                                        if let Some(ch) = popup.buffer[popup.cursor..].chars().next() {
                                            popup.buffer.drain(popup.cursor..popup.cursor + ch.len_utf8());
                                        }
                                    }
                                }
                            }
                            Action::PopupInputClear => {
                                if let Some(popup) = &mut state.popup_input {
                                    popup.buffer.clear();
                                    popup.cursor = 0;
                                }
                            }
                            Action::PopupInputSelectAll => {
                                if let Some(popup) = &mut state.popup_input {
                                    popup.buffer.clear();
                                    popup.cursor = 0;
                                }
                            }
                            Action::PopupInputCursorLeft => {
                                if let Some(popup) = &mut state.popup_input {
                                    if popup.cursor > 0 {
                                        popup.cursor = popup.buffer[..popup.cursor]
                                            .char_indices()
                                            .last()
                                            .map(|(i, _)| i)
                                            .unwrap_or(0);
                                    }
                                }
                            }
                            Action::PopupInputCursorRight => {
                                if let Some(popup) = &mut state.popup_input {
                                    if popup.cursor < popup.buffer.len() {
                                        if let Some(c) = popup.buffer[popup.cursor..].chars().next() {
                                            popup.cursor += c.len_utf8();
                                        }
                                    }
                                }
                            }
                            Action::PopupInputCursorHome => {
                                if let Some(popup) = &mut state.popup_input {
                                    popup.cursor = 0;
                                }
                            }
                            Action::PopupInputCursorEnd => {
                                if let Some(popup) = &mut state.popup_input {
                                    popup.cursor = popup.buffer.len();
                                }
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

fn map_key_to_action(
    code: KeyCode,
    modifiers: KeyModifiers,
    state: &AppState,
    config: &Config,
) -> Action {
    // If help is shown, any key closes it
    if state.show_help {
        return Action::HideHelp;
    }

    // If popup is shown, intercept all keys
    if state.popup_input.is_some() {
        return match code {
            KeyCode::Enter => Action::PopupInputSubmit,
            KeyCode::Esc => Action::HidePopupInput,
            KeyCode::Char('u') if modifiers.contains(KeyModifiers::CONTROL) => {
                Action::PopupInputClear
            }
            KeyCode::Char('a') if modifiers.contains(KeyModifiers::CONTROL) => {
                Action::PopupInputSelectAll
            }
            KeyCode::Char(c) => Action::PopupInputChar(c),
            KeyCode::Backspace => Action::PopupInputBackspace,
            KeyCode::Delete => Action::PopupInputDelete,
            KeyCode::Left => Action::PopupInputCursorLeft,
            KeyCode::Right => Action::PopupInputCursorRight,
            KeyCode::Home => Action::PopupInputCursorHome,
            KeyCode::End => Action::PopupInputCursorEnd,
            _ => Action::None,
        };
    }

    // If input panel is focused, handle input-specific keys
    if state.is_input_focused() {
        return match code {
            // Esc moves focus back to sidebar
            KeyCode::Esc => Action::FocusSidebar,
            // Shift+Enter or Alt+Enter inserts newline
            KeyCode::Enter if modifiers.contains(KeyModifiers::SHIFT) => Action::InputNewline,
            KeyCode::Enter if modifiers.contains(KeyModifiers::ALT) => Action::InputNewline,
            KeyCode::Enter => Action::SendInput,
            KeyCode::Backspace => Action::InputBackspace,
            // Cursor movement
            KeyCode::Left => Action::CursorLeft,
            KeyCode::Right => Action::CursorRight,
            KeyCode::Home => Action::CursorHome,
            KeyCode::End => Action::CursorEnd,
            KeyCode::Char(c) => Action::InputChar(c),
            _ => Action::None,
        };
    }

    // Sidebar focused - check popup trigger key first
    if let KeyCode::Char(c) = code {
        let key_str = c.to_string();

        // Check popup trigger key
        if key_str == config.popup_trigger_key {
            return Action::ShowPopupInput {
                title: "Filter Agents".to_string(),
                prompt: "Pattern (name/session/window/dir):".to_string(),
                initial: state.filter_pattern.clone().unwrap_or_default(),
                popup_type: crate::app::PopupType::Filter,
            };
        }

        // Then check configured key bindings
        if let Some(action) = config.key_bindings.get_action(&key_str) {
            return match action {
                KeyAction::Navigate(NavAction::NextAgent) => Action::NextAgent,
                KeyAction::Navigate(NavAction::PrevAgent) => Action::PrevAgent,
                KeyAction::Approve => Action::Approve,
                KeyAction::Reject => Action::Reject,
                KeyAction::ApproveAll => Action::ApproveAll,
                KeyAction::SendNumber(n) => Action::SendNumber(*n),
                KeyAction::SendKeys(keys) => Action::SendKeys(keys.clone()),
                KeyAction::KillApp { method } => Action::KillApp {
                    method: method.clone(),
                },
            };
        }
    }

    // Arrow keys as fallback (always work even if j/k remapped)
    match code {
        KeyCode::Down => return Action::NextAgent,
        KeyCode::Up => return Action::PrevAgent,
        _ => {}
    }

    // Other keys remain hardcoded
    match code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,

        KeyCode::Tab => Action::NextAgent,

        // Left/Right arrows for focus navigation
        KeyCode::Right => Action::FocusInput,
        KeyCode::Left => Action::None, // Already on sidebar

        // Multi-selection
        KeyCode::Char(' ') => Action::ToggleSelection,
        KeyCode::Char('a') if modifiers.contains(KeyModifiers::CONTROL) => Action::SelectAll,

        // Focus pane with 'f'
        KeyCode::Char('f') | KeyCode::Char('F') => Action::FocusPane,

        KeyCode::Char('s') | KeyCode::Char('S') => Action::ToggleSubagentLog,
        KeyCode::Char('t') | KeyCode::Char('T') => Action::ToggleSummaryDetail,
        KeyCode::Char('r') => Action::Refresh,

        // Sidebar resize (only < and >)
        KeyCode::Char('<') => Action::SidebarNarrower,
        KeyCode::Char('>') => Action::SidebarWider,

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
