use ratatui::style::{Color, Modifier, Style};

/// Colour-code a percentage value (0–100).
pub fn pct_color(pct: f64) -> Color {
    if pct >= 90.0      { Color::Red    }
    else if pct >= 70.0 { Color::Yellow }
    else                { Color::Green  }
}

pub fn pct_color_f32(pct: f32) -> Color {
    pct_color(pct as f64)
}

pub fn header_style() -> Style {
    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
}

pub fn title_style() -> Style {
    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
}

pub fn dim_style() -> Style {
    Style::default().fg(Color::DarkGray)
}

pub fn highlight_style() -> Style {
    Style::default()
        .fg(Color::Black)
        .bg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
}

pub fn tab_active_style() -> Style {
    Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
}

pub fn tab_inactive_style() -> Style {
    Style::default().fg(Color::DarkGray)
}
