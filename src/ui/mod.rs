pub mod braille;
pub mod widgets;
pub mod layout;
pub mod theme;

/// Truncate a string to at most `max_chars` characters, appending … if needed.
pub fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let mut r: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        r.push('…');
        r
    }
}

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::app::AppState;
use crate::input::ActiveTab;

pub fn draw(f: &mut Frame, app: &AppState) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // title bar
            Constraint::Length(1),  // tab bar
            Constraint::Min(0),     // main content
            Constraint::Length(1),  // status bar
        ])
        .split(area);

    widgets::title_bar::render(f, chunks[0], app);
    widgets::tab_bar::render(f, chunks[1], app);

    match app.active_tab {
        ActiveTab::Overview  => layout::overview::render(f, chunks[2], app),
        ActiveTab::Processes => layout::processes::render(f, chunks[2], app),
        ActiveTab::Network   => layout::network::render(f, chunks[2], app),
        ActiveTab::Disk      => layout::disk::render(f, chunks[2], app),
    }

    widgets::status_bar::render(f, chunks[3], app);
}

