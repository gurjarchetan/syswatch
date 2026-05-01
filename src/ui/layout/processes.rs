use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::app::AppState;
use crate::ui::theme;

fn status_style(status: &str) -> Style {
    match status {
        "R" => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        "Z" => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        "T" => Style::default().fg(Color::Yellow),
        "D" => Style::default().fg(Color::Magenta),
        "X" => Style::default().fg(Color::Red).add_modifier(Modifier::DIM),
        "I" => Style::default().fg(Color::DarkGray),
        _   => Style::default().fg(Color::DarkGray),
    }
}

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let state = app.state.read();
    let summary = state.proc_summary.clone();
    let mut procs = state.processes.clone();
    drop(state); // release lock early

    // Apply filter
    if !app.filter_text.is_empty() {
        let q = app.filter_text.to_lowercase();
        procs.retain(|p| p.name.to_lowercase().contains(&q));
    }

    // Sorting
    match app.sort_col {
        SortCol::Cpu  => procs.sort_by(|a, b| b.cpu_pct.partial_cmp(&a.cpu_pct).unwrap_or(std::cmp::Ordering::Equal)),
        SortCol::Mem  => procs.sort_by(|a, b| b.mem_pct.partial_cmp(&a.mem_pct).unwrap_or(std::cmp::Ordering::Equal)),
        SortCol::Pid  => procs.sort_by_key(|p| p.pid),
        SortCol::Name => procs.sort_by(|a, b| a.name.cmp(&b.name)),
    }

    let mut lines: Vec<Line> = Vec::new();

    // ── summary bar ─────────────────────────────────────────────────────────
    lines.push(Line::from(vec![
        Span::styled("Tasks: ", theme::dim_style()),
        Span::styled(format!("{} total  ", summary.total), theme::header_style()),
        Span::styled("R:", theme::dim_style()),
        Span::styled(format!("{} ", summary.running), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled("  S:", theme::dim_style()),
        Span::styled(format!("{} ", summary.sleeping), Style::default().fg(Color::Cyan)),
        Span::styled("  Z:", theme::dim_style()),
        Span::styled(format!("{} ",
            summary.zombie),
            if summary.zombie > 0 { Style::default().fg(Color::Red).add_modifier(Modifier::BOLD) }
            else { Style::default().fg(Color::DarkGray) }
        ),
        Span::styled("  T:", theme::dim_style()),
        Span::styled(format!("{} ", summary.stopped),  Style::default().fg(Color::Yellow)),
        Span::styled("  O:", theme::dim_style()),
        Span::styled(format!("{}", summary.other), Style::default().fg(Color::DarkGray)),
    ]));

    // ── column header ────────────────────────────────────────────────────────
    lines.push(Line::from(vec![
        Span::styled(
            format!("{:>7}  {:<20} {:<10} {:>7}  {:>7}  {:<3} {:>5} {}",
                "PID", "NAME", "USER", "CPU%", "MEM%", "ST", "THR", "STATUS"),
            theme::header_style(),
        ),
    ]));

    // ── separator ────────────────────────────────────────────────────────────
    let sep_w = (area.width as usize).saturating_sub(2);
    lines.push(Line::from(Span::styled("─".repeat(sep_w), Style::default().fg(Color::DarkGray))));

    // header rows above = 3; border = 2 → need to subtract 5 total
    let visible_rows = (area.height as usize).saturating_sub(5);
    let scroll_off   = app.scroll_offset;

    for (idx, proc) in procs.iter().skip(scroll_off).take(visible_rows).enumerate() {
        let abs_idx  = idx + scroll_off;
        let is_sel   = abs_idx == app.selected_proc;
        let is_kill  = app.kill_confirm && is_sel;

        let row_style = if is_kill {
            Style::default().fg(Color::White).bg(Color::Red).add_modifier(Modifier::BOLD)
        } else if is_sel {
            theme::highlight_style()
        } else {
            Style::default()
        };

        let cpu_color = if is_sel || is_kill { row_style.fg.unwrap_or(Color::White) }
                        else { theme::pct_color_f32(proc.cpu_pct) };
        let mem_color = if is_sel || is_kill { row_style.fg.unwrap_or(Color::White) }
                        else { theme::pct_color_f32(proc.mem_pct) };

        lines.push(Line::from(vec![
            Span::styled(
                format!("{:>7}  {:<20} {:<10} ",
                    proc.pid,
                    crate::ui::truncate(&proc.name, 18),
                    crate::ui::truncate(&proc.user, 8)),
                row_style,
            ),
            Span::styled(format!("{:>7.2}  ", proc.cpu_pct), Style::default().fg(cpu_color)),
            Span::styled(format!("{:>7.2}  ", proc.mem_pct), Style::default().fg(mem_color)),
            Span::styled(format!("{:<3} ", proc.status),     status_style(&proc.status)),
            Span::styled(format!("{:>5} ", proc.threads),    theme::dim_style()),
            Span::styled(proc.status_full.clone(),            status_style(&proc.status)),
        ]));
    }

    // Kill-confirm hint at bottom if active
    if app.kill_confirm {
        lines.push(Line::from(vec![
            Span::styled(
                "  Press k again to send SIGTERM, K for SIGKILL, Esc to cancel  ",
                Style::default().fg(Color::White).bg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]));
    } else if app.filter_mode {
        lines.push(Line::from(vec![
            Span::styled(format!("  Filter: {}▋ ", app.filter_text), theme::header_style()),
        ]));
    }

    let title = format!(
        " Processes ({}) [/filter  f:sort({})  k:kill] ",
        procs.len(),
        app.sort_col.label()
    );

    let block = Block::default()
        .title(Span::styled(title, theme::title_style()))
        .borders(Borders::ALL);

    f.render_widget(Paragraph::new(lines).block(block), area);
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortCol { Cpu, Mem, Pid, Name }

impl SortCol {
    pub fn label(&self) -> &'static str {
        match self {
            SortCol::Cpu  => "CPU",
            SortCol::Mem  => "MEM",
            SortCol::Pid  => "PID",
            SortCol::Name => "NAME",
        }
    }
    pub fn next(self) -> Self {
        match self {
            SortCol::Cpu  => SortCol::Mem,
            SortCol::Mem  => SortCol::Pid,
            SortCol::Pid  => SortCol::Name,
            SortCol::Name => SortCol::Cpu,
        }
    }
}

