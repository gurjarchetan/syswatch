use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    text::{Line, Span},
    widgets::Paragraph,
};
use crate::app::AppState;
use crate::ui::theme;

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let filter_hint = if app.filter_mode {
        format!(" Filter: {}█ ", app.filter_text)
    } else {
        " [/] Filter  [f] Sort  [k] Kill  [q] Quit  [Tab] Switch tab  [↑↓] Scroll".to_string()
    };

    let line = Line::from(vec![
        Span::styled(filter_hint, theme::dim_style()),
    ]);

    f.render_widget(Paragraph::new(line).alignment(Alignment::Left), area);
}
