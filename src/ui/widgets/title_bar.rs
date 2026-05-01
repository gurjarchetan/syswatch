use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    text::{Line, Span},
    widgets::Paragraph,
};
use crate::app::AppState;
use crate::ui::theme;
use chrono::Local;

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let state = app.state.read();
    let time  = Local::now().format("%H:%M:%S").to_string();
    let uptime = {
        let s = state.uptime_secs;
        format!("up {:02}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
    };

    let line = Line::from(vec![
        Span::styled(" ◈ SysWatch ", theme::header_style()),
        Span::styled(format!("| {} | {} ", state.hostname, state.os_name), theme::dim_style()),
        Span::styled(format!("{} | {} ", uptime, time), theme::dim_style()),
    ]);

    f.render_widget(Paragraph::new(line).alignment(Alignment::Left), area);
}
