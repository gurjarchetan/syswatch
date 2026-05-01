use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::Paragraph,
};
use crate::app::AppState;
use crate::input::ActiveTab;
use crate::ui::theme;

const TABS: &[(&str, ActiveTab)] = &[
    ("F1 Overview", ActiveTab::Overview),
    ("F2 Processes", ActiveTab::Processes),
    ("F3 Network", ActiveTab::Network),
    ("F4 Disk", ActiveTab::Disk),
];

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let spans: Vec<Span> = TABS.iter().flat_map(|(label, tab)| {
        let style = if app.active_tab == *tab {
            theme::tab_active_style()
        } else {
            theme::tab_inactive_style()
        };
        [Span::styled(format!(" {} ", label), style), Span::raw(" ")]
    }).collect();

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}
