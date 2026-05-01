use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::app::AppState;
use crate::collector::disk::DiskStats;
use crate::ui::{theme, widgets::gauge};

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let state = app.state.read();
    let disks = &state.disks;

    let mut lines = vec![
        Line::from(vec![Span::styled(
            format!("{:<20} {:<8} {:>22}  {:<20}",
                "Mount", "FS", "Usage", "I/O"),
            theme::header_style(),
        )]),
    ];

    for disk in disks {
        let pct = disk.used_pct();
        let (bar, color) = gauge::ascii_bar(pct, 16);
        lines.push(Line::from(vec![
            Span::raw(format!("{:<20} ", crate::ui::truncate(&disk.mount, 18))),
            Span::raw(format!("{:<8} ", crate::ui::truncate(&disk.fs_type, 6))),
            Span::styled(bar, Style::default().fg(color)),
            Span::raw(format!(
                "  R:{:<10} W:{:<10}",
                DiskStats::fmt_speed(disk.read_bps),
                DiskStats::fmt_speed(disk.write_bps),
            )),
        ]));

        // Total / Used / Free
        let free_kb = disk.total_kb.saturating_sub(disk.used_kb);
        use crate::collector::memory::MemStats;
        lines.push(Line::from(vec![
            Span::styled(format!(
                "   Total:{} Used:{} Free:{}",
                MemStats::fmt_kb(disk.total_kb),
                MemStats::fmt_kb(disk.used_kb),
                MemStats::fmt_kb(free_kb),
            ), theme::dim_style()),
        ]));
    }

    let block = Block::default()
        .title(Span::styled(" Disk Health ", theme::title_style()))
        .borders(Borders::ALL);

    f.render_widget(Paragraph::new(lines).block(block), area);
}
