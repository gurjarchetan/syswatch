use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::app::AppState;
use crate::collector::disk::DiskStats;
use crate::ui::theme;

fn gradient_bar(pct: f64, width: usize) -> Vec<Span<'static>> {
    let filled = ((pct / 100.0) * width as f64).round().clamp(0.0, width as f64) as usize;
    let mut spans: Vec<Span<'static>> = Vec::with_capacity(width + 2);
    spans.push(Span::styled("▕", theme::dim_style()));
    for i in 0..width {
        let pos_pct = (i as f64 / width as f64) * 100.0;
        let ch = if i < filled { "█" } else { "░" };
        let c  = if i < filled { theme::pct_color(pos_pct.max(pct * 0.3)) }
                 else { theme::C_BORDER };
        spans.push(Span::styled(ch, Style::default().fg(c)));
    }
    spans.push(Span::styled("▏", theme::dim_style()));
    spans
}

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    // Clone the disk snapshot under the lock, then release it before rendering
    // so the collector thread is not blocked during layout and formatting.
    let disks = app.state.read().disks.clone();

    let w_fs    = 18usize;
    let w_type  =  8usize;
    let w_size  =  6usize;
    let w_used  =  6usize;
    let w_avail =  6usize;
    let w_bar   = 14usize;
    let w_pct   =  4usize;

    let mut lines: Vec<Line> = Vec::new();

    // ── column header ─────────────────────────────────────────────────────
    let hdr = theme::header_style().add_modifier(Modifier::UNDERLINED);
    lines.push(Line::from(vec![
        Span::styled(format!("{:<w_fs$}  ", "Filesystem"), hdr),
        Span::styled(format!("{:<w_type$} ", "Type"),      hdr),
        Span::styled(format!("{:>w_size$} ", "Size"),      hdr),
        Span::styled(format!("{:>w_used$} ", "Used"),      hdr),
        Span::styled(format!("{:>w_avail$} ", "Avail"),    hdr),
        Span::styled(format!("{:<w_bar$}  ", ""),          Style::default()),
        Span::styled(format!("{:>w_pct$}   ", "Use%"),     hdr),
        Span::styled("Mounted on",                         hdr),
    ]));

    if disks.is_empty() {
        lines.push(Line::from(Span::styled("  No disks found", theme::dim_style())));
    }

    // ── one row per filesystem ─────────────────────────────────────────────
    for disk in &disks {
        let pct     = disk.used_pct();
        let color   = theme::pct_color(pct);
        let fs_name = crate::ui::truncate(&disk.name, w_fs);
        let fs_type = crate::ui::truncate(&disk.fs_type, w_type - 1);
        let size_s  = DiskStats::fmt_size(disk.total_kb);
        let used_s  = DiskStats::fmt_size(disk.used_kb);
        let avail_s = DiskStats::fmt_size(disk.avail_kb);

        let mut row: Vec<Span> = vec![
            Span::styled(format!("{:<w_fs$}  ", fs_name), theme::dim_style()),
            Span::styled(format!("{:<w_type$} ", fs_type), Style::default().fg(theme::C_BLUE)),
            Span::styled(format!("{:>w_size$} ", size_s),  theme::dim_style()),
            Span::styled(format!("{:>w_used$} ", used_s),  Style::default().fg(color)),
            Span::styled(format!("{:>w_avail$} ", avail_s), Style::default().fg(theme::C_GREEN)),
        ];
        row.extend(gradient_bar(pct, w_bar));
        row.push(Span::raw("  "));
        row.push(Span::styled(
            format!("{:>w_pct$}%  ", pct as u64),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));
        row.push(Span::styled(disk.mount.clone(), theme::dim_style()));
        lines.push(Line::from(row));

        // I/O sub-row only when there is activity
        if disk.read_bps > 0 || disk.write_bps > 0 || disk.read_iops > 0 || disk.write_iops > 0 {
            lines.push(Line::from(vec![
                Span::styled("    r: ", theme::dim_style()),
                Span::styled(format!("{:<10}", DiskStats::fmt_speed(disk.read_bps)),
                    Style::default().fg(theme::C_TEAL)),
                Span::styled(" w: ", theme::dim_style()),
                Span::styled(format!("{:<10}", DiskStats::fmt_speed(disk.write_bps)),
                    Style::default().fg(theme::C_YELLOW)),
                Span::styled(" riops: ", theme::dim_style()),
                Span::styled(format!("{:<5}", disk.read_iops),
                    Style::default().fg(theme::C_TEAL)),
                Span::styled(" wiops: ", theme::dim_style()),
                Span::styled(format!("{}", disk.write_iops),
                    Style::default().fg(theme::C_YELLOW)),
            ]));
        }
    }

    let block = Block::default()
        .title(Span::styled(" Disk ", theme::title_style()))
        .borders(Borders::ALL)
        .border_style(theme::border_style());

    let scroll_offset = app.scroll_offset as u16;
    f.render_widget(
        Paragraph::new(lines).block(block).scroll((scroll_offset, 0)),
        area,
    );
}
