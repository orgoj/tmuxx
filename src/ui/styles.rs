use ratatui::style::{Color, Modifier, Style};

/// Central style definitions for the application
pub struct Styles;

impl Styles {
    // Status colors
    pub fn idle() -> Style {
        Style::default().fg(Color::Green)
    }

    pub fn processing() -> Style {
        Style::default().fg(Color::Yellow)
    }

    pub fn awaiting_approval() -> Style {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    }

    pub fn error() -> Style {
        Style::default().fg(Color::Red)
    }

    pub fn unknown() -> Style {
        Style::default().fg(Color::DarkGray)
    }

    // UI element styles
    pub fn header() -> Style {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }

    pub fn selected() -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }

    pub fn normal() -> Style {
        Style::default().fg(Color::White)
    }

    pub fn dimmed() -> Style {
        Style::default().fg(Color::DarkGray)
    }

    pub fn highlight() -> Style {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    }

    pub fn border() -> Style {
        Style::default().fg(Color::Gray)
    }

    pub fn border_focused() -> Style {
        Style::default().fg(Color::Cyan)
    }

    // Subagent styles
    pub fn subagent_running() -> Style {
        Style::default().fg(Color::Cyan)
    }

    pub fn subagent_completed() -> Style {
        Style::default().fg(Color::Green)
    }

    pub fn subagent_failed() -> Style {
        Style::default().fg(Color::Red)
    }

    // Footer/Help styles
    pub fn footer_key() -> Style {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    }

    pub fn footer_text() -> Style {
        Style::default().fg(Color::White)
    }

    pub fn parse_color(name: &str) -> Color {
        let name = name.trim().to_lowercase();

        // Hex support (#RRGGBB)
        if name.starts_with('#') && name.len() == 7 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&name[1..3], 16),
                u8::from_str_radix(&name[3..5], 16),
                u8::from_str_radix(&name[5..7], 16),
            ) {
                return Color::Rgb(r, g, b);
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
                    return Color::Rgb(r, g, b);
                }
            }
        }

        match name.as_str() {
            "magenta" => Color::Magenta,
            "blue" => Color::Blue,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "cyan" => Color::Cyan,
            "red" => Color::Red,
            "white" => Color::White,
            "black" => Color::Rgb(0, 0, 0),
            "gray" | "grey" => Color::Gray,
            "darkgray" | "darkgrey" => Color::DarkGray,
            "lightmagenta" => Color::LightMagenta,
            "lightblue" => Color::LightBlue,
            "lightgreen" => Color::LightGreen,
            "lightyellow" => Color::LightYellow,
            "lightcyan" => Color::LightCyan,
            "lightred" => Color::LightRed,
            _ => Color::Gray, // Safer fallback than bright cyan
        }
    }
}
