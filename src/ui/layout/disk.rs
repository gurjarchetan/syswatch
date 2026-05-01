use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::app::AppState;
use crate::collector::disk::DiskStats;
use crate::ui::theme;

fn use_color(pct: f64) -> Color {
    if pct >= 90.0 { Color::Red }
    else if pct >= 75.0 { Color::Yellow }
    else { Color::Green }
}

fn use_bar(pct: f64, width: usize) -> String {
    let filled = ((pct / 100.0) * width as f64).round().clamp(0.0, width as f64) as usize;
    format!("[{}{}]", "█".repeat(filled), "░".repeat(width - filled))
}

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let state = app.state.read();
    let disks = &state.disks;

    // df -hT style columns
    let w_fs   = 20usize;
    let w_type = 9usize;
    let w_size = 6usize;
    let w_used = 6usize;
    let w_avail= 6usize;
    let w_bar  = 12usize;
    let w_pct  = 4usize;

    let mut lines: Vec<Line> = Vec::new();

    // ── header ─────────────────────────────────────────────────────────────
    let hdr = Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
    lines.push(Line::from(vec![
        Span::styled(format!("{:<w_fs$}  ", "Filesystem"),  hdr),
        Span::styled(format!("{:<w_type$} ", "Type"),        hdr),
        Span::styled(format!("{:>w_size$} ", "Size"),        hdr),
        Span::styled(format!("{:>w_used$} ", "Used"),        hdr),
        Span::styled(format!("{:>w_avail$} ", "Avail"),      hdr),
        Span::styled(format!("{:<w_bar$}  ", ""),            Style::default()),
        Span::styled(format!("{:>w_pct$}   ", "Use%"),       hdr),
        Span::styled("Mounted on",                           hdr),
    ]));

    // ── one row per filesystem ─────────────────────────────────────────────
    for disk in disks {
        let pct   = disk.used_pct();
        let color = use_color(pct);
        let bar   = use_bar(pct, w_bar);

        let fs_name = crate::ui::truncate(&disk.name, w_fs);
        let fs_type = crate::ui::truncate(&disk.fs_type, w_type - 1);
        let size_s  = DiskStats::fmt_size(disk.total_kb);
        let used_s  = DiskStats::fmt_size(disk.used_kb);
        let avail_s = DiskStats::fmt_size(disk.avail_kb);

        lines.push(Line::from(vec![
            Span::styled(format!("{:<w_fs$}  ", fs_name),  theme::dim_style()),
            Span::raw(   format!("{:<w_type$} ", fs_type)),
            Span::raw(   format!("{:>w_size$} ", size_s)),
            Span::styled(format!("{:>w_used$} ", used_s),  Style::default().fg(color)),
            Span::raw(   format!("{:>w_avail$} ", avail_s)),
            Span::styled(format!("{} ", bar),              Style::default().fg(color)),
            Span::styled(format!("{:>w_pct$}%  ", pct as u64), Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::styled(disk.mount.clone(),               theme::dim_style()),
        ]));

        // I/O sub-row only when there is activity
        if disk.read_bps > 0 || disk.write_bps > 0 || disk.read_iops > 0 || disk.write_iops > 0 {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  r:{:<10} w:{:<10}  riops:{:<5} wiops:{}",
                        DiskStats::fmt_speed(disk.read_bps),
                        DiskStats::fmt_speed(disk.write_bps),
                        disk.read_iops,
                        disk.write_iops,
                    ),
                    Style::default().fg(Color::Cyan),
                ),
            ]));
        }
    }

    let block = Block::default()
        .title(Span::styled(" Disk ", theme::title_style()))
        .borders(Borders::ALL);

    let scroll_offset = app.scroll_offset as u16;
    f.render_widget(
        Paragraph::new(lines).block(block).scroll((scroll_offset, 0)),
        area,
    );
}
