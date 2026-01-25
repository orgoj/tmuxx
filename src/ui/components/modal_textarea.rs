use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
    Frame,
};
use tui_textarea::{Input, Key, TextArea};

/// State for modal textarea dialog
#[derive(Debug, Clone)]
pub struct ModalTextareaState {
    pub textarea: TextArea<'static>,
    pub title: String,
    pub is_single_line: bool,
    pub readonly: bool,
}

impl ModalTextareaState {
    pub fn new(
        title: String,
        prompt: String,
        initial: String,
        single_line: bool,
        readonly: bool,
    ) -> Self {
        let mut textarea = if initial.is_empty() {
            TextArea::default()
        } else {
            TextArea::new(initial.lines().map(|s| s.to_string()).collect::<Vec<_>>())
        };

        // Background color - light gray for both readonly and editable (200 on 0-255 scale)
        let bg_color = Color::Rgb(200, 200, 200);

        // Configure styling with background
        textarea.set_style(Style::default().fg(Color::White).bg(bg_color));
        textarea.set_cursor_style(Style::default().bg(Color::Black).fg(Color::White));
        textarea.set_cursor_line_style(Style::default()); // Disable underline on cursor line
        textarea.set_placeholder_style(Style::default().fg(Color::DarkGray).bg(bg_color));
        textarea.set_placeholder_text(&prompt);

        Self {
            textarea,
            title,
            is_single_line: single_line,
            readonly,
        }
    }

    /// Handle input key event
    /// Returns true if dialog should close (Esc pressed or Enter in single-line mode)
    pub fn handle_input(&mut self, input: Input) -> bool {
        match input {
            Input { key: Key::Esc, .. } => true, // Always close on Esc

            // In readonly mode, only allow scrolling
            Input { key: Key::Up, .. } if self.readonly => {
                self.textarea.move_cursor(tui_textarea::CursorMove::Up);
                false
            }
            Input { key: Key::Down, .. } if self.readonly => {
                self.textarea.move_cursor(tui_textarea::CursorMove::Down);
                false
            }
            Input {
                key: Key::PageUp, ..
            } if self.readonly => {
                self.textarea.scroll(tui_textarea::Scrolling::PageUp);
                false
            }
            Input {
                key: Key::PageDown, ..
            } if self.readonly => {
                self.textarea.scroll(tui_textarea::Scrolling::PageDown);
                false
            }

            // Normal editable mode
            Input {
                key: Key::Enter, ..
            } if self.is_single_line => true,
            /* Input {
                key: Key::Enter,
                ctrl: true,
                ..
            } => true, */
            Input {
                key: Key::Enter,
                alt: true,
                ..
            } => true,
            input if !self.readonly => {
                self.textarea.input(input);
                false
            }

            // Ignore all other keys in readonly mode
            _ => false,
        }
    }

    pub fn get_text(&self) -> String {
        self.textarea.lines().join("\n")
    }
}

/// Widget for rendering modal textarea
pub struct ModalTextareaWidget;

impl ModalTextareaWidget {
    pub fn render(frame: &mut Frame, area: Rect, state: &ModalTextareaState) {
        // Check minimum terminal size
        if area.width < 40 || area.height < 10 {
            return;
        }

        // Create centered popup (80% width, 60% height)
        let popup_area = Self::centered_popup(area, 80, 60);

        // Clear background
        frame.render_widget(Clear, popup_area);

        // Block with NO background color (transparent)
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(state.title.as_str())
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        // Create layout inside block
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(inner);

        // Render textarea (placeholder is built-in)
        frame.render_widget(&state.textarea, chunks[0]);

        // Render scrollbar
        let lines_count = state.textarea.lines().len();
        let (row, _) = state.textarea.cursor();
        let scrollbar_state = ScrollbarState::new(lines_count).position(row);
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        frame.render_stateful_widget(
            scrollbar,
            chunks[0].inner(ratatui::layout::Margin {
                vertical: 0,
                horizontal: 0,
            }), // Render over the textarea
            &mut scrollbar_state.clone(), // Clone because we don't have mutable state here easily, but state is small
        );

        // Render hints on ONE LINE
        let hints = if state.readonly {
            Line::from(vec![
                Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
                Span::raw(" Close  "),
                Span::styled("[↑/↓]", Style::default().fg(Color::Yellow)),
                Span::raw(" Scroll"),
            ])
        } else {
            Line::from(vec![
                Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
                Span::raw(if state.is_single_line {
                    " Submit  "
                } else {
                    " Newline  "
                }),
                Span::raw(if !state.is_single_line { "  " } else { "" }),
                Span::styled(
                    if !state.is_single_line {
                        "[Alt+Enter]"
                    } else {
                        ""
                    },
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(if !state.is_single_line {
                    " Submit  "
                } else {
                    ""
                }),
                Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
                Span::raw(" Cancel  "),
                Span::styled("[Ctrl+U/R]", Style::default().fg(Color::Yellow)),
                Span::raw(" Undo/Redo"),
            ])
        };

        let hints_paragraph = Paragraph::new(hints);
        frame.render_widget(hints_paragraph, chunks[1]);
    }

    fn centered_popup(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(area);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}
