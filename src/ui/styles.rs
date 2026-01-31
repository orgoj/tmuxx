use crate::app::config::ThemeConfig;
use ratatui::style::{Color, Modifier, Style};

/// Central style definitions for the application
#[derive(Debug, Clone)]
pub struct Styles {
    pub idle: Style,
    pub processing: Style,
    pub awaiting_approval: Style,
    pub error: Style,
    pub unknown: Style,
    pub header: Style,
    pub selected: Style,
    pub normal: Style,
    pub dimmed: Style,
    pub highlight: Style,
    pub border: Style,
    pub border_focused: Style,
    pub subagent_running: Style,
    pub subagent_completed: Style,
    pub subagent_failed: Style,
    pub footer_key: Style,
    pub footer_text: Style,
    pub bg: Color,
}

impl Styles {
    pub fn new(theme: &ThemeConfig) -> Self {
        let mut styles = Self {
            idle: Style::default(),
            processing: Style::default(),
            awaiting_approval: Style::default(),
            error: Style::default(),
            unknown: Style::default(),
            header: Style::default(),
            selected: Style::default(),
            normal: Style::default(),
            dimmed: Style::default(),
            highlight: Style::default(),
            border: Style::default(),
            border_focused: Style::default(),
            subagent_running: Style::default(),
            subagent_completed: Style::default(),
            subagent_failed: Style::default(),
            footer_key: Style::default(),
            footer_text: Style::default(),
            bg: Color::Reset,
        };

        if let Some(c) = Self::parse_color(&theme.idle) {
            styles.idle = styles.idle.fg(c);
        }
        if let Some(c) = Self::parse_color(&theme.processing) {
            styles.processing = styles.processing.fg(c);
        }
        if let Some(c) = Self::parse_color(&theme.approval) {
            styles.awaiting_approval = styles.awaiting_approval.fg(c).add_modifier(Modifier::BOLD);
        }
        if let Some(c) = Self::parse_color(&theme.error) {
            styles.error = styles.error.fg(c);
        }
        if let Some(c) = Self::parse_color(&theme.unknown) {
            styles.unknown = styles.unknown.fg(c);
        }

        if let Some(c) = Self::parse_color(&theme.header) {
            styles.header = styles.header.fg(c).add_modifier(Modifier::BOLD);
        }

        if let Some(c) = Self::parse_color(&theme.selected_fg) {
            styles.selected = styles.selected.fg(c);
        }
        if let Some(c) = Self::parse_color(&theme.selected_bg) {
            styles.selected = styles.selected.bg(c);
        }
        styles.selected = styles.selected.add_modifier(Modifier::BOLD);

        if let Some(c) = Self::parse_color(&theme.normal) {
            styles.normal = styles.normal.fg(c);
        }
        if let Some(c) = Self::parse_color(&theme.dimmed) {
            styles.dimmed = styles.dimmed.fg(c);
        }
        if let Some(c) = Self::parse_color(&theme.highlight) {
            styles.highlight = styles.highlight.fg(c).add_modifier(Modifier::BOLD);
        }

        if let Some(c) = Self::parse_color(&theme.border) {
            styles.border = styles.border.fg(c);
        }
        if let Some(c) = Self::parse_color(&theme.border_focused) {
            styles.border_focused = styles.border_focused.fg(c);
        }

        if let Some(c) = Self::parse_color(&theme.subagent_running) {
            styles.subagent_running = styles.subagent_running.fg(c);
        }
        if let Some(c) = Self::parse_color(&theme.subagent_completed) {
            styles.subagent_completed = styles.subagent_completed.fg(c);
        }
        if let Some(c) = Self::parse_color(&theme.subagent_failed) {
            styles.subagent_failed = styles.subagent_failed.fg(c);
        }

        if let Some(c) = Self::parse_color(&theme.footer_key) {
            styles.footer_key = styles.footer_key.fg(c).add_modifier(Modifier::BOLD);
        }
        if let Some(c) = Self::parse_color(&theme.footer_text) {
            styles.footer_text = styles.footer_text.fg(c);
        }
        if let Some(c) = Self::parse_color(&theme.bg) {
            styles.bg = c;
        }

        styles
    }

    pub fn parse_color(name: &str) -> Option<Color> {
        let name = name.trim().to_lowercase();

        if name == "none" || name.is_empty() {
            return None;
        }

        // Hex support (#RRGGBB)
        if name.starts_with('#') && name.len() == 7 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&name[1..3], 16),
                u8::from_str_radix(&name[3..5], 16),
                u8::from_str_radix(&name[5..7], 16),
            ) {
                return Some(Color::Rgb(r, g, b));
            }
        }

        // RGB support (rgb(r,g,b))
        if name.starts_with("rgb(") && name.ends_with(')') {
            let parts: Vec<&str> = name[4..name.len() - 1]
                .split(',')
                .map(|s| s.trim())
                .collect();
            if parts.len() == 3 {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    parts[0].parse::<u8>(),
                    parts[1].parse::<u8>(),
                    parts[2].parse::<u8>(),
                ) {
                    return Some(Color::Rgb(r, g, b));
                }
            }
        }

        let c = match name.as_str() {
            "magenta" => Color::Magenta,
            "blue" => Color::Blue,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "cyan" => Color::Cyan,
            "red" => Color::Red,
            "white" => Color::White,
            "black" => Color::Black,
            "gray" | "grey" => Color::Gray,
            "darkgray" | "darkgrey" | "dark_gray" => Color::DarkGray,
            "lightmagenta" => Color::LightMagenta,
            "lightblue" => Color::LightBlue,
            "lightgreen" => Color::LightGreen,
            "lightyellow" => Color::LightYellow,
            "lightcyan" => Color::LightCyan,
            "lightred" => Color::LightRed,
            _ => return None,
        };
        Some(c)
    }
}
