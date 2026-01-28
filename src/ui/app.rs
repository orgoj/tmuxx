use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
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
use tui_textarea::Input;

use crate::app::key_binding::CommandConfig;
use crate::app::{Action, AppState, Config, KeyAction, NavAction};
use crate::monitor::{MonitorTask, SystemStatsCollector};
use crate::parsers::ParserRegistry;
use crate::tmux::TmuxClient;

use super::components::{
    AgentTreeWidget, FooterWidget, HeaderWidget, InputWidget, MenuTreeWidget, ModalTextareaWidget,
    PanePreviewWidget, PopupInputWidget, SubagentLogWidget,
};
use super::Layout;

// Layout constants
const SUMMARY_HEIGHT: u16 = 15;
const PREVIEW_MIN_HEIGHT: u16 = 5;
const INPUT_BORDER_HEIGHT: u16 = 2;

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

    // Create shared flag for user interaction (notification reset)
    let user_interacted = Arc::new(AtomicBool::new(false));
    let user_interacted_clone = user_interacted.clone();

    // Start monitor task
    let monitor = MonitorTask::new(
        tmux_client.clone(),
        parser_registry.clone(),
        tx,
        Duration::from_millis(config.poll_interval_ms),
        config.clone(),
        user_interacted_clone,
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
        &user_interacted,
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
    user_interacted: &Arc<AtomicBool>,
) -> Result<()> {
    let mut needs_redraw = true;

    loop {
        // Advance animation tick
        let old_tick = state.tick;
        state.tick();
        if state.tick != old_tick {
            needs_redraw = true;
        }

        if needs_redraw {
            // Update system stats
            system_stats.refresh();
            state.system_stats = system_stats.stats().clone();

            // Draw UI
            terminal.draw(|frame| {
                let size = frame.area();
                let main_chunks = Layout::main_layout(size);

                // Header
                HeaderWidget::render(frame, main_chunks[0], state);

                // Calculate input height based on config
                let input_height = if state.config.hide_bottom_input {
                    0 // No input widget shown
                } else {
                    InputWidget::calculate_height(state.get_input(), 6)
                };

                if state.show_subagent_log {
                    // With subagent log: sidebar | summary+preview+input | subagent_log
                    let (left, preview, subagent_log) =
                        Layout::content_layout_with_log(main_chunks[1], &state.sidebar_width);
                    AgentTreeWidget::render(frame, left, state);

                    // Split preview area for summary, preview, and input
                    let preview_chunks = ratatui::layout::Layout::default()
                        .direction(ratatui::layout::Direction::Vertical)
                        .constraints([
                            ratatui::layout::Constraint::Length(SUMMARY_HEIGHT),
                            ratatui::layout::Constraint::Min(PREVIEW_MIN_HEIGHT),
                            ratatui::layout::Constraint::Length(input_height + INPUT_BORDER_HEIGHT),
                        ])
                        .split(preview);
                    if state.show_summary_detail {
                        PanePreviewWidget::render_summary(frame, preview_chunks[0], state);
                    }

                    // Only render detailed view if an agent is actually selected/visible
                    if state.selected_agent().is_some() {
                        PanePreviewWidget::render_detailed(frame, preview_chunks[1], state);
                    }

                    // Only render input if not hidden
                    if !state.config.hide_bottom_input {
                        InputWidget::render(frame, preview_chunks[2], state);
                    }
                    SubagentLogWidget::render(frame, subagent_log, state);
                } else {
                    // Normal: sidebar | summary+preview±input
                    if state.config.hide_bottom_input {
                        // No input panel at all
                        let (left, summary, preview) = Layout::content_layout_no_input(
                            main_chunks[1],
                            &state.sidebar_width,
                            state.show_summary_detail,
                        );
                        AgentTreeWidget::render(frame, left, state);
                        if state.show_summary_detail {
                            PanePreviewWidget::render_summary(frame, summary, state);
                        }
                        if state.selected_agent().is_some() {
                            PanePreviewWidget::render_detailed(frame, preview, state);
                        }
                    } else {
                        // With input panel
                        let (left, summary, preview, input_area) =
                            Layout::content_layout_with_input(
                                main_chunks[1],
                                &state.sidebar_width,
                                input_height,
                                state.show_summary_detail,
                            );
                        AgentTreeWidget::render(frame, left, state);
                        if state.show_summary_detail {
                            PanePreviewWidget::render_summary(frame, summary, state);
                        }
                        if state.selected_agent().is_some() {
                            PanePreviewWidget::render_detailed(frame, preview, state);
                        }
                        InputWidget::render(frame, input_area, state);
                    }
                }

                // Footer
                FooterWidget::render(frame, main_chunks[2], state, &state.config);

                // Popup input (before help)
                if let Some(popup_state) = &state.popup_input {
                    PopupInputWidget::render(frame, size, popup_state);
                }

                // Modal textarea (before help)
                if let Some(modal_state) = &state.modal_textarea {
                    ModalTextareaWidget::render(frame, size, modal_state);
                }

                // Menu Tree (before help)
                if state.show_menu {
                    MenuTreeWidget::render(
                        frame,
                        size,
                        &mut state.menu_tree,
                        &state.config.menu,
                        &state.config,
                        "Command Menu",
                    );
                }

                // Prompts Menu (before help)
                if state.show_prompts {
                    MenuTreeWidget::render(
                        frame,
                        size,
                        &mut state.prompts_tree,
                        &state.config.prompts,
                        &state.config,
                        "Prompts Menu",
                    );
                }

                // Help overlay (highest priority - render last)
                if state.show_help {
                    if let Some(modal_state) = &state.modal_textarea {
                        ModalTextareaWidget::render(frame, size, modal_state);
                    }
                }
            })?;
            needs_redraw = false;
        }

        // Handle events with short timeout for responsive UI (~60fps)
        let timeout = Duration::from_millis(16);

        tokio::select! {
            // Handle monitor updates
            Some(update) = rx.recv() => {
                state.agents = update.agents;
                // Sync selection based on agent IDs
                state.sync_selection();

                // Update cached visibility projection after agent list changes
                state.update_visible_indices();

                // Refresh TODO content for the newly selected/updated agent
                state.refresh_project_todo();
                needs_redraw = true;
            }

            // Handle keyboard and mouse events
            _ = tokio::time::sleep(timeout) => {
                // Process all pending events to avoid input lag
                while event::poll(Duration::from_millis(0))? {
                    let event = event::read()?;
                    needs_redraw = true;

                    // Signal user interaction to reset notification state
                    if matches!(event, Event::Key(_) | Event::Mouse(_)) {
                        user_interacted.store(true, Ordering::Relaxed);
                    }

                    // Handle mouse events
                    if let Event::Mouse(mouse) = event {
                        let size = terminal.size()?;
                        let area = ratatui::layout::Rect::new(0, 0, size.width, size.height);
                        let main_chunks = Layout::main_layout(area);
                        let _footer_area = main_chunks[2];
                        let (sidebar, _, _, input_area) = Layout::content_layout_with_input(
                            main_chunks[1], &state.sidebar_width, 3, state.show_summary_detail
                        );

                        match mouse.kind {
                            MouseEventKind::Down(MouseButton::Left) => {
                                let x = mouse.column;
                                let y = mouse.row;

                                    // Check if click is in sidebar - try to select agent
                                    if x >= sidebar.x && x < sidebar.x + sidebar.width
                                        && y >= sidebar.y && y < sidebar.y + sidebar.height
                                    {
                                        state.focus_sidebar();
                                        // Calculate which agent was clicked based on row
                                        // Use precise logic from AgentTreeWidget
                                        let rel_y = (y - sidebar.y).saturating_sub(1) as usize;
                                        let width = sidebar.width.saturating_sub(2) as usize; // inside borders

                                        if let Some(idx) = AgentTreeWidget::get_agent_index_at_row(rel_y, state, width) {
                                            state.select_agent(idx);
                                            state.refresh_project_todo();
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
                                state.refresh_project_todo();
                            }
                            MouseEventKind::ScrollDown => {
                                state.select_next();
                                state.refresh_project_todo();
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // Handle paste events
                    if let Event::Paste(data) = &event {
                        if let Some(modal) = &mut state.modal_textarea {
                            if !modal.readonly {
                                for line in data.lines() {
                                    modal.textarea.insert_str(line);
                                    modal.textarea.insert_newline();
                                }
                                // Remove the last extra newline if added (optional, but insert_newline adds one)
                                modal.textarea.delete_char();
                            }
                        } else if !state.config.hide_bottom_input {
                            // Also handle paste for main input widget if modal is not open
                            for c in data.chars() {
                                state.input_char(c);
                            }
                        }
                    }

                    // Handle keyboard events
                    if let Event::Key(key) = event {

                        // Special handling for modal textarea (both editable and readonly)
                        if state.modal_textarea.is_some() {
                            // Check for special keys first
                            let action = map_key_to_action(key.code, key.modifiers, state, &state.config);
                            state.log_action(&action);

                            match action {
                                Action::ModalTextareaSubmit => {
                                    if let Some(modal) = state.modal_textarea.take() {
                                        let text = modal.get_text();
                                        // Send text to selected agent
                                        if let Some(agent) = state.agents.get_agent(state.selected_index) {
                                            if let Err(e) = tmux_client.send_keys(&agent.target, &text) {
                                                state.set_error(format!("Failed to send input: {}", e));
                                            }
                                        }
                                    }
                                }
                                Action::HideModalTextarea => {
                                    state.modal_textarea = None;
                                }
                                Action::HideHelp => {
                                    state.show_help = false;
                                    state.modal_textarea = None;
                                }
                                _ => {
                                    // Pass all other keys to textarea using Into<Input> trait
                                    if let Some(modal) = &mut state.modal_textarea {
                                        let input: Input = key.into();
                                        let should_close = modal.handle_input(input);
                                        if should_close {
                                            state.modal_textarea = None;
                                        }
                                    }
                                }
                            }
                        } else if state.show_menu {
                            use crate::ui::components::menu_tree::{find_flat_menu_item_by_index, get_current_items_count};

                            match key.code {
                                KeyCode::Esc => {
                                    state.toggle_menu();
                                }
                                 KeyCode::Down | KeyCode::Char('j') if state.menu_tree.filter.is_empty() => {
                                     let count = get_current_items_count(&state.config.menu, &state.menu_tree);
                                     state.menu_tree.key_down(count);
                                 }
                                 KeyCode::Up | KeyCode::Char('k') if state.menu_tree.filter.is_empty() => {
                                     let count = get_current_items_count(&state.config.menu, &state.menu_tree);
                                     state.menu_tree.key_up(count);
                                 }
                                 KeyCode::Down => {
                                     let count = get_current_items_count(&state.config.menu, &state.menu_tree);
                                     state.menu_tree.key_down(count);
                                 }
                                 KeyCode::Up => {
                                     let count = get_current_items_count(&state.config.menu, &state.menu_tree);
                                     state.menu_tree.key_up(count);
                                 }
                                 KeyCode::Right | KeyCode::Char('l') if state.menu_tree.filter.is_empty() => {
                                      if let Some(index) = state.menu_tree.list_state.selected() {
                                          let path = find_flat_menu_item_by_index(&state.config.menu, &state.menu_tree, index)
                                              .filter(|f| !f.item.items.is_empty())
                                              .map(|f| f.path);
                                          if let Some(p) = path {
                                              state.menu_tree.toggle_expansion(p);
                                          }
                                      }
                                 }
                                 KeyCode::Char('*') => {
                                      state.menu_tree.expand_all = !state.menu_tree.expand_all;
                                 }
                                 KeyCode::Left | KeyCode::Char('h') if state.menu_tree.filter.is_empty() => {
                                      if let Some(index) = state.menu_tree.list_state.selected() {
                                          let res = find_flat_menu_item_by_index(&state.config.menu, &state.menu_tree, index)
                                              .map(|f| (f.path.clone(), state.menu_tree.expanded_paths.contains(&f.path)));

                                          if let Some((path, is_expanded)) = res {
                                              if is_expanded {
                                                  state.menu_tree.expanded_paths.remove(&path);
                                              } else if path.len() > 1 {
                                                  let parent_path = path[..path.len()-1].to_vec();
                                                  state.menu_tree.expanded_paths.remove(&parent_path);
                                              }
                                          }
                                      }
                                 }
                                 KeyCode::Right => {
                                      if let Some(index) = state.menu_tree.list_state.selected() {
                                          let path = find_flat_menu_item_by_index(&state.config.menu, &state.menu_tree, index)
                                              .filter(|f| !f.item.items.is_empty())
                                              .map(|f| f.path);
                                          if let Some(p) = path {
                                              state.menu_tree.expanded_paths.insert(p);
                                          }
                                      }
                                 }
                                 KeyCode::Left => {
                                      if let Some(index) = state.menu_tree.list_state.selected() {
                                          let path = find_flat_menu_item_by_index(&state.config.menu, &state.menu_tree, index)
                                              .map(|f| f.path);
                                          if let Some(p) = path {
                                              state.menu_tree.expanded_paths.remove(&p);
                                          }
                                      }
                                 }
                                 KeyCode::PageDown => {
                                    let count = get_current_items_count(&state.config.menu, &state.menu_tree);
                                    for _ in 0..10 { state.menu_tree.key_down(count); }
                                }
                                KeyCode::PageUp => {
                                    let count = get_current_items_count(&state.config.menu, &state.menu_tree);
                                    for _ in 0..10 { state.menu_tree.key_up(count); }
                                }
                                 KeyCode::Backspace => {
                                     if !state.menu_tree.filter.is_empty() {
                                         state.menu_tree.filter.pop();
                                         state.menu_tree.list_state.select(Some(0));
                                     }
                                 }

                                 KeyCode::Enter => {
                                     if let Some(index) = state.menu_tree.list_state.selected() {
                                         let (cmd, is_submenu, p) = if let Some(flat) = find_flat_menu_item_by_index(&state.config.menu, &state.menu_tree, index) {
                                              (flat.item.execute_command.clone(), !flat.item.items.is_empty(), flat.path)
                                         } else {
                                              (None, false, Vec::new())
                                         };

                                         if let Some(execute_command) = cmd {
                                              state.toggle_menu();

                                              // Expand variables
                                              let expanded = if let Some(agent) = state.selected_agent() {
                                                  expand_command_variables(&execute_command.command, agent)
                                              } else {
                                                  execute_command.command.clone()
                                              };

                                              let path = if let Some(agent) = state.selected_agent() {
                                                  agent.path.clone()
                                              } else {
                                                  String::new()
                                              };

                                              let action = Action::ExecuteCommand {
                                                  command: execute_command.command.clone(),
                                                  blocking: execute_command.blocking,
                                                  terminal: execute_command.terminal,
                                                  external_terminal: execute_command.external_terminal,
                                              };
                                              state.log_action(&action);

                                              if execute_command.external_terminal {
                                                   if let Some(wrapper) = &state.config.terminal_wrapper {
                                                       let wrapped = wrapper.replace("{cmd}", &expanded);
                                                       let mut cmd = tokio::process::Command::new("bash");
                                                       cmd.args(["-c", &wrapped])
                                                          .stdin(std::process::Stdio::null())
                                                          .stdout(std::process::Stdio::null())
                                                          .stderr(std::process::Stdio::null());

                                                       if !path.is_empty() {
                                                           cmd.current_dir(&path);
                                                       }

                                                       match cmd.spawn() {
                                                           Ok(_) => state.set_status(format!("External: {}", expanded)),
                                                           Err(e) => state.set_error(format!("Failed to spawn external terminal: {}", e)),
                                                       }
                                                   } else {
                                                       state.set_error("terminal_wrapper not configured".to_string());
                                                   }
                                              } else if execute_command.terminal {
                                                     // Suspend TUI
                                                     if let Err(e) = crossterm::terminal::disable_raw_mode() {
                                                         state.set_error(format!("Failed to disable raw mode: {}", e));
                                                     }
                                                     if let Err(e) = crossterm::execute!(
                                                         terminal.backend_mut(),
                                                         crossterm::terminal::LeaveAlternateScreen,
                                                         crossterm::event::DisableMouseCapture
                                                     ) {
                                                         state.set_error(format!("Failed to leave alternate screen: {}", e));
                                                     }
                                                     if let Err(e) = terminal.show_cursor() {
                                                         state.set_error(format!("Failed to show cursor: {}", e));
                                                     }

                                                     // Run command synchronously
                                                     let mut command = std::process::Command::new("bash");
                                                     command.args(["-c", &expanded])
                                                         .stdin(std::process::Stdio::inherit())
                                                         .stdout(std::process::Stdio::inherit())
                                                         .stderr(std::process::Stdio::inherit());

                                                     if !path.is_empty() {
                                                         command.current_dir(&path);
                                                     }

                                                     let _ = command.status();

                                                     // Restore TUI
                                                     if let Err(e) = crossterm::terminal::enable_raw_mode() {
                                                         state.set_error(format!("Failed to enable raw mode: {}", e));
                                                     }
                                                     if let Err(e) = crossterm::execute!(
                                                         terminal.backend_mut(),
                                                         crossterm::terminal::EnterAlternateScreen,
                                                         crossterm::event::EnableMouseCapture
                                                     ) {
                                                         state.set_error(format!("Failed to enter alternate screen: {}", e));
                                                     }
                                                     if let Err(e) = terminal.hide_cursor() {
                                                         state.set_error(format!("Failed to hide cursor: {}", e));
                                                     }
                                                     if let Err(e) = terminal.clear() {
                                                         state.set_error(format!("Failed to clear terminal: {}", e));
                                                     }
                                              } else if execute_command.blocking {
                                                     let mut cmd = tokio::process::Command::new("bash");
                                                     cmd.args(["-c", &expanded]);
                                                     if !path.is_empty() {
                                                         cmd.current_dir(&path);
                                                     }
                                                     match cmd.output().await {
                                                         Ok(output) => {
                                                              if output.status.success() {
                                                                   state.set_status(format!("Executed: {}", expanded));
                                                              } else {
                                                                   state.set_error(format!("Failed: {}", expanded));
                                                              }
                                                         }
                                                         Err(e) => state.set_error(format!("Failed to execute: {}", e)),
                                                     }
                                              } else {
                                                   // Background spawn
                                                   let mut cmd = tokio::process::Command::new("bash");
                                                   cmd.args(["-c", &expanded])
                                                      .stdin(std::process::Stdio::null())
                                                      .stdout(std::process::Stdio::null())
                                                      .stderr(std::process::Stdio::null());
                                                   if !path.is_empty() {
                                                       cmd.current_dir(&path);
                                                   }
                                                   let _ = cmd.spawn();
                                                   state.set_status(format!("Started: {}", expanded));
                                              }
                                         } else if is_submenu {
                                              state.menu_tree.toggle_expansion(p);
                                         }
                                     }
                                 }

                                KeyCode::Char(c) => {
                                    state.menu_tree.filter.push(c);
                                    state.menu_tree.list_state.select(Some(0)); // Reset to top
                                }
                                _ => {}
                            }
                        } else if state.show_prompts {
                            use crate::ui::components::menu_tree::{find_flat_menu_item_by_index, get_current_items_count};
                            use crate::ui::components::ModalTextareaState;

                            match key.code {
                                KeyCode::Esc => {
                                    state.toggle_prompts();
                                }
                                 KeyCode::Down | KeyCode::Char('j') if state.prompts_tree.filter.is_empty() => {
                                     let count = get_current_items_count(&state.config.prompts, &state.prompts_tree);
                                     state.prompts_tree.key_down(count);
                                 }
                                 KeyCode::Up | KeyCode::Char('k') if state.prompts_tree.filter.is_empty() => {
                                     let count = get_current_items_count(&state.config.prompts, &state.prompts_tree);
                                     state.prompts_tree.key_up(count);
                                 }
                                 KeyCode::Down => {
                                     let count = get_current_items_count(&state.config.prompts, &state.prompts_tree);
                                     state.prompts_tree.key_down(count);
                                 }
                                 KeyCode::Up => {
                                     let count = get_current_items_count(&state.config.prompts, &state.prompts_tree);
                                     state.prompts_tree.key_up(count);
                                 }
                                 KeyCode::Right | KeyCode::Char('l') if state.prompts_tree.filter.is_empty() => {
                                      if let Some(index) = state.prompts_tree.list_state.selected() {
                                          let path = find_flat_menu_item_by_index(&state.config.prompts, &state.prompts_tree, index)
                                              .filter(|f| !f.item.items.is_empty())
                                              .map(|f| f.path);
                                          if let Some(p) = path {
                                              state.prompts_tree.toggle_expansion(p);
                                          }
                                      }
                                 }
                                 KeyCode::Char('*') => {
                                      state.prompts_tree.expand_all = !state.prompts_tree.expand_all;
                                 }
                                 KeyCode::Left | KeyCode::Char('h') if state.prompts_tree.filter.is_empty() => {
                                      if let Some(index) = state.prompts_tree.list_state.selected() {
                                          let res = find_flat_menu_item_by_index(&state.config.prompts, &state.prompts_tree, index)
                                              .map(|f| (f.path.clone(), state.prompts_tree.expanded_paths.contains(&f.path)));

                                          if let Some((path, is_expanded)) = res {
                                              if is_expanded {
                                                  state.prompts_tree.expanded_paths.remove(&path);
                                              } else if path.len() > 1 {
                                                  let parent_path = path[..path.len()-1].to_vec();
                                                  state.prompts_tree.expanded_paths.remove(&parent_path);
                                              }
                                          }
                                      }
                                 }
                                 KeyCode::Right => {
                                      if let Some(index) = state.prompts_tree.list_state.selected() {
                                          let path = find_flat_menu_item_by_index(&state.config.prompts, &state.prompts_tree, index)
                                              .filter(|f| !f.item.items.is_empty())
                                              .map(|f| f.path);
                                          if let Some(p) = path {
                                              state.prompts_tree.expanded_paths.insert(p);
                                          }
                                      }
                                 }
                                 KeyCode::Left => {
                                      if let Some(index) = state.prompts_tree.list_state.selected() {
                                          let path = find_flat_menu_item_by_index(&state.config.prompts, &state.prompts_tree, index)
                                              .map(|f| f.path);
                                          if let Some(p) = path {
                                              state.prompts_tree.expanded_paths.remove(&p);
                                          }
                                      }
                                 }
                                 KeyCode::PageDown => {
                                    let count = get_current_items_count(&state.config.prompts, &state.prompts_tree);
                                    for _ in 0..10 { state.prompts_tree.key_down(count); }
                                }
                                KeyCode::PageUp => {
                                    let count = get_current_items_count(&state.config.prompts, &state.prompts_tree);
                                    for _ in 0..10 { state.prompts_tree.key_up(count); }
                                }
                                 KeyCode::Backspace => {
                                     if !state.prompts_tree.filter.is_empty() {
                                         state.prompts_tree.filter.pop();
                                         state.prompts_tree.list_state.select(Some(0));
                                     }
                                 }

                                 KeyCode::Enter => {
                                     let selection = if let Some(index) = state.prompts_tree.list_state.selected() {
                                         find_flat_menu_item_by_index(&state.config.prompts, &state.prompts_tree, index)
                                             .map(|flat| (flat.item.text.clone(), flat.item.name.clone(), flat.path.clone(), !flat.item.items.is_empty()))
                                     } else {
                                         None
                                     };

                                     if let Some((text_opt, name, path, is_submenu)) = selection {
                                         if let Some(text) = text_opt {
                                                state.toggle_prompts();

                                                if key.modifiers.contains(KeyModifiers::ALT) {
                                                    // Open in ModalTextarea
                                                    state.modal_textarea = Some(ModalTextareaState::new(
                                                        format!("Edit Prompt: {}", name),
                                                        "Verify/Edit before sending".to_string(),
                                                        text.clone(),
                                                        false, // multiline
                                                        false, // editable
                                                    ));
                                                } else {
                                                    // Send directly
                                                    if let Some(agent) = state.selected_agent() {
                                                        if let Err(e) = tmux_client.send_keys_many(
                                                            &agent.target,
                                                            &[&text, "Enter"],
                                                        ) {
                                                            state.set_error(format!(
                                                                "Failed to send text: {}",
                                                                e
                                                            ));
                                                        } else {
                                                            state.set_status(format!("Sent: {}", name));
                                                        }
                                                    }
                                                }
                                         } else if is_submenu {
                                             state.prompts_tree.toggle_expansion(path);
                                         }
                                     }
                                 }

                                KeyCode::Char(c) => {
                                    state.prompts_tree.filter.push(c);
                                    state.prompts_tree.list_state.select(Some(0)); // Reset to top
                                }
                                _ => {}
                            }
                        } else {
                            // Normal key handling when modal is not active
                            let action = map_key_to_action(key.code, key.modifiers, state, &state.config);
                            state.log_action(&action);

                            match action {
                                Action::Quit => {
                                    state.should_quit = true;
                                }
                                Action::NextAgent => {
                                    state.select_next();
                                    state.refresh_project_todo();
                                }
                                Action::PrevAgent => {
                                    state.select_prev();
                                    state.refresh_project_todo();
                                }
                                Action::FirstAgent => {
                                    state.select_first();
                                    state.refresh_project_todo();
                                }
                                Action::LastAgent => {
                                    state.select_last();
                                    state.refresh_project_todo();
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
                                                if let Err(e) =
                                                    tmux_client.send_keys_many(&target, &["y", "Enter"])
                                                {
                                                    state.set_error(format!("Failed to approve: {}", e));
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                                Action::Reject => {
                                    let indices = state.get_operation_indices();
                                    for idx in indices {
                                        if let Some(agent) = state.agents.get_agent(idx) {
                                            if agent.status.needs_attention() {
                                                let target = agent.target.clone();
                                                if let Err(e) =
                                                    tmux_client.send_keys_many(&target, &["n", "Enter"])
                                                {
                                                    state.set_error(format!("Failed to reject: {}", e));
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                                Action::ApproveAll => {
                                    for agent in &state.agents.root_agents {
                                        if agent.status.needs_attention() {
                                            if let Err(e) =
                                                tmux_client.send_keys_many(&agent.target, &["y", "Enter"])
                                            {
                                                state.set_error(format!(
                                                    "Failed to approve {}: {}",
                                                    agent.target, e
                                                ));
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
                                Action::ToggleMenu => {
                                    state.toggle_menu();
                                }
                                Action::TogglePrompts => {
                                    state.toggle_prompts();
                                }
                                Action::Refresh => {
                                    state.clear_error();
                                    if let Err(e) = terminal.clear() {
                                        state.set_error(format!("Failed to clear screen: {}", e));
                                    }
                                }
                                Action::ShowHelp => {
                                    state.toggle_help();
                                }
                                Action::HideHelp => {
                                    state.show_help = false;
                                }
                                Action::FocusInput => {
                                    // Only allow input focus when input panel is visible
                                    if !state.config.hide_bottom_input {
                                        state.focus_input();
                                    }
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
                                    if let Some(agent) = state.selected_agent() {
                                        let target = agent.target.clone();
                                        let res = if input.is_empty() {
                                            tmux_client.send_keys(&target, "Enter")
                                        } else {
                                            tmux_client.send_keys_many(&target, &[&input, "Enter"])
                                        };
                                        if let Err(e) = res {
                                            state.set_error(format!("Failed to send input: {}", e));
                                        }
                                    }
                                }
                                Action::SendNumber(num) => {
                                    if let Some(agent) = state.selected_agent() {
                                        let target = agent.target.clone();
                                        let num_str = num.to_string();
                                        if let Err(e) =
                                            tmux_client.send_keys_many(&target, &[&num_str, "Enter"])
                                        {
                                            state.set_error(format!("Failed to send number: {}", e));
                                        }
                                    }
                                }
                                Action::SidebarWider => {
                                    state.sidebar_width.wider();
                                }
                                Action::SidebarNarrower => {
                                    state.sidebar_width.narrower();
                                }
                                Action::SelectAgent(idx) => {
                                    state.select_agent(idx);
                                }
                                Action::ScrollUp => {
                                    state.select_prev();
                                    state.refresh_project_todo();
                                }
                                Action::ScrollDown => {
                                    state.select_next();
                                    state.refresh_project_todo();
                                }
                                Action::SendKeys(keys) => {
                                    let indices = state.get_operation_indices();
                                    for idx in indices {
                                        if let Some(agent) = state.agents.get_agent(idx) {
                                            let target = agent.target.clone();
                                            if let Err(e) = tmux_client.send_keys(&target, &keys) {
                                                state.set_error(format!("Failed to send keys: {}", e));
                                                break;
                                            } else {
                                                state.set_status(format!("Sent keys to {}: {}", target, keys));
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
                                Action::KillSession => {
                                    if let Some(agent) = state.selected_agent() {
                                        Action::ShowPopupInput {
                                            title: "Confirm Kill Session".to_string(),
                                            prompt: format!("Are you sure you want to kill session '{}'? (y/n)", agent.session),
                                            initial: String::new(), // Empty buffer expects y/n
                                            popup_type: crate::app::PopupType::KillConfirmation {
                                                session: agent.session.clone(),
                                            },
                                        };
                                        // We need to trigger the action immediately to show popup
                                        // But Action::KillSession is just a trigger.
                                        // Actually, we should dispatch ShowPopupInput from here.
                                        state.popup_input = Some(crate::app::PopupInputState {
                                            title: "Confirm Kill Session".to_string(),
                                            prompt: format!("Kill session '{}'? Type 'y' to confirm.", agent.session),
                                            buffer: String::new(),
                                            cursor: 0,
                                            popup_type: crate::app::PopupType::KillConfirmation {
                                                session: agent.session.clone(),
                                            },
                                        });
                                    }
                                }
                                Action::ExecuteCommand {
                                    command,
                                    blocking,
                                    terminal: is_terminal,
                                    external_terminal,
                                } => {
                                    if let Some(agent) = state.selected_agent() {
                                        let expanded = expand_command_variables(&command, agent);
                                        let path = agent.path.clone();

                                        // Case 1: External Terminal (wrapper)
                                        if external_terminal {
                                            if let Some(wrapper) = &state.config.terminal_wrapper {
                                                let wrapped = wrapper.replace("{cmd}", &expanded);
                                                let mut cmd = tokio::process::Command::new("bash");
                                                cmd.args(["-c", &wrapped])
                                                   .stdin(std::process::Stdio::null())
                                                   .stdout(std::process::Stdio::null())
                                                   .stderr(std::process::Stdio::null());

                                                if !path.is_empty() {
                                                    cmd.current_dir(&path);
                                                }

                                                match cmd.spawn() {
                                                    Ok(_) => state.set_status(format!("External: {}", expanded)),
                                                    Err(e) => state.set_error(format!("Failed to spawn external terminal: {}", e)),
                                                }
                                            } else {
                                                state.set_error("terminal_wrapper not configured".to_string());
                                            }
                                        }
                                        // Case 2: Terminal application (interactive, takes over screen)
                                        else if is_terminal {
                                            // Suspend TUI
                                            if let Err(e) = disable_raw_mode() {
                                                state.set_error(format!("Failed to disable raw mode: {}", e));
                                            }
                                            if let Err(e) = execute!(
                                                terminal.backend_mut(),
                                                LeaveAlternateScreen,
                                                DisableMouseCapture
                                            ) {
                                                state
                                                    .set_error(format!("Failed to leave alternate screen: {}", e));
                                            }
                                            if let Err(e) = terminal.show_cursor() {
                                                state.set_error(format!("Failed to show cursor: {}", e));
                                            }

                                            // Run command synchronously with inherited stdio
                                            let mut command = std::process::Command::new("bash");
                                            command.args(["-c", &expanded])
                                                .stdin(std::process::Stdio::inherit())
                                                .stdout(std::process::Stdio::inherit())
                                                .stderr(std::process::Stdio::inherit());

                                            if !path.is_empty() {
                                                command.current_dir(&path);
                                            }

                                            let result = command.status();

                                            // Restore TUI
                                            if let Err(e) = enable_raw_mode() {
                                                state.set_error(format!("Failed to enable raw mode: {}", e));
                                            }
                                            if let Err(e) = execute!(
                                                terminal.backend_mut(),
                                                EnterAlternateScreen,
                                                EnableMouseCapture
                                            ) {
                                                state
                                                    .set_error(format!("Failed to enter alternate screen: {}", e));
                                            }
                                            if let Err(e) = terminal.hide_cursor() {
                                                state.set_error(format!("Failed to hide cursor: {}", e));
                                            }
                                            if let Err(e) = terminal.clear() {
                                                state.set_error(format!("Failed to clear terminal: {}", e));
                                            }

                                            match result {
                                                Ok(status) => {
                                                    if status.success() {
                                                        state.set_status(format!("Executed: {}", expanded));
                                                    } else {
                                                        state.set_error(format!(
                                                            "Command failed: {} (exit code: {})",
                                                            expanded,
                                                            status.code().unwrap_or(-1)
                                                        ));
                                                    }
                                                }
                                                Err(e) => {
                                                    state.set_error(format!("Failed to execute: {}", e))
                                                }
                                            }
                                        }
                                        // Case 2: Blocking background command (waits for output)
                                        else if blocking {
                                            use std::io::Write;
                                            let debug_mode = state.config.debug_mode;

                                            let mut cmd = tokio::process::Command::new("bash");
                                            cmd.args(["-c", &expanded]);

                                            if !path.is_empty() {
                                                cmd.current_dir(&path);
                                            }

                                            let result = cmd.output()
                                                .await
                                                .map(|output| {
                                                    // Helper to log if debug enabled
                                                    if debug_mode {
                                                         if let Ok(mut file) = std::fs::OpenOptions::new()
                                                            .create(true)
                                                            .append(true)
                                                            .open(".tmuxx.log")
                                                        {
                                                            let _ = writeln!(file, "--- Command: {} ---", expanded);
                                                            let _ = file.write_all(&output.stdout);
                                                            let _ = file.write_all(&output.stderr);
                                                            let _ = writeln!(file, "-------------------");
                                                        }
                                                    }

                                                    if output.status.success() {
                                                        let stdout = String::from_utf8_lossy(&output.stdout)
                                                            .to_string();
                                                        if stdout.is_empty() {
                                                            format!("Executed: {}", expanded)
                                                        } else {
                                                            format!(
                                                                "Executed: {} → {}",
                                                                expanded,
                                                                stdout.trim()
                                                            )
                                                        }
                                                    } else {
                                                        let stderr = String::from_utf8_lossy(&output.stderr)
                                                            .to_string();
                                                        format!(
                                                            "Failed: {} | Error: {}",
                                                            expanded,
                                                            stderr.trim()
                                                        )
                                                    }
                                                })
                                                .map_err(|e| format!("Failed to execute: {}", e));

                                            match result {
                                                Ok(msg) => state.set_status(msg),
                                                Err(e) => state.set_error(e),
                                            }
                                        }
                                        // Case 3: Non-blocking background command (fire and forget)
                                        else {
                                            let debug_mode = state.config.debug_mode;

                                            // Non-blocking (async) - prevent output to screen UNLESS debug
                                            // If debug, redirect to log. If not debug, null.
                                            let mut cmd = tokio::process::Command::new("bash");
                                            cmd.args(["-c", &expanded])
                                                .stdin(std::process::Stdio::null())
                                                .stdout(get_log_stdio(debug_mode))
                                                .stderr(get_log_stdio(debug_mode));

                                            if !path.is_empty() {
                                                cmd.current_dir(&path);
                                            }

                                            let result = cmd.spawn()
                                                .map(|_| format!("Started: {}", expanded))
                                                .map_err(|e| format!("Failed to spawn: {}", e));

                                            match result {
                                                Ok(msg) => state.set_status(msg),
                                                Err(e) => state.set_error(e),
                                            }
                                        }
                                    }
                                }
                                Action::CaptureTestCase => {
                                    use crate::app::{PopupInputState, PopupType};
                                    if let Some(agent) = state.selected_agent() {
                                        let content = agent.last_content.clone();
                                        state.popup_input = Some(PopupInputState {
                                            title: "Capture Test Case".to_string(),
                                            prompt: "Expected Status (idle, working, error, approval):".to_string(),
                                            buffer: String::new(),
                                            cursor: 0,
                                            popup_type: PopupType::CaptureStatus { content },
                                        });
                                    } else {
                                        state.set_error("No agent selected".to_string());
                                    }
                                }
                                Action::ShowPopupInput {
                                    title,
                                    prompt,
                                    initial,
                                    popup_type,
                                } => {
                                    use crate::app::PopupInputState;
                                    // Set cursor at end of buffer for rename dialog (easier to edit)
                                    let cursor = initial.len();
                                    state.popup_input = Some(PopupInputState {
                                        title,
                                        prompt,
                                        buffer: initial,
                                        cursor,
                                        popup_type,
                                    });
                                }
                                Action::ShowModalTextarea {
                                    title,
                                    prompt,
                                    initial,
                                    single_line,
                                } => {
                                    use crate::ui::components::ModalTextareaState;
                                    state.modal_textarea = Some(ModalTextareaState::new(
                                        title, prompt, initial, single_line, false, // not readonly
                                    ));
                                }
                                Action::TogglePaneTreeMode => {
                                    let new_mode = if state.config.pane_tree.mode == "compact" {
                                        "full"
                                    } else {
                                        "compact"
                                    };
                                    state.config.pane_tree.mode = new_mode.to_string();
                                    state.set_status(format!("Switched to {} view", new_mode));
                                }
                                Action::ToggleFilterActive => {
                                    state.toggle_filter_active();
                                    if state.filter_active {
                                        state.set_status("Showing active (non-idle) agents only".to_string());
                                    } else {
                                        state.set_status("Showing all agents".to_string());
                                    }
                                }
                                Action::ToggleFilterSelected => {
                                    state.toggle_filter_selected();
                                    if state.filter_selected {
                                        state.set_status("Showing selected agents only".to_string());
                                    } else {
                                        state.set_status("Showing all agents".to_string());
                                    }
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
                                                    state.set_filter_pattern(None);
                                                } else {
                                                    state.set_filter_pattern(Some(popup.buffer));
                                                }
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
                                            PopupType::RenameSession { session } => {
                                                let new_name = popup.buffer.trim();
                                                if new_name.is_empty() {
                                                    state.set_error("Session name cannot be empty".to_string());
                                                } else if new_name.contains('.') || new_name.contains(':') {
                                                    state.set_error("Session name cannot contain '.' or ':'".to_string());
                                                } else if new_name != session {
                                                    if let Err(e) = tmux_client.rename_session(&session, new_name) {
                                                        state.set_error(format!("Failed to rename session: {}", e));
                                                    }
                                                }
                                                // If new_name == session, just close dialog silently
                                            }
                                            PopupType::KillConfirmation { session } => {
                                                if popup.buffer.trim().eq_ignore_ascii_case("y") {
                                                    if let Err(e) = tmux_client.kill_session(&session) {
                                                        state.set_error(format!("Failed to kill session: {}", e));
                                                    } else {
                                                        state.set_status(format!("Killed session: {}", session));
                                                        // Note: The monitor loop will update the tree shortly
                                                    }
                                                }
                                            }
                                            PopupType::CaptureStatus { content } => {
                                                let status_str = popup.buffer.trim().to_lowercase();
                                                if status_str.is_empty() {
                                                    state.set_error("Status cannot be empty".to_string());
                                                } else if let Some(agent) = state.selected_agent() {
                                                    // Generate filename: case_<STATUS>_<TIMESTAMP>.txt
                                                    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                                                    // Clean status string for filename
                                                    let safe_status = status_str.replace(|c: char| !c.is_alphanumeric(), "_");
                                                    let filename = format!("case_{}_{}.txt", safe_status, timestamp);

                                                    // Determine directory: tests/fixtures/{agent_config_id}
                                                    // Use agent config ID (from config) so that multiple panes of same agent type go to same folder
                                                    let safe_name = agent.config_id.to_lowercase().replace(|c: char| !c.is_alphanumeric() && c != '-', "_");
                                                    let dir_name = if safe_name.is_empty() {
                                                        "unknown"
                                                    } else {
                                                        &safe_name
                                                    };

                                                    let mut path = std::path::PathBuf::from("tests/fixtures");
                                                    path.push(dir_name);

                                                    // Ensure directory exists
                                                    if let Err(e) = std::fs::create_dir_all(&path) {
                                                        state.set_error(format!("Failed to create directory: {}", e));
                                                    } else {
                                                        path.push(filename);

                                                        // Write content (from captured snapshot)
                                                        if let Err(e) = std::fs::write(&path, &content) {
                                                            state.set_error(format!("Failed to write test case: {}", e));
                                                        } else {
                                                            state.set_status(format!("Captured test case: {}", path.display()));
                                                        }
                                                    }
                                                } else {
                                                    state.set_error("No agent selected".to_string());
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
                                Action::HideModalTextarea => {
                                    // This should not happen here (handled in modal textarea mode)
                                }
                                Action::ModalTextareaSubmit => {
                                    // This should not happen here (handled in modal textarea mode)
                                }
                                Action::ReloadConfig => {
                                    match Config::try_load_merged() {
                                        Ok(new_config) => {
                                            state.reload_config(new_config);
                                        }
                                        Err(e) => {
                                            state.set_error(format!("Reload failed: {}", e));
                                        }
                                    }
                                }
                                Action::None => {}
                            }
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
    // If help is shown, handle scrolling or close
    if state.show_help {
        return match code {
            KeyCode::Esc => Action::HideHelp,
            KeyCode::Up
            | KeyCode::Down
            | KeyCode::PageUp
            | KeyCode::PageDown
            | KeyCode::Left
            | KeyCode::Right
            | KeyCode::Home
            | KeyCode::End => Action::None,
            _ => Action::HideHelp, // Any other key closes help
        };
    }

    // If modal textarea is shown, only handle special keys
    if let Some(modal) = &state.modal_textarea {
        return match code {
            KeyCode::Esc => Action::HideModalTextarea,
            KeyCode::Enter if modal.is_single_line => Action::ModalTextareaSubmit,
            KeyCode::Enter if modifiers.contains(KeyModifiers::ALT) => Action::ModalTextareaSubmit,
            _ => Action::None, // All other keys handled directly in event loop
        };
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
    let key_str = match code {
        KeyCode::Char(c) => {
            let ctrl = modifiers.contains(KeyModifiers::CONTROL);
            let alt = modifiers.contains(KeyModifiers::ALT);
            let shift = modifiers.contains(KeyModifiers::SHIFT);
            let is_uppercase = c.is_ascii_uppercase();
            let base_lowercase = c.to_ascii_lowercase();

            match (ctrl, alt, shift, is_uppercase) {
                (true, _, _, _) => format!("C-{}", base_lowercase),
                (false, true, true, _) => format!("M-S-{}", base_lowercase),
                (false, true, false, true) => format!("M-{}", base_lowercase),
                (false, true, false, false) => format!("M-{}", base_lowercase),
                (false, false, true, _) => c.to_uppercase().to_string(),
                (false, false, false, true) => c.to_string(),
                (false, false, false, false) => c.to_string(),
            }
        }
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::BackTab => "BackTab".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Insert => "Insert".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        KeyCode::F(n) => format!("F{}", n),
        _ => String::new(),
    };

    if !key_str.is_empty() {
        // Check popup trigger key (only for unmodified keys)
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
                KeyAction::Navigate(NavAction::FirstAgent) => Action::FirstAgent,
                KeyAction::Navigate(NavAction::LastAgent) => Action::LastAgent,
                KeyAction::Approve => Action::Approve,
                KeyAction::Reject => Action::Reject,
                KeyAction::ApproveAll => Action::ApproveAll,
                KeyAction::SendNumber(n) => Action::SendNumber(*n),
                KeyAction::SendKeys(keys) => Action::SendKeys(keys.clone()),
                KeyAction::KillApp { method } => Action::KillApp {
                    method: method.clone(),
                },
                KeyAction::KillSession => Action::KillSession,
                KeyAction::RenameSession => {
                    if let Some(agent) = state.selected_agent() {
                        Action::ShowPopupInput {
                            title: "Rename Session".to_string(),
                            prompt: "New session name:".to_string(),
                            initial: agent.session.clone(),
                            popup_type: crate::app::PopupType::RenameSession {
                                session: agent.session.clone(),
                            },
                        }
                    } else {
                        Action::None
                    }
                }
                KeyAction::Refresh => Action::Refresh,
                KeyAction::ExecuteCommand(CommandConfig {
                    command,
                    blocking,
                    terminal,
                    external_terminal,
                }) => Action::ExecuteCommand {
                    command: command.clone(),
                    blocking: *blocking,
                    terminal: *terminal,
                    external_terminal: *external_terminal,
                },
                KeyAction::ToggleMenu => Action::ToggleMenu,
                KeyAction::TogglePrompts => Action::TogglePrompts,
                KeyAction::ToggleSubagentLog => Action::ToggleSubagentLog,
                KeyAction::CaptureTestCase => Action::CaptureTestCase,
                KeyAction::TogglePaneTreeMode => Action::TogglePaneTreeMode,
                KeyAction::ToggleFilterActive => Action::ToggleFilterActive,
                KeyAction::ToggleFilterSelected => Action::ToggleFilterSelected,
                KeyAction::ReloadConfig => Action::ReloadConfig,
            };
        }
    }

    // Arrow keys as fallback (always work even if j/k remapped)
    match code {
        KeyCode::Down => return Action::NextAgent,
        KeyCode::Up => return Action::PrevAgent,
        KeyCode::Home => return Action::FirstAgent,
        KeyCode::End => return Action::LastAgent,
        _ => {}
    }

    // Other keys remain hardcoded
    match code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,

        KeyCode::Tab => Action::NextAgent,

        // Left/Right arrows for focus navigation
        KeyCode::Right => {
            // Only allow input focus when input panel is visible
            if config.hide_bottom_input {
                Action::None
            } else {
                Action::FocusInput
            }
        }
        KeyCode::Left => Action::None, // Already on sidebar

        // Multi-selection
        KeyCode::Char(' ') => Action::ToggleSelection,
        KeyCode::Char('a') if modifiers.contains(KeyModifiers::CONTROL) => Action::SelectAll,

        // Focus pane with 'f'
        KeyCode::Char('f') | KeyCode::Char('F') => Action::FocusPane,

        KeyCode::Char('t') | KeyCode::Char('T') => Action::ToggleSummaryDetail,

        // Sidebar resize (only < and >)
        KeyCode::Char('<') => Action::SidebarNarrower,
        KeyCode::Char('>') => Action::SidebarWider,

        KeyCode::Char('h') | KeyCode::Char('?') => Action::ShowHelp,

        // Modal textarea input (Shift+I)
        KeyCode::Char('I') => Action::ShowModalTextarea {
            title: "Multi-line Input".to_string(),
            prompt: "Enter message to agent (Alt+Enter to submit, Esc to cancel)".to_string(),
            initial: String::new(),
            single_line: false,
        },

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

/// Expand variables in a command template using agent context
///
/// Supported variables:
/// - `${SESSION_NAME}` - Agent's tmux session name
/// - `${SESSION_DIR}` - Agent's working directory path
/// - `${WINDOW_INDEX}` - Agent's tmux window index
/// - `${WINDOW_NAME}` - Agent's tmux window name
/// - `${PANE_INDEX}` - Agent's tmux pane index
/// - `${PANE_TARGET}` - Agent's tmux target (session:window.pane)
/// - `${ENV:VAR}` - Environment variable value
fn expand_command_variables(template: &str, agent: &crate::agents::MonitoredAgent) -> String {
    use regex::Regex;

    let mut result = template.to_string();

    // Replace ${SESSION_NAME}
    result = result.replace("${SESSION_NAME}", &agent.session);

    // Replace ${SESSION_DIR}
    result = result.replace("${SESSION_DIR}", &agent.path);

    // Replace ${WINDOW_INDEX}
    result = result.replace("${WINDOW_INDEX}", &agent.window.to_string());

    // Replace ${WINDOW_NAME}
    result = result.replace("${WINDOW_NAME}", &agent.window_name);

    // Replace ${PANE_INDEX}
    result = result.replace("${PANE_INDEX}", &agent.pane.to_string());

    // Replace ${PANE_TARGET}
    result = result.replace("${PANE_TARGET}", &agent.target);

    // Replace ${ENV:VAR} using regex
    if let Ok(re) = Regex::new(r"\$\{ENV:([^}]+)\}") {
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                let var_name = &caps[1];
                std::env::var(var_name).unwrap_or_default()
            })
            .to_string();
    }

    result
}

/// Helper to get stdio for logging (debug mode) or null
fn get_log_stdio(debug_mode: bool) -> std::process::Stdio {
    if debug_mode {
        // Try to open .tmuxx.log in current directory
        if let Ok(file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(".tmuxx.log")
        {
            return std::process::Stdio::from(file);
        }
    }
    std::process::Stdio::null()
}
