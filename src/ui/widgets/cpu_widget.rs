use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::app::AppState;
use crate::ui::{braille, theme};

/// Map a 0–100 percentage to a gradient colour: blue→cyan→green→yellow→red
fn gradient_color(pct: f32) -> Color {
    if pct >= 90.0 { Color::Red }
    else if pct >= 70.0 { Color::Yellow }
    else if pct >= 40.0 { Color::Green }
    else if pct >= 15.0 { Color::Cyan }
    else { Color::Blue }
}

/// Build a compact visual bar for a single core: "C00 [████░░░░░░]  3.5%  1.8GHz"
fn core_line(idx: usize, usage: f32, freq_mhz: u64) -> Line<'static> {
    let bar_w = 10usize;
    let filled = ((usage / 100.0) * bar_w as f32).round() as usize;
    let color  = gradient_color(usage);
    let freq_str = if freq_mhz >= 1000 {
        format!("{:.1}GHz", freq_mhz as f64 / 1000.0)
    } else {
        format!("{}MHz", freq_mhz)
    };
    Line::from(vec![
        Span::styled(format!("C{:<2} ", idx), theme::dim_style()),
        Span::raw("["),
        Span::styled("█".repeat(filled),      Style::default().fg(color)),
        Span::styled("░".repeat(bar_w - filled), Style::default().fg(Color::DarkGray)),
        Span::raw("] "),
        Span::styled(format!("{:5.1}%", usage), Style::default().fg(color)),
        Span::styled(format!(" {:>8}", freq_str), theme::dim_style()),
    ])
}

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let state = app.state.read();
    let cpu = &state.cpu;

    // ── dimensions ─────────────────────────────────────────────────────────
    let inner_w  = (area.width  as usize).saturating_sub(2).max(1);
    let inner_h  = (area.height as usize).saturating_sub(2).max(1);

    // We show: 1 global header + 1 load avg row + ceil(cores/2) core rows + graph
    let core_rows  = (cpu.cores.len() + 1) / 2;
    let header_rows = 3; // global bar + load avg + separator
    let graph_h = inner_h.saturating_sub(header_rows + core_rows).max(2);
    let graph_w = inner_w;

    let mut lines: Vec<Line> = Vec::new();

    // ── global usage bar ───────────────────────────────────────────────────
    let global_pct = cpu.global;
    let bar_w   = (inner_w.saturating_sub(30)).max(10).min(40);
    let filled  = ((global_pct / 100.0) * bar_w as f32).round() as usize;
    let bar_col = gradient_color(global_pct);
    lines.push(Line::from(vec![
        Span::styled(format!("CPU({}) ", cpu.count), theme::header_style()),
        Span::raw("["),
        Span::styled("█".repeat(filled),           Style::default().fg(bar_col)),
        Span::styled("░".repeat(bar_w - filled),   Style::default().fg(Color::DarkGray)),
        Span::raw("] "),
        Span::styled(format!("{:5.1}%", global_pct), Style::default().fg(bar_col)),
        Span::raw("  "),
        Span::styled(crate::ui::truncate(&cpu.brand, inner_w.saturating_sub(bar_w + 16)), theme::dim_style()),
    ]));

    // ── load average ───────────────────────────────────────────────────────
    let [l1, l5, l15] = cpu.load_avg;
    let nproc = cpu.count as f64;
    let la_col = |v: f64| {
        let ratio = v / nproc;
        if ratio >= 1.0 { Color::Red }
        else if ratio >= 0.7 { Color::Yellow }
        else { Color::Green }
    };
    lines.push(Line::from(vec![
        Span::styled("Load avg  ", theme::dim_style()),
        Span::styled(format!("{:.2}", l1),  Style::default().fg(la_col(l1))),
        Span::styled(" (1m)  ",             theme::dim_style()),
        Span::styled(format!("{:.2}", l5),  Style::default().fg(la_col(l5))),
        Span::styled(" (5m)  ",             theme::dim_style()),
        Span::styled(format!("{:.2}", l15), Style::default().fg(la_col(l15))),
        Span::styled(" (15m)", theme::dim_style()),
    ]));

    // ── per-core rows (2 cores side-by-side) ──────────────────────────────
    let half = inner_w / 2;
    let cores = &cpu.cores;
    let freqs = &cpu.freqs;
    let get_freq = |i: usize| freqs.get(i).copied().unwrap_or(0);

    for pair_start in (0..cores.len()).step_by(2) {
        let left  = core_line(pair_start,     cores[pair_start],     get_freq(pair_start));
        let right = if pair_start + 1 < cores.len() {
            core_line(pair_start + 1, cores[pair_start + 1], get_freq(pair_start + 1))
        } else {
            Line::from("")
        };

        // Merge two core lines side-by-side with padding
        let mut spans = left.spans.clone();
        // Pad to half width
        let left_len: usize = spans.iter().map(|s| s.content.chars().count()).sum();
        if left_len < half { spans.push(Span::raw(" ".repeat(half - left_len))); }
        spans.extend(right.spans);
        lines.push(Line::from(spans));
    }

    // ── braille history graph (gradient: blue at bottom → red at top) ─────
    let spark_data: Vec<u64> = cpu.history.iter().map(|&v| v as u64).collect();
    let graph_lines = braille::render(&spark_data, graph_w, graph_h);
    let total_graph_rows = graph_lines.len();

    for (row_idx, gl) in graph_lines.iter().enumerate() {
        // top rows = highest values = hotter color
        let ratio = if total_graph_rows <= 1 { 1.0 } else {
            (total_graph_rows - 1 - row_idx) as f64 / (total_graph_rows - 1) as f64
        };
        let color = if ratio >= 0.9      { Color::Red }
                    else if ratio >= 0.7 { Color::Yellow }
                    else if ratio >= 0.4 { Color::Green }
                    else if ratio >= 0.2 { Color::Cyan }
                    else                 { Color::Blue };
        lines.push(Line::from(vec![
            Span::styled(gl.clone(), Style::default().fg(color)),
        ]));
    }

    let block = Block::default()
        .title(Span::styled(" CPU ", theme::title_style()))
        .borders(Borders::ALL);

    f.render_widget(Paragraph::new(lines).block(block), area);
}

