use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use crate::app::AppState;
use crate::input::ActiveTab;
use crate::ui::theme;

const TABS: &[(&str, &str, ActiveTab)] = &[
    ("F1", "Overview",  ActiveTab::Overview),
    ("F2", "Processes", ActiveTab::Processes),
    ("F3", "Network",   ActiveTab::Network),
    ("F4", "Disk",      ActiveTab::Disk),
];

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let mut spans: Vec<Span> = vec![Span::raw(" ")];

    for (key, label, tab) in TABS {
        if app.active_tab == *tab {
            spans.push(Span::styled(
                format!(" {} ", key),
                Style::default().fg(theme::C_ACCENT).bg(theme::C_BG_SEL).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(
                format!(" {} ", label),
                Style::default().fg(theme::C_WHITE).bg(theme::C_BG_SEL).add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(
                format!(" {} ", key),
                Style::default().fg(theme::C_BLUE),
            ));
            spans.push(Span::styled(
                format!(" {} ", label),
                theme::dim_style(),
            ));
        }
        spans.push(Span::raw("  "));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}
