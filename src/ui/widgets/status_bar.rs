use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use crate::app::AppState;
use crate::ui::theme;

fn key(k: &'static str) -> Span<'static> {
    Span::styled(
        format!(" {} ", k),
        Style::default().fg(theme::C_ACCENT).bg(theme::C_BG_SEL).add_modifier(Modifier::BOLD),
    )
}
fn hint(h: &'static str) -> Span<'static> {
    Span::styled(format!(" {}  ", h), theme::dim_style())
}

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let line = if app.filter_mode {
        Line::from(vec![
            Span::styled(
                format!("  🔍 Filter ▸ {}▋  ", app.filter_text),
                Style::default().fg(theme::C_YELLOW).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Esc = clear", theme::dim_style()),
        ])
    } else {
        Line::from(vec![
            key("/"), hint("Filter"),
            key("f"), hint("Sort"),
            key("k"), hint("Kill"),
            key("↑↓"), hint("Scroll"),
            key("Tab"), hint("Switch"),
            key("q"), hint("Quit"),
        ])
    };

    f.render_widget(Paragraph::new(line).alignment(Alignment::Left), area);
}
