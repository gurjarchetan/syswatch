// Re-export common gauge helpers used by layouts
use ratatui::style::Color;
use crate::ui::theme;

/// Build a coloured ASCII bar string.
pub fn ascii_bar(pct: f64, width: usize) -> (String, Color) {
    let filled = ((pct / 100.0) * width as f64) as usize;
    let bar = format!(
        "[{}{}] {:5.1}%",
        "█".repeat(filled),
        "░".repeat(width - filled),
        pct
    );
    (bar, theme::pct_color(pct))
}
