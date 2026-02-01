use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Flex, Layout, Margin, Rect},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};

/// Type of command in command palette
#[derive(Debug, Clone, PartialEq)]
pub enum CommandType {
    /// ExecuteCommand action
    ExecuteCommand { command: String },
    /// SendKeys action
    SendKeys { keys: String },
    /// Navigate action
    Navigate(crate::app::NavAction),
    /// Shell command (with ! prefix)
    ShellCommand { command: String },
    /// KeyAction (approve, reject, etc.)
    KeyAction { name: String },
}

/// Item in command palette
#[derive(Debug, Clone)]
pub struct CommandPaletteItem {
    /// Display name
    pub name: String,
    /// Type of command
    pub item_type: CommandType,
}

pub struct CommandPaletteWidget;

impl CommandPaletteWidget {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        state: &crate::app::CommandPaletteState,
        styles: &crate::ui::Styles,
    ) {
        // Check minimum terminal size
        if area.width < 60 || area.height < 15 {
            return;
        }

        // Create centered popup (70% width, 40% height)
        let popup_area = Self::centered_popup(area, 70, 40);

        frame.render_widget(Clear, popup_area);

        // Block with title "Command Palette"
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Line::from(vec![
                Span::styled(" Command Palette ", styles.header),
                Span::raw(" "),
                if state.filter.starts_with('!') {
                    Span::styled("[Shell Mode]", styles.highlight)
                } else {
                    Span::raw("")
                },
            ]))
            .title_style(styles.header);
        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        // Simple fixed layout: filter (1) | separator (1) | list (fill) | spacer (1) | hints (2)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Filter input (only 1 line now)
                Constraint::Length(1), // Separator line
                Constraint::Min(5),    // Command list (fills remaining space)
                Constraint::Length(1), // Spacer line (requested empty line)
                Constraint::Length(2), // Hints
            ])
            .split(inner);

        // Add horizontal padding for each widget
        let padded_chunks = [
            chunks[0].inner(Margin {
                horizontal: 1,
                vertical: 0,
            }),
            chunks[1].inner(Margin {
                horizontal: 1,
                vertical: 0,
            }),
            chunks[2].inner(Margin {
                horizontal: 1,
                vertical: 0,
            }),
            chunks[3].inner(Margin {
                horizontal: 1,
                vertical: 0,
            }),
            chunks[4].inner(Margin {
                horizontal: 1,
                vertical: 0,
            }),
        ];

        // Render filter input
        let filter_text = Paragraph::new(Line::from(vec![
            Span::styled(">", styles.dimmed),
            Span::raw(" "),
            if state.filter.is_empty() {
                Span::styled(
                    "Type to search commands, or use ! for shell commands...",
                    styles.dimmed,
                )
            } else {
                Span::styled(&state.filter, styles.normal)
            },
        ]));
        frame.render_widget(filter_text, padded_chunks[0]);

        // Render separator line
        let separator = Paragraph::new(Span::raw("─".repeat(padded_chunks[1].width as usize)))
            .style(styles.dimmed)
            .alignment(Alignment::Left);
        frame.render_widget(separator, padded_chunks[1]);

        // Render filtered commands with fuzzy matching
        let filtered_items = Self::filter_commands(&state.items, &state.filter);
        let list_items: Vec<ListItem> = filtered_items
            .iter()
            .map(|(_i, item)| {
                let icon = match &item.item_type {
                    CommandType::ExecuteCommand { .. } => "⚡",
                    CommandType::SendKeys { .. } => "⌨",
                    CommandType::Navigate(_) => "📍",
                    CommandType::ShellCommand { .. } => "🔧",
                    CommandType::KeyAction { .. } => "▸",
                };

                ListItem::new(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(format!("{} ", icon), styles.normal),
                    Span::styled(&item.name, styles.normal),
                ]))
            })
            .collect();

        // Create a mutable list state for rendering
        let mut list_state = state.list_state.clone();

        let list = List::new(list_items).highlight_style(styles.selected);
        frame.render_stateful_widget(list, padded_chunks[2], &mut list_state);

        // Render scrollbar if items don't fit
        if filtered_items.len() > padded_chunks[2].height as usize {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"))
                .track_symbol(Some("│"));

            let mut scrollbar_state = ScrollbarState::new(filtered_items.len());
            if let Some(selected) = list_state.selected() {
                scrollbar_state = scrollbar_state.position(selected);
            }

            frame.render_stateful_widget(
                scrollbar,
                padded_chunks[2].inner(Margin {
                    vertical: 0,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );
        }

        // Render hints (now in padded_chunks[4])
        let hints = vec![
            Line::from(vec![
                Span::styled("[Enter]", styles.footer_key),
                Span::raw(" Execute  "),
                Span::styled("[Esc]", styles.footer_key),
                Span::raw(" Cancel  "),
                Span::styled("[↑/↓]", styles.footer_key),
                Span::raw(" Navigate"),
                Span::raw("  "),
                Span::styled("[Home/End]", styles.footer_key),
                Span::raw(" First/Last"),
            ]),
            Line::from(vec![
                Span::styled("[!cmd]", styles.footer_key),
                Span::raw(" Shell in selected dir  "),
                Span::styled("[Ctrl+U]", styles.footer_key),
                Span::raw(" Clear"),
            ]),
        ];
        frame.render_widget(Paragraph::new(hints), padded_chunks[4]);
    }

    /// Filter commands based on input
    pub fn filter_commands<'a>(
        items: &'a [CommandPaletteItem],
        filter: &str,
    ) -> Vec<(usize, &'a CommandPaletteItem)> {
        let matcher = SkimMatcherV2::default();

        if filter.is_empty() {
            return items.iter().enumerate().collect();
        }

        // Check if it's a shell command (! prefix)
        if let Some(stripped) = filter.strip_prefix('!') {
            // Filter shell commands by fuzzy match after the !
            let shell_filter = stripped.trim();
            if shell_filter.is_empty() {
                // Just show all shell commands
                return items
                    .iter()
                    .enumerate()
                    .filter(|(_, item)| matches!(item.item_type, CommandType::ShellCommand { .. }))
                    .collect();
            }

            // Fuzzy match shell commands
            return items
                .iter()
                .enumerate()
                .filter(|(_, item)| {
                    if let CommandType::ShellCommand { command } = &item.item_type {
                        matcher.fuzzy_match(command, shell_filter).is_some()
                    } else {
                        false
                    }
                })
                .collect();
        }

        // Fuzzy match against names for internal commands
        items
            .iter()
            .enumerate()
            .filter(|(_, item)| matcher.fuzzy_match(&item.name, filter).is_some())
            .collect()
    }

    /// Create a centered popup rect with given width percentage and height percentage
    fn centered_popup(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        // Use Flex::Center for proper centering (Ratatui 0.29)
        let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }
}
