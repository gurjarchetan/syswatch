use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use crate::app::AppState;
use crate::ui::theme;
use chrono::Local;

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let state  = app.state.read();
    let time   = Local::now().format("%H:%M:%S").to_string();
    let s      = state.uptime_secs;
    let uptime = format!("up {:02}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60);

    let line = Line::from(vec![
        Span::styled(" ◈ ", Style::default().fg(theme::C_ACCENT).add_modifier(Modifier::BOLD)),
        Span::styled("SysWatch", Style::default().fg(theme::C_WHITE).add_modifier(Modifier::BOLD)),
        Span::styled("  ·  ", theme::dim_style()),
        Span::styled(state.hostname.clone(), Style::default().fg(theme::C_TEAL).add_modifier(Modifier::BOLD)),
        Span::styled("  ·  ", theme::dim_style()),
        Span::styled(state.os_name.clone(), Style::default().fg(theme::C_DIM)),
        Span::styled("  ·  ", theme::dim_style()),
        Span::styled(uptime, Style::default().fg(theme::C_GREEN)),
        Span::styled("  ·  ", theme::dim_style()),
        Span::styled(time, Style::default().fg(theme::C_YELLOW).add_modifier(Modifier::BOLD)),
    ]);

    f.render_widget(Paragraph::new(line).alignment(Alignment::Left), area);
}
