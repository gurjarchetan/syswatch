use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::app::AppState;
use crate::collector::memory::MemStats;
use crate::ui::theme;

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let state = app.state.read();
    let mem = &state.memory;

    let used_pct = mem.used_pct();
    let swap_pct = mem.swap_pct();

    let ram_bar  = build_bar(used_pct, 20);
    let swap_bar = build_bar(swap_pct, 20);

    let lines = vec![
        Line::from(vec![
            Span::styled("RAM  ", theme::header_style()),
            Span::styled(ram_bar, Style::default().fg(theme::pct_color(used_pct))),
            Span::raw(format!(
                "  {} / {} (Free: {})",
                MemStats::fmt_kb(mem.used_kb),
                MemStats::fmt_kb(mem.total_kb),
                MemStats::fmt_kb(mem.free_kb),
            )),
        ]),
        Line::from(vec![
            Span::styled("Swap ", theme::header_style()),
            Span::styled(swap_bar, Style::default().fg(theme::pct_color(swap_pct))),
            Span::raw(format!(
                "  {} / {}",
                MemStats::fmt_kb(mem.swap_used_kb),
                MemStats::fmt_kb(mem.swap_total_kb),
            )),
        ]),
    ];

    let block = Block::default()
        .title(Span::styled(" Memory ", theme::title_style()))
        .borders(Borders::ALL);

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn build_bar(pct: f64, width: usize) -> String {
    let filled = ((pct / 100.0) * width as f64) as usize;
    format!(
        "[{}{}] {:5.1}%",
        "█".repeat(filled),
        "░".repeat(width - filled),
        pct
    )
}
