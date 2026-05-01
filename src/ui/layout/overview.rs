use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};
use crate::app::AppState;
use crate::ui::widgets;

/// Overview: IP bar (top 1 line) | CPU (left) | Memory + Network + Disk summary (right)
pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    // Split vertically: 1-line IP strip + rest
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    render_ip_bar(f, rows[0], app);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(rows[1]);

    let right_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(cols[1]);

    widgets::cpu_widget::render(f, cols[0], app);
    widgets::mem_widget::render(f, right_rows[0], app);
    render_net_summary(f, right_rows[1], app);
    render_disk_summary(f, right_rows[2], app);
}

fn render_ip_bar(f: &mut Frame, area: Rect, app: &AppState) {
    use ratatui::{
        layout::Alignment,
        text::{Line, Span},
        widgets::Paragraph,
    };
    use crate::ui::theme;

    let state = app.state.read();

    let line = Line::from(vec![
        Span::styled("  ⬡ Private IP: ", theme::dim_style()),
        Span::styled(state.private_ip.clone(), theme::header_style()),
        Span::styled("   ⬡ Public IP:  ", theme::dim_style()),
        Span::styled(state.public_ip.clone(), theme::header_style()),
    ]);

    f.render_widget(Paragraph::new(line).alignment(Alignment::Left), area);
}

fn render_net_summary(f: &mut Frame, area: Rect, app: &AppState) {
    use ratatui::{
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph},
    };
    use crate::collector::network::NetStats;
    use crate::ui::theme;

    let state = app.state.read();
    let net = &state.network;

    let lines = vec![
        Line::from(vec![
            Span::styled("▲ TX ", theme::header_style()),
            Span::raw(NetStats::fmt_speed(net.tx_bps)),
            Span::raw(format!("  Total: {}", NetStats::fmt_bytes(net.total_tx))),
        ]),
        Line::from(vec![
            Span::styled("▼ RX ", theme::header_style()),
            Span::raw(NetStats::fmt_speed(net.rx_bps)),
            Span::raw(format!("  Total: {}", NetStats::fmt_bytes(net.total_rx))),
        ]),
    ];

    let block = Block::default()
        .title(Span::styled(" Network ", theme::title_style()))
        .borders(Borders::ALL);

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_disk_summary(f: &mut Frame, area: Rect, app: &AppState) {
    use ratatui::{
        style::{Color, Style, Modifier},
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph},
    };
    use crate::collector::disk::DiskStats;
    use crate::ui::theme;

    let state = app.state.read();
    let disks = &state.disks;

    let inner_w = (area.width as usize).saturating_sub(4).max(20);
    let bar_w = 10usize;

    let mut lines: Vec<Line> = Vec::new();

    if disks.is_empty() {
        lines.push(Line::from(Span::styled("No disks found", theme::dim_style())));
    } else {
        for disk in disks {
            let pct = disk.used_pct();
            let color = if pct >= 90.0 { Color::Red }
                else if pct >= 75.0 { Color::Yellow }
                else { Color::Green };
            let filled = ((pct / 100.0) * bar_w as f64).round().clamp(0.0, bar_w as f64) as usize;
            let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(bar_w - filled));
            let size_s = DiskStats::fmt_size(disk.total_kb);
            let mount = crate::ui::truncate(&disk.mount, inner_w.saturating_sub(bar_w + 18));
            lines.push(Line::from(vec![
                Span::styled(format!("{} ", bar), Style::default().fg(color)),
                Span::styled(format!("{:>3}%", pct as u64), Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::raw(format!(" {:>6}  ", size_s)),
                Span::styled(mount, theme::dim_style()),
            ]));
        }
    }

    let block = Block::default()
        .title(Span::styled(" Disk ", theme::title_style()))
        .borders(Borders::ALL);

    f.render_widget(Paragraph::new(lines).block(block), area);
}
