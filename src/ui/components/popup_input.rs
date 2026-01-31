use ratatui::{
    layout::{Constraint, Direction, Flex, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::app::PopupInputState;

/// Popup input dialog widget
pub struct PopupInputWidget;

impl PopupInputWidget {
    /// Render the popup dialog
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        state: &PopupInputState,
        styles: &crate::ui::Styles,
    ) {
        // Check minimum terminal size (80x24)
        if area.width < 80 || area.height < 24 {
            // Terminal too small - don't render popup
            return;
        }

        // Create centered popup using Ratatui's Layout with Flex::Center
        // Width: 70%, Height: 12 lines (1+1+3+1+2 content + 2 borders + 2 margin)
        let popup_area = Self::centered_popup(area, 70, 12);

        // Clear background
        frame.render_widget(Clear, popup_area);

        // Render main block with title (this creates the outer border)
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(state.title.clone())
            .title_style(styles.header)
            .style(styles.header);
        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        // Create layout inside the block (using inner area)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(1), // Prompt line
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Input field with border
                Constraint::Length(1), // Spacer
                Constraint::Min(2),    // Hints
            ])
            .split(inner);

        // Render prompt (chunks[0])
        let prompt = Paragraph::new(state.prompt.as_str()).style(styles.normal);
        frame.render_widget(prompt, chunks[0]);

        // Render input field with border (chunks[2])
        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(styles.idle);
        let input_inner = input_block.inner(chunks[2]);
        frame.render_widget(input_block, chunks[2]);

        // Calculate visible window for horizontal scrolling
        let (visible_text, cursor_x) =
            Self::calculate_scroll(&state.buffer, state.cursor, input_inner.width as usize);

        // Render visible text inside the input border
        let input_text = Paragraph::new(visible_text).style(styles.normal);
        frame.render_widget(input_text, input_inner);

        // Render cursor (green block)
        if cursor_x < input_inner.width {
            let cursor_area = Rect {
                x: input_inner.x + cursor_x,
                y: input_inner.y,
                width: 1,
                height: 1,
            };
            let cursor = Block::default().style(styles.selected);
            frame.render_widget(cursor, cursor_area);
        }

        // Render button hints (chunks[4])
        let hints = vec![
            Line::from(vec![
                Span::styled("[Enter]", styles.footer_key),
                Span::raw(" Submit  "),
                Span::styled("[Esc]", styles.footer_key),
                Span::raw(" Cancel"),
            ]),
            Line::from(vec![
                Span::styled("[Ctrl+U]", styles.footer_key),
                Span::raw(" Clear  "),
                Span::styled("[Ctrl+A]", styles.footer_key),
                Span::raw(" Select All"),
            ]),
        ];
        let hints_paragraph = Paragraph::new(hints).style(styles.normal);
        frame.render_widget(hints_paragraph, chunks[4]);
    }

    /// Calculate visible window for horizontal scrolling
    /// Returns (visible_text, cursor_x_position)
    fn calculate_scroll(buffer: &str, cursor: usize, width: usize) -> (String, u16) {
        if buffer.is_empty() {
            return (String::new(), 0);
        }

        let text_width = buffer.width();

        // If text fits in window, show it all
        if text_width <= width {
            let cursor_x = buffer[..cursor].width();
            return (buffer.to_string(), cursor_x as u16);
        }

        // Text is wider than window - need to scroll
        // Keep cursor in center third of window when possible
        let cursor_visual = buffer[..cursor].width();
        let center = width / 2;

        // Calculate scroll offset
        let scroll_offset = if cursor_visual <= center {
            // Cursor in first half - show from start
            0
        } else if cursor_visual >= text_width.saturating_sub(center) {
            // Cursor near end - show end
            text_width.saturating_sub(width)
        } else {
            // Cursor in middle - center it
            cursor_visual.saturating_sub(center)
        };

        // Extract visible substring by visual width
        let mut visible = String::new();
        let mut current_width = 0;
        let mut chars_skipped = 0;

        for (i, ch) in buffer.chars().enumerate() {
            let ch_width = ch.width().unwrap_or(0);

            if current_width < scroll_offset {
                current_width += ch_width;
                chars_skipped = i + 1;
                continue;
            }

            if current_width >= scroll_offset + width {
                break;
            }

            visible.push(ch);
            current_width += ch_width;
        }

        // Calculate cursor position within visible window
        let cursor_x = if cursor <= chars_skipped {
            0
        } else {
            buffer[chars_skipped..cursor].width()
        };

        (visible, cursor_x.min(width) as u16)
    }

    /// Create a centered popup rect with given width percentage and fixed height
    /// Based on Ratatui official popup example
    fn centered_popup(area: Rect, percent_x: u16, height: u16) -> Rect {
        // Use Flex::Center for proper centering (Ratatui 0.29)
        let vertical = Layout::vertical([Constraint::Length(height)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }
}
