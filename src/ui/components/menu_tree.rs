use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::menu_config::{MenuConfig, MenuItem};

use std::collections::HashSet;

#[derive(Debug)]
pub struct MenuTreeState {
    pub list_state: ListState,
    pub filter: String,
    /// Set of unique paths (names joined) that are currently expanded
    pub expanded_paths: HashSet<Vec<String>>,
    /// If true, all nodes are treated as expanded
    pub expand_all: bool,
}

impl Default for MenuTreeState {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuTreeState {
    pub fn new() -> Self {
        Self {
            list_state: ListState::default().with_selected(Some(0)),
            filter: String::new(),
            expanded_paths: HashSet::new(),
            expand_all: false,
        }
    }

    pub fn key_down(&mut self, items_len: usize) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= items_len.saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn key_up(&mut self, items_len: usize) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    items_len.saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn toggle_expansion(&mut self, path: Vec<String>) {
        if self.expanded_paths.contains(&path) {
            self.expanded_paths.remove(&path);
        } else {
            self.expanded_paths.insert(path);
        }
    }
}

pub struct MenuTreeWidget;

#[derive(Clone)]
pub struct FlatMenuItem<'a> {
    pub level: usize,
    pub item: &'a MenuItem,
    pub path: Vec<String>,
}

impl MenuTreeWidget {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        menu_state: &mut MenuTreeState,
        config: &MenuConfig,
        styles: &crate::ui::Styles,
        title: &str,
    ) {
        let area = centered_rect(area, 60, 60);

        frame.render_widget(Clear, area);

        let mut block = Block::default()
            .title(Line::from(vec![
                Span::styled(format!(" {} ", title), styles.header),
                Span::raw(" (Right:Expand, *:All, Esc:Close) "),
            ]))
            .borders(Borders::ALL)
            .border_style(styles.header);

        if title.contains("Prompts") {
            block = block.title_bottom(
                Line::from(vec![
                    Span::styled("[Enter]", styles.footer_key),
                    Span::raw(" Send  "),
                    Span::styled("[Alt+Enter]", styles.footer_key),
                    Span::raw(" Edit & Send"),
                ])
                .alignment(ratatui::layout::Alignment::Center),
            );
        } else {
            block = block.title_bottom(
                Line::from(vec![
                    Span::styled("[Enter]", styles.footer_key),
                    Span::raw(" Execute/Expand"),
                ])
                .alignment(ratatui::layout::Alignment::Center),
            );
        }

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        // Flatten the tree
        let items_to_render = flatten_tree(config, menu_state);

        // Get selected item content for preview
        let preview_text = if let Some(index) = menu_state.list_state.selected() {
            if let Some(flat) = items_to_render.get(index) {
                if let Some(cmd) = &flat.item.execute_command {
                    Some(format!("Command: {}", cmd.command))
                } else {
                    flat.item.text.as_ref().map(|t| format!("Prompt: {}", t))
                }
            } else {
                None
            }
        } else {
            None
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(if !menu_state.filter.is_empty() { 2 } else { 0 }),
                Constraint::Min(0),
                Constraint::Length(if preview_text.is_some() { 3 } else { 0 }),
            ])
            .split(inner_area);

        if !menu_state.filter.is_empty() {
            let filter_text =
                Paragraph::new(format!(" Filter: {}", menu_state.filter)).style(styles.highlight);
            frame.render_widget(filter_text, chunks[0]);
        }

        let list_area = chunks[1];

        if let Some(text) = preview_text {
            let preview_block = Block::default()
                .borders(Borders::TOP)
                .border_style(styles.dimmed);
            let preview_paragraph = Paragraph::new(text)
                .block(preview_block)
                .style(styles.dimmed);
            frame.render_widget(preview_paragraph, chunks[2]);
        }

        let list_items: Vec<ListItem> = items_to_render
            .iter()
            .map(|flat| {
                let depth_indent = "  ".repeat(flat.level);
                let has_children = !flat.item.items.is_empty();
                let is_expanded = menu_state.expand_all
                    || menu_state.expanded_paths.contains(&flat.path)
                    || !menu_state.filter.is_empty();

                let prefix = if has_children {
                    if is_expanded {
                        "▼ "
                    } else {
                        "▶ "
                    }
                } else {
                    "  "
                };

                let content = Line::from(vec![
                    Span::raw(depth_indent),
                    Span::styled(prefix, styles.dimmed),
                    Span::raw(&flat.item.name),
                ]);
                ListItem::new(content)
            })
            .collect();

        let list = List::new(list_items)
            .highlight_style(styles.selected)
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, list_area, &mut menu_state.list_state);
    }
}

fn flatten_tree<'a>(config: &'a MenuConfig, state: &MenuTreeState) -> Vec<FlatMenuItem<'a>> {
    let mut flat = Vec::new();
    let matcher = SkimMatcherV2::default();

    fn recurse<'a>(
        items: &'a [MenuItem],
        state: &MenuTreeState,
        level: usize,
        current_path: &[String],
        flat: &mut Vec<FlatMenuItem<'a>>,
        matcher: &SkimMatcherV2,
    ) {
        for item in items {
            let mut path = current_path.to_owned();
            path.push(item.name.clone());

            let is_match =
                state.filter.is_empty() || matcher.fuzzy_match(&item.name, &state.filter).is_some();

            // Should we show this item?
            // If filtering: show if it matches OR any descendant matches.
            // If not filtering: show if level 0 OR parent expanded.

            let should_show = if !state.filter.is_empty() {
                is_match || has_matching_descendant(item, &state.filter, matcher)
            } else {
                true // Level 0 items are always passed to recurse, deeper ones checked below
            };

            if should_show {
                flat.push(FlatMenuItem {
                    level,
                    item,
                    path: path.clone(),
                });

                let is_expanded = state.expand_all
                    || state.expanded_paths.contains(&path)
                    || !state.filter.is_empty();
                if is_expanded && !item.items.is_empty() {
                    recurse(&item.items, state, level + 1, &path, flat, matcher);
                }
            }
        }
    }

    recurse(&config.items, state, 0, &Vec::new(), &mut flat, &matcher);
    flat
}

fn has_matching_descendant(item: &MenuItem, filter: &str, matcher: &SkimMatcherV2) -> bool {
    for child in &item.items {
        if matcher.fuzzy_match(&child.name, filter).is_some()
            || has_matching_descendant(child, filter, matcher)
        {
            return true;
        }
    }
    false
}

fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn get_current_items_count(config: &MenuConfig, state: &MenuTreeState) -> usize {
    flatten_tree(config, state).len()
}

pub fn find_flat_menu_item_by_index<'a>(
    config: &'a MenuConfig,
    state: &MenuTreeState,
    index: usize,
) -> Option<FlatMenuItem<'a>> {
    flatten_tree(config, state).get(index).cloned()
}
