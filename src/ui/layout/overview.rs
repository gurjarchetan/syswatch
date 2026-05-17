use crate::app::AppState;
use crate::collector::{disk::DiskStats, network::NetStats};
use crate::ui::{braille, theme, widgets};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Overview: IP bar (1 line) | left [CPU small + Proc snapshot] | right [Mem/Net/Disk]
pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    render_ip_bar(f, rows[0], app);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(rows[1]);

    // Left column: CPU (compact, ~42%) + Process snapshot (~58%)
    let left_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(cols[0]);

    let right_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(cols[1]);

    widgets::cpu_widget::render(f, left_rows[0], app);
    render_proc_snapshot(f, left_rows[1], app);
    widgets::mem_widget::render(f, right_rows[0], app);
    render_net_summary(f, right_rows[1], app);
    render_disk_summary(f, right_rows[2], app);
}

fn render_ip_bar(f: &mut Frame, area: Rect, app: &AppState) {
    let state = app.state.read();
    let line = Line::from(vec![
        Span::styled(
            "  ◈ ",
            Style::default()
                .fg(theme::C_ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Private ", theme::dim_style()),
        Span::styled(
            state.private_ip.clone(),
            Style::default()
                .fg(theme::C_TEAL)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "   ◈ ",
            Style::default()
                .fg(theme::C_ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Public  ", theme::dim_style()),
        Span::styled(
            state.public_ip.clone(),
            Style::default()
                .fg(theme::C_MAGENTA)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    f.render_widget(Paragraph::new(line).alignment(Alignment::Left), area);
}

/// Compact top-process snapshot for F1 — no interaction, sorted by CPU desc
fn render_proc_snapshot(f: &mut Frame, area: Rect, app: &AppState) {
    // Hold the read lock only for the duration of this function.
    // state.processes is already sorted by CPU descending by the collector,
    // so no clone or re-sort is needed here.
    let state = app.state.read();
    let summary_total = state.proc_summary.total;
    let summary_running = state.proc_summary.running;
    let summary_sleeping = state.proc_summary.sleeping;
    let summary_other = state.proc_summary.other;
    let summary_zombie = state.proc_summary.zombie;

    let inner_w = (area.width as usize).saturating_sub(2).max(20);
    // Fixed columns: 7(pid)+2+16(name)+8(user)+space(1) = 34 + 2×(8+2) value cols + 4 status
    // Available for 2 bars: inner_w - 50
    let bar_w = ((inner_w.saturating_sub(50)) / 2).clamp(4, 8);
    let sep_w = inner_w;
    // visible rows: area height - 2 (border) - 2 (summary + colhdr) - 1 (sep)
    let max_rows = (area.height as usize).saturating_sub(5).max(1);

    let mut lines: Vec<Line> = Vec::new();

    // ── summary pill row ──────────────────────────────────────────────────
    lines.push(Line::from(vec![
        Span::styled("Tasks ", theme::dim_style()),
        Span::styled(format!("{}", summary_total), theme::header_style()),
        Span::raw("  "),
        Span::styled("● ", Style::default().fg(theme::C_GREEN)),
        Span::styled(
            format!("{} run", summary_running),
            Style::default()
                .fg(theme::C_GREEN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{} sleep", summary_sleeping),
            Style::default().fg(theme::C_BLUE),
        ),
        Span::raw("  "),
        Span::styled(format!("{} idle", summary_other), theme::dim_style()),
        if summary_zombie > 0 {
            Span::styled(
                format!("  ⚠ {} zombie", summary_zombie),
                Style::default()
                    .fg(theme::C_RED)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::raw("")
        },
    ]));

    // ── column header ─────────────────────────────────────────────────────
    lines.push(Line::from(vec![
        Span::styled(
            format!("{:>7}  {:<16} {:<8}", "PID", "NAME", "USER"),
            theme::dim_style(),
        ),
        Span::styled(
            format!(" {:>5}  {:>bar_w$}", "CPU%", ""),
            Style::default()
                .fg(theme::C_TEAL)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  {:>5}  {:>bar_w$}", "MEM%", ""),
            Style::default()
                .fg(theme::C_BLUE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ST", theme::dim_style()),
    ]));

    lines.push(Line::from(Span::styled(
        "─".repeat(sep_w),
        theme::border_style(),
    )));

    // ── process rows ──────────────────────────────────────────────────────
    for proc in state.processes.iter().take(max_rows) {
        let cpu_color = theme::pct_color_f32(proc.cpu_pct);
        let mem_color = theme::pct_color_f32(proc.mem_pct);

        let cpu_filled = ((proc.cpu_pct / 100.0) * bar_w as f32)
            .round()
            .clamp(0.0, bar_w as f32) as usize;
        let mem_filled = ((proc.mem_pct / 100.0) * bar_w as f32)
            .round()
            .clamp(0.0, bar_w as f32) as usize;
        let mut cpu_bar = String::with_capacity(bar_w * 3);
        for i in 0..bar_w {
            cpu_bar.push(if i < cpu_filled { '█' } else { '░' });
        }
        let mut mem_bar = String::with_capacity(bar_w * 3);
        for i in 0..bar_w {
            mem_bar.push(if i < mem_filled { '█' } else { '░' });
        }

        let st_style = match proc.status {
            "R" => Style::default()
                .fg(theme::C_GREEN)
                .add_modifier(Modifier::BOLD),
            "Z" => Style::default()
                .fg(theme::C_RED)
                .add_modifier(Modifier::BOLD),
            "T" => Style::default().fg(theme::C_YELLOW),
            _ => theme::dim_style(),
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!(
                    "{:>7}  {:<16} {:<8}",
                    proc.pid,
                    crate::ui::truncate(&proc.name, 14),
                    crate::ui::truncate(&proc.user, 7)
                ),
                theme::dim_style(),
            ),
            Span::styled(
                format!(" {:>5.1}  ", proc.cpu_pct),
                Style::default().fg(cpu_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(cpu_bar, Style::default().fg(cpu_color)),
            Span::styled(
                format!("  {:>5.1}  ", proc.mem_pct),
                Style::default().fg(mem_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(mem_bar, Style::default().fg(mem_color)),
            Span::styled(format!("  {}", proc.status), st_style),
        ]));
    }

    let title = " Processes — top by CPU (F2 for full view) ".to_string();
    let block = Block::default()
        .title(Span::styled(title, theme::title_style()))
        .borders(Borders::ALL)
        .border_style(theme::border_style());

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_net_summary(f: &mut Frame, area: Rect, app: &AppState) {
    let inner_w = (area.width as usize).saturating_sub(2).max(20);
    let inner_h = (area.height as usize).saturating_sub(2).max(1);

    // Pre-compute sizes (independent of state) so we can hold the lock briefly.
    let remaining = inner_h.saturating_sub(3);
    let each_h = (remaining / 2).max(2);
    let y_w = 4usize;
    let graph_w = inner_w.saturating_sub(y_w + 2).max(4);

    // Acquire read lock once; compute braille rows inside to avoid cloning histories.
    let (rx_bps, tx_bps, total_rx, total_tx, rx_peak, tx_peak, rx_rows, tx_rows) = {
        let state = app.state.read();
        let rx_peak = state.network.rx_history.iter().max().copied().unwrap_or(0);
        let tx_peak = state.network.tx_history.iter().max().copied().unwrap_or(0);
        let (rx_rows, tx_rows) = if each_h >= 2 {
            (
                braille::render(&state.network.rx_history, graph_w, each_h),
                braille::render(&state.network.tx_history, graph_w, each_h),
            )
        } else {
            (vec![], vec![])
        };
        (
            state.network.rx_bps,
            state.network.tx_bps,
            state.network.total_rx,
            state.network.total_tx,
            rx_peak,
            tx_peak,
            rx_rows,
            tx_rows,
        )
    };

    let mut lines: Vec<Line> = Vec::new();

    // Speed summary rows
    lines.push(Line::from(vec![
        Span::styled(
            "▼ RX ",
            Style::default()
                .fg(theme::C_TEAL)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            NetStats::fmt_speed(rx_bps),
            Style::default()
                .fg(theme::C_WHITE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  total {}", NetStats::fmt_bytes(total_rx)),
            theme::dim_style(),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            "▲ TX ",
            Style::default()
                .fg(theme::C_MAGENTA)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            NetStats::fmt_speed(tx_bps),
            Style::default()
                .fg(theme::C_WHITE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  total {}", NetStats::fmt_bytes(total_tx)),
            theme::dim_style(),
        ),
    ]));

    // ── Braille graphs: RX above, TX below ───────────────────────────────
    if each_h >= 2 {
        lines.push(Line::from(Span::styled(
            "─".repeat(inner_w),
            theme::border_style(),
        )));

        // RX graph
        for (i, row_str) in rx_rows.iter().enumerate() {
            let spd = if i == 0 {
                shorten_speed(rx_peak)
            } else {
                "    ".to_string()
            };
            lines.push(Line::from(vec![
                Span::styled(format!("{}|", spd), theme::dim_style()),
                Span::styled(row_str.clone(), Style::default().fg(theme::C_TEAL)),
                Span::styled("|", theme::dim_style()),
            ]));
        }
        lines.push(Line::from(Span::styled(
            format!("    └{}┘ ▼ RX", "─".repeat(graph_w)),
            Style::default()
                .fg(theme::C_TEAL)
                .add_modifier(Modifier::DIM),
        )));

        // TX graph
        for (i, row_str) in tx_rows.iter().enumerate() {
            let spd = if i == 0 {
                shorten_speed(tx_peak)
            } else {
                "    ".to_string()
            };
            lines.push(Line::from(vec![
                Span::styled(format!("{}|", spd), theme::dim_style()),
                Span::styled(row_str.clone(), Style::default().fg(theme::C_MAGENTA)),
                Span::styled("|", theme::dim_style()),
            ]));
        }
        lines.push(Line::from(Span::styled(
            format!("    └{}┘ ▲ TX", "─".repeat(graph_w)),
            Style::default()
                .fg(theme::C_MAGENTA)
                .add_modifier(Modifier::DIM),
        )));
    }

    let block = Block::default()
        .title(Span::styled(" Network ", theme::title_style()))
        .borders(Borders::ALL)
        .border_style(theme::border_style());

    f.render_widget(Paragraph::new(lines).block(block), area);
}

/// 4-char-wide abbreviated bandwidth label for graph y-axis (e.g. "87K ", " 2M ")
fn shorten_speed(bps: u64) -> String {
    if bps == 0 {
        "   0".to_string()
    } else if bps >= 1_073_741_824 {
        format!("{:>3}G", bps / 1_073_741_824)
    } else if bps >= 1_048_576 {
        format!("{:>3}M", bps / 1_048_576)
    } else if bps >= 1024 {
        format!("{:>3}K", bps / 1024)
    } else {
        format!("{:>3}B", bps)
    }
}

fn render_disk_summary(f: &mut Frame, area: Rect, app: &AppState) {
    let state = app.state.read();
    let disks = &state.disks;
    let inner_w = (area.width as usize).saturating_sub(2).max(20);
    let bar_w = 8usize;
    let mut lines: Vec<Line> = Vec::new();

    if disks.is_empty() {
        lines.push(Line::from(Span::styled(
            "No disks found",
            theme::dim_style(),
        )));
    } else {
        for disk in disks {
            let pct = disk.used_pct();
            let color = theme::pct_color(pct);
            let filled = ((pct / 100.0) * bar_w as f64)
                .round()
                .clamp(0.0, bar_w as f64) as usize;

            // Gradient bar spans
            let mut bar_spans: Vec<Span> = Vec::with_capacity(bar_w + 2);
            bar_spans.push(Span::styled("▕", theme::dim_style()));
            for i in 0..bar_w {
                let pos_pct = (i as f64 / bar_w as f64) * 100.0;
                let ch = if i < filled { "█" } else { "░" };
                let c = if i < filled {
                    theme::pct_color(pos_pct.max(pct * 0.3))
                } else {
                    theme::C_BORDER
                };
                bar_spans.push(Span::styled(ch, Style::default().fg(c)));
            }
            bar_spans.push(Span::styled("▏", theme::dim_style()));

            // % + size + mount on same line
            bar_spans.push(Span::styled(
                format!(" {:>3}%", pct as u64),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ));
            let size_s = DiskStats::fmt_size(disk.total_kb);
            let mount_max = inner_w.saturating_sub(bar_w + 16);
            bar_spans.push(Span::styled(format!(" {:>6}", size_s), theme::dim_style()));
            bar_spans.push(Span::styled(
                format!(" {}", crate::ui::truncate(&disk.mount, mount_max)),
                theme::dim_style(),
            ));
            lines.push(Line::from(bar_spans));
        }
    }

    let block = Block::default()
        .title(Span::styled(" Disk ", theme::title_style()))
        .borders(Borders::ALL)
        .border_style(theme::border_style());

    f.render_widget(Paragraph::new(lines).block(block), area);
}
