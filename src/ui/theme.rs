use ratatui::style::{Color, Modifier, Style};

// ── Palette ──────────────────────────────────────────────────────────────────
pub const C_ACCENT:   Color = Color::Rgb(82,  196, 255);  // bright sky-blue
pub const C_GREEN:    Color = Color::Rgb(80,  250, 123);  // neon green
pub const C_YELLOW:   Color = Color::Rgb(255, 215,  0);   // gold
pub const C_ORANGE:   Color = Color::Rgb(255, 140,  0);   // orange
pub const C_RED:      Color = Color::Rgb(255,  85,  85);  // vivid red
pub const C_MAGENTA:  Color = Color::Rgb(255, 121, 198);  // hot pink
pub const C_BLUE:     Color = Color::Rgb(98,  114, 255);  // periwinkle
pub const C_TEAL:     Color = Color::Rgb(0,   210, 180);  // teal
pub const C_DIM:      Color = Color::Rgb(90,  100, 120);  // muted blue-grey
pub const C_BORDER:   Color = Color::Rgb(55,   65,  85);  // panel border
pub const C_BG_SEL:   Color = Color::Rgb(30,   50,  80);  // selection bg
pub const C_WHITE:    Color = Color::Rgb(220, 225, 235);  // near-white text

/// Gradient: green → yellow → orange → red
pub fn pct_color(pct: f64) -> Color {
    if pct >= 90.0      { C_RED    }
    else if pct >= 75.0 { C_ORANGE }
    else if pct >= 50.0 { C_YELLOW }
    else if pct >= 25.0 { C_GREEN  }
    else                { C_TEAL   }
}

pub fn pct_color_f32(pct: f32) -> Color {
    pct_color(pct as f64)
}

pub fn header_style() -> Style {
    Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD)
}

pub fn title_style() -> Style {
    Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD)
}

pub fn dim_style() -> Style {
    Style::default().fg(C_DIM)
}

pub fn value_style() -> Style {
    Style::default().fg(C_WHITE)
}

pub fn highlight_style() -> Style {
    Style::default()
        .fg(C_ACCENT)
        .bg(C_BG_SEL)
        .add_modifier(Modifier::BOLD)
}

pub fn tab_active_style() -> Style {
    Style::default()
        .fg(Color::Rgb(10, 10, 20))
        .bg(C_ACCENT)
        .add_modifier(Modifier::BOLD)
}

pub fn tab_inactive_style() -> Style {
    Style::default().fg(C_DIM)
}

pub fn border_style() -> Style {
    Style::default().fg(C_BORDER)
}

pub fn border_style_active() -> Style {
    Style::default().fg(C_ACCENT)
}
