use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
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
        state: &mut MenuTreeState,
        config: &MenuConfig,
        app_config: &crate::app::Config,
        title: &str,
    ) {
        let area = centered_rect(area, 60, 60);

        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(Line::from(vec![
                Span::styled(
                    format!(" {} ", title),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" (Right/Enter:Expand, *:All, Esc:Close) "),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(if !state.filter.is_empty() { 2 } else { 0 }),
                Constraint::Min(0),
            ])
            .split(inner_area);

        if !state.filter.is_empty() {
            let filter_text = Paragraph::new(format!(" Filter: {}", state.filter))
                .style(Style::default().fg(Color::Yellow));
            frame.render_widget(filter_text, chunks[0]);
        }

        let list_area = chunks[1];

        // Flatten the tree
        let items_to_render = flatten_tree(config, state);

        let list_items: Vec<ListItem> = items_to_render
            .iter()
            .map(|flat| {
                let depth_indent = "  ".repeat(flat.level);
                let has_children = !flat.item.items.is_empty();
                let is_expanded = state.expand_all
                    || state.expanded_paths.contains(&flat.path)
                    || !state.filter.is_empty();

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
                    Span::styled(prefix, Style::default().fg(Color::DarkGray)),
                    Span::raw(&flat.item.name),
                ]);
                ListItem::new(content)
            })
            .collect();

        let list = List::new(list_items)
            .highlight_style(
                Style::default()
                    .bg(parse_color(&app_config.current_item_bg_color))
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, list_area, &mut state.list_state);
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

/// Helper to parse color string to Ratatui Color
fn parse_color(color_str: &str) -> Color {
    match color_str.to_lowercase().as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "gray" | "grey" => Color::Gray,
        "darkgray" | "darkgrey" => Color::DarkGray,
        s if s.starts_with('#') => {
            if let Ok(c) = color_from_hex(s) {
                c
            } else {
                Color::Reset
            }
        }
        _ => Color::Reset,
    }
}

fn color_from_hex(hex: &str) -> Result<Color, ()> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| ())?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| ())?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| ())?;
        Ok(Color::Rgb(r, g, b))
    } else {
        Err(())
    }
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
