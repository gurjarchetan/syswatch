use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::app::AppState;
use crate::collector::disk::DiskStats;
use crate::ui::theme;

/// Returns a compact bar like "[███████░░░] 68.3%"
fn use_bar(pct: f64, bar_w: usize) -> (String, String, Color) {
    let filled = ((pct / 100.0) * bar_w as f64).round() as usize;
    let bar_part = format!("{}{}", "█".repeat(filled), "░".repeat(bar_w - filled));
    let pct_part = format!("{:5.1}%", pct);
    let color = if pct >= 90.0 { Color::Red }
                else if pct >= 70.0 { Color::Yellow }
                else { Color::Green };
    (bar_part, pct_part, color)
}

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let state  = app.state.read();
    let disks  = &state.disks;
    let width  = (area.width as usize).saturating_sub(2).max(40);

    // Column widths — df -h style
    // Filesystem | Size | Used | Avail | Use% bar | Mounted on | R/W IOPS
    let col_fs    = 12usize;
    let col_size  = 7usize;
    let col_used  = 7usize;
    let col_avail = 7usize;
    let col_bar   = 14usize;  // bar portion
    let col_pct   = 6usize;
    let col_iops  = 16usize;  // "R:9999 W:9999"
    let col_mount = width.saturating_sub(col_fs + col_size + col_used + col_avail + col_bar + col_pct + col_iops + 8);
    let col_mount = col_mount.max(8).min(24);

    let mut lines: Vec<Line> = Vec::new();

    // ── header ─────────────────────────────────────────────────────────────
    lines.push(Line::from(vec![
        Span::styled(format!("{:<col_fs$} ", "Filesystem"),   theme::header_style()),
        Span::styled(format!("{:>col_size$} ", "Size"),        theme::header_style()),
        Span::styled(format!("{:>col_used$} ", "Used"),        theme::header_style()),
        Span::styled(format!("{:>col_avail$} ", "Avail"),      theme::header_style()),
        Span::styled(format!("{:<col_bar$}{:<col_pct$} ", "Use%", ""), theme::header_style()),
        Span::styled(format!("{:<col_mount$} ", "Mounted on"), theme::header_style()),
        Span::styled(format!("{:<col_iops$}", "R-IOPS  W-IOPS"), theme::header_style()),
    ]));

    // ── separator ──────────────────────────────────────────────────────────
    lines.push(Line::from(Span::styled(
        "─".repeat(width.min(area.width as usize)),
        Style::default().fg(Color::DarkGray),
    )));

    // ── one row per mounted disk ────────────────────────────────────────────
    for disk in disks {
        let pct  = disk.used_pct();
        let (bar_part, pct_part, color) = use_bar(pct, col_bar);

        // shorten device name: /dev/mapper/ubuntu-root → ubuntu-root
        let dev_short: String = disk.name.rsplit('/').next()
            .unwrap_or(&disk.name)
            .chars().take(col_fs)
            .collect();

        let total_str = DiskStats::fmt_size(disk.total_kb);
        let used_str  = DiskStats::fmt_size(disk.used_kb);
        let avail_str = DiskStats::fmt_size(disk.avail_kb);
        let mount_str: String = disk.mount.chars().take(col_mount).collect();
        let iops_str  = format!("{:<7} {:<7}",
            format!("R:{}", disk.read_iops),
            format!("W:{}", disk.write_iops),
        );

        lines.push(Line::from(vec![
            Span::styled(format!("{:<col_fs$} ", dev_short),         theme::dim_style()),
            Span::raw(format!("{:>col_size$} ", total_str)),
            Span::styled(format!("{:>col_used$} ", used_str),        Style::default().fg(color)),
            Span::raw(format!("{:>col_avail$} ", avail_str)),
            Span::styled(bar_part,                                   Style::default().fg(color)),
            Span::styled(format!(" {:<col_pct$} ", pct_part),       Style::default().fg(color)),
            Span::styled(format!("{:<col_mount$} ", mount_str),      theme::dim_style()),
            Span::styled(iops_str,                                   Style::default().fg(Color::Cyan)),
        ]));

        // Sub-row: FS type + throughput
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:<6}  r:{:<10} w:{:<10}",
                    disk.fs_type,
                    DiskStats::fmt_speed(disk.read_bps),
                    DiskStats::fmt_speed(disk.write_bps),
                ),
                theme::dim_style(),
            ),
        ]));
    }

    let block = Block::default()
        .title(Span::styled(" Disk  [df -h] ", theme::title_style()))
        .borders(Borders::ALL);

    f.render_widget(Paragraph::new(lines).block(block), area);
}

