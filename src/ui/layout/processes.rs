use crate::app::AppState;
use crate::collector::memory::MemStats;
use crate::ui::theme;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

fn status_style(status: &str) -> Style {
    match status {
        "R" => Style::default()
            .fg(theme::C_GREEN)
            .add_modifier(Modifier::BOLD),
        "Z" => Style::default()
            .fg(theme::C_RED)
            .add_modifier(Modifier::BOLD),
        "T" => Style::default().fg(theme::C_YELLOW),
        "D" => Style::default().fg(theme::C_MAGENTA),
        "X" => Style::default()
            .fg(theme::C_RED)
            .add_modifier(Modifier::DIM),
        _ => Style::default().fg(theme::C_DIM),
    }
}

/// Tiny inline bar for CPU/MEM columns  ████░░  (6 chars)
fn pct_bar(pct: f32, width: usize) -> String {
    let filled = ((pct / 100.0) * width as f32)
        .round()
        .clamp(0.0, width as f32) as usize;
    let mut s = String::with_capacity(width * 3); // '█' and '░' are each 3 bytes
    for _ in 0..filled {
        s.push('█');
    }
    for _ in filled..width {
        s.push('░');
    }
    s
}

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    // Use pre-sorted+filtered cache — zero allocation on 6 of every 7 frames.
    let procs = &app.proc_cache;
    let summary = &app.proc_summary_cache;

    let mut lines: Vec<Line> = Vec::new();

    // ── summary bar ───────────────────────────────────────────────────────
    lines.push(Line::from(vec![
        Span::styled("Tasks ", theme::dim_style()),
        Span::styled(format!("{}", summary.total), theme::header_style()),
        Span::styled("  Run ", theme::dim_style()),
        Span::styled(
            format!("{}", summary.running),
            Style::default()
                .fg(theme::C_GREEN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  Sleep ", theme::dim_style()),
        Span::styled(
            format!("{}", summary.sleeping),
            Style::default().fg(theme::C_BLUE),
        ),
        Span::styled("  Idle ", theme::dim_style()),
        Span::styled(format!("{}", summary.other), theme::dim_style()),
        Span::styled("  Stop ", theme::dim_style()),
        Span::styled(
            format!("{}", summary.stopped),
            Style::default().fg(theme::C_YELLOW),
        ),
        Span::styled("  Zombie ", theme::dim_style()),
        Span::styled(
            format!("{}", summary.zombie),
            if summary.zombie > 0 {
                Style::default()
                    .fg(theme::C_RED)
                    .add_modifier(Modifier::BOLD)
            } else {
                theme::dim_style()
            },
        ),
    ]));

    // ── sort toolbar ──────────────────────────────────────────────────────
    let sort_btn = |label: &'static str, active: bool| -> Span<'static> {
        if active {
            Span::styled(
                format!(" {}▼ ", label),
                Style::default()
                    .fg(ratatui::style::Color::Rgb(10, 10, 20))
                    .bg(theme::C_ACCENT)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(format!(" {} ", label), theme::dim_style())
        }
    };
    lines.push(Line::from(vec![
        Span::styled("Sort:", theme::dim_style()),
        sort_btn("CPU", app.sort_col == SortCol::Cpu),
        sort_btn("MEM", app.sort_col == SortCol::Mem),
        sort_btn("PID", app.sort_col == SortCol::Pid),
        sort_btn("NAME", app.sort_col == SortCol::Name),
        Span::styled(
            "   /=filter  k=kill  K=SIGKILL  Esc=cancel",
            theme::dim_style(),
        ),
    ]));

    // ── column header ─────────────────────────────────────────────────────
    let mini_w = 6usize;
    lines.push(Line::from(vec![
        Span::styled(format!("{:>7}  ", "PID"), theme::header_style()),
        Span::styled(format!("{:<18} ", "NAME"), theme::header_style()),
        Span::styled(format!("{:<9} ", "USER"), theme::header_style()),
        Span::styled(
            format!("{:>5}  {:>mini_w$}  ", "CPU%", ""),
            Style::default()
                .fg(theme::C_TEAL)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:>5}  {:>mini_w$}  ", "MEM%", ""),
            Style::default()
                .fg(theme::C_BLUE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("{:<3} {:>4}  ", "ST", "THR"), theme::header_style()),
    ]));

    // ── separator ─────────────────────────────────────────────────────────
    let sep_w = (area.width as usize).saturating_sub(2);
    lines.push(Line::from(Span::styled(
        "─".repeat(sep_w),
        theme::border_style(),
    )));

    // header = 4 rows, border = 2 → 6
    let visible_rows = (area.height as usize).saturating_sub(6);
    let scroll_off = app.scroll_offset;

    for (idx, proc) in procs.iter().skip(scroll_off).take(visible_rows).enumerate() {
        let abs_idx = idx + scroll_off;
        let is_sel = abs_idx == app.selected_proc;
        let is_kill = app.kill_confirm && is_sel;

        let row_style = if is_kill {
            Style::default()
                .fg(ratatui::style::Color::Rgb(255, 255, 255))
                .bg(theme::C_RED)
                .add_modifier(Modifier::BOLD)
        } else if is_sel {
            theme::highlight_style()
        } else {
            Style::default()
        };

        let cpu_color = if is_sel || is_kill {
            row_style.fg.unwrap_or(theme::C_WHITE)
        } else {
            theme::pct_color_f32(proc.cpu_pct)
        };
        let mem_color = if is_sel || is_kill {
            row_style.fg.unwrap_or(theme::C_WHITE)
        } else {
            theme::pct_color_f32(proc.mem_pct)
        };

        let cpu_bar = pct_bar(proc.cpu_pct, mini_w);
        let mem_bar = pct_bar(proc.mem_pct, mini_w);

        lines.push(Line::from(vec![
            Span::styled(format!("{:>7}  ", proc.pid), row_style),
            Span::styled(
                format!("{:<18} ", crate::ui::truncate(&proc.name, 16)),
                row_style,
            ),
            Span::styled(
                format!("{:<9} ", crate::ui::truncate(&proc.user, 8)),
                row_style,
            ),
            Span::styled(
                format!("{:>5.1}  ", proc.cpu_pct),
                Style::default().fg(cpu_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(cpu_bar, Style::default().fg(cpu_color)),
            Span::raw("  "),
            Span::styled(
                format!("{:>5.1}  ", proc.mem_pct),
                Style::default().fg(mem_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(mem_bar, Style::default().fg(mem_color)),
            Span::raw("  "),
            Span::styled(format!("{:<3} ", proc.status), status_style(proc.status)),
            Span::styled(format!("{:>4}  ", proc.threads), theme::dim_style()),
        ]));
    }

    if app.kill_confirm {
        lines.push(Line::from(vec![Span::styled(
            "  ⚠  k=SIGTERM  K=SIGKILL  Esc=cancel  ",
            Style::default()
                .fg(ratatui::style::Color::Rgb(255, 255, 255))
                .bg(theme::C_RED)
                .add_modifier(Modifier::BOLD),
        )]));
    } else if app.filter_mode {
        lines.push(Line::from(vec![Span::styled(
            format!("  Filter ▸ {}▋", app.filter_text),
            Style::default()
                .fg(theme::C_YELLOW)
                .add_modifier(Modifier::BOLD),
        )]));
    }

    let title = format!(" Processes ({}) ", procs.len());
    let block = Block::default()
        .title(Span::styled(title, theme::title_style()))
        .borders(Borders::ALL)
        .border_style(theme::border_style());

    f.render_widget(Paragraph::new(lines).block(block), area);

    // Overlay detail popup if requested
    if app.show_proc_detail {
        render_detail_popup(f, area, app, &procs);
    }
}

fn render_detail_popup(
    f: &mut Frame,
    parent: Rect,
    app: &AppState,
    procs: &[crate::collector::process::ProcessInfo],
) {
    let Some(proc) = app
        .detail_pid
        .and_then(|pid| procs.iter().find(|p| p.pid == pid))
    else {
        return;
    };

    // ── Popup geometry: centred, 50 wide × 16 tall ────────────────────────
    let popup_w: u16 = 52.min(parent.width.saturating_sub(4));
    let popup_h: u16 = 16.min(parent.height.saturating_sub(4));
    let popup_x = parent.x + (parent.width.saturating_sub(popup_w)) / 2;
    let popup_y = parent.y + (parent.height.saturating_sub(popup_h)) / 2;
    let popup_area = Rect {
        x: popup_x,
        y: popup_y,
        width: popup_w,
        height: popup_h,
    };

    f.render_widget(Clear, popup_area);

    let inner_w = (popup_w as usize).saturating_sub(4).max(20);
    let bar_w = inner_w.saturating_sub(12).clamp(8, 20);

    // Helper: gradient bar
    let make_bar = |pct: f32| -> Vec<Span<'static>> {
        let filled = ((pct / 100.0) * bar_w as f32)
            .round()
            .clamp(0.0, bar_w as f32) as usize;
        let color = theme::pct_color_f32(pct);
        let mut v = vec![Span::styled("▕", theme::dim_style())];
        for i in 0..bar_w {
            let pos_pct = (i as f64 / bar_w as f64) * 100.0;
            let ch = if i < filled { "█" } else { "░" };
            let c = if i < filled {
                theme::pct_color(pos_pct.max(pct as f64 * 0.3))
            } else {
                theme::C_BORDER
            };
            v.push(Span::styled(ch, Style::default().fg(c)));
        }
        v.push(Span::styled("▏", theme::dim_style()));
        v.push(Span::styled(
            format!(" {:5.1}%", pct),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));
        v
    };

    let sep = "─".repeat(inner_w);

    let st_color = match proc.status {
        "R" => theme::C_GREEN,
        "Z" => theme::C_RED,
        "T" => theme::C_YELLOW,
        "D" => theme::C_MAGENTA,
        _ => theme::C_DIM,
    };

    let mut lines: Vec<Line> = vec![
        // Name + PID
        Line::from(vec![
            Span::styled(
                format!("  {}", proc.name),
                Style::default()
                    .fg(theme::C_ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("  pid {}", proc.pid), theme::dim_style()),
        ]),
        Line::from(Span::styled(sep.clone(), theme::border_style())),
        // User + status
        Line::from(vec![
            Span::styled("  User   ", theme::dim_style()),
            Span::styled(proc.user.clone(), Style::default().fg(theme::C_WHITE)),
            Span::styled("   Status  ", theme::dim_style()),
            Span::styled(
                format!("{} ({})", proc.status_full, proc.status),
                Style::default().fg(st_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        // Threads + memory
        Line::from(vec![
            Span::styled("  Threads ", theme::dim_style()),
            Span::styled(
                format!("{}", proc.threads),
                Style::default()
                    .fg(theme::C_WHITE)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("   Memory  ", theme::dim_style()),
            Span::styled(
                MemStats::fmt_kb(proc.mem_kb),
                Style::default()
                    .fg(theme::C_BLUE)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(Span::styled(sep.clone(), theme::border_style())),
    ];

    // CPU bar
    let mut cpu_row = vec![Span::styled("  CPU   ", theme::dim_style())];
    cpu_row.extend(make_bar(proc.cpu_pct));
    lines.push(Line::from(cpu_row));

    // MEM bar
    let mut mem_row = vec![Span::styled("  MEM   ", theme::dim_style())];
    mem_row.extend(make_bar(proc.mem_pct));
    lines.push(Line::from(mem_row));

    lines.push(Line::from(Span::styled(sep.clone(), theme::border_style())));

    // Hints
    lines.push(Line::from(vec![
        Span::styled("  Enter/Esc ", theme::dim_style()),
        Span::styled("close", Style::default().fg(theme::C_ACCENT)),
        Span::styled("   k ", theme::dim_style()),
        Span::styled("SIGTERM", Style::default().fg(theme::C_ORANGE)),
        Span::styled("   K ", theme::dim_style()),
        Span::styled("SIGKILL", Style::default().fg(theme::C_RED)),
    ]));

    let block = Block::default()
        .title(Span::styled(" Process Detail ", theme::title_style()))
        .borders(Borders::ALL)
        .border_style(theme::border_style_active()); // accent border to stand out

    f.render_widget(Paragraph::new(lines).block(block), popup_area);
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortCol {
    Cpu,
    Mem,
    Pid,
    Name,
}

impl SortCol {
    pub fn label(&self) -> &'static str {
        match self {
            SortCol::Cpu => "CPU",
            SortCol::Mem => "MEM",
            SortCol::Pid => "PID",
            SortCol::Name => "NAME",
        }
    }
    pub fn next(self) -> Self {
        match self {
            SortCol::Cpu => SortCol::Mem,
            SortCol::Mem => SortCol::Pid,
            SortCol::Pid => SortCol::Name,
            SortCol::Name => SortCol::Cpu,
        }
    }
}
