use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::app::AppState;
use crate::ui::theme;

fn gradient_color(pct: f32) -> Color {
    if pct >= 90.0 { Color::Red }
    else if pct >= 70.0 { Color::Yellow }
    else if pct >= 40.0 { Color::Green }
    else { Color::Cyan }
}

/// Convert 0-100 value to block char ▁▂▃▄▅▆▇█
fn block_char(v: f32) -> char {
    let blocks = [' ','▁','▂','▃','▄','▅','▆','▇','█'];
    let idx = ((v / 100.0) * 8.0).round().clamp(0.0, 8.0) as usize;
    blocks[idx]
}

fn core_line(idx: usize, usage: f32, freq_mhz: u64) -> Line<'static> {
    let bar_w  = 12usize;
    let filled = ((usage / 100.0) * bar_w as f32).round().clamp(0.0, bar_w as f32) as usize;
    let color  = gradient_color(usage);
    let freq_str = if freq_mhz >= 1000 {
        format!("{:.1}GHz", freq_mhz as f64 / 1000.0)
    } else if freq_mhz > 0 {
        format!("{}MHz", freq_mhz)
    } else {
        String::new()
    };
    Line::from(vec![
        Span::styled(format!("C{:<2}", idx), theme::dim_style()),
        Span::raw(" ▕"),
        Span::styled("█".repeat(filled),           Style::default().fg(color)),
        Span::styled("░".repeat(bar_w - filled),   Style::default().fg(Color::DarkGray)),
        Span::raw("▏ "),
        Span::styled(format!("{:5.1}%", usage), Style::default().fg(color).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" {:>7}", freq_str), theme::dim_style()),
    ])
}

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let state   = app.state.read();
    let cpu     = &state.cpu;
    let inner_w = (area.width  as usize).saturating_sub(2).max(1);
    let inner_h = (area.height as usize).saturating_sub(2).max(1);

    let mut lines: Vec<Line> = Vec::new();

    // ── global bar ─────────────────────────────────────────────────────────
    let g     = cpu.global;
    let bar_w = (inner_w.saturating_sub(28)).max(8).min(36);
    let filled= ((g / 100.0) * bar_w as f32).round().clamp(0.0, bar_w as f32) as usize;
    let gc    = gradient_color(g);
    let brand = crate::ui::truncate(&cpu.brand, inner_w.saturating_sub(bar_w + 22));
    lines.push(Line::from(vec![
        Span::styled(format!("CPU×{} ", cpu.count), theme::header_style()),
        Span::raw("▕"),
        Span::styled("█".repeat(filled),         Style::default().fg(gc)),
        Span::styled("░".repeat(bar_w - filled), Style::default().fg(Color::DarkGray)),
        Span::raw("▏ "),
        Span::styled(format!("{:5.1}%", g), Style::default().fg(gc).add_modifier(Modifier::BOLD)),
        Span::styled(format!("  {}", brand), theme::dim_style()),
    ]));

    // ── load average ───────────────────────────────────────────────────────
    let [l1, l5, l15] = cpu.load_avg;
    let nproc = cpu.count as f64;
    let la_col = |v: f64| {
        if v / nproc >= 1.0 { Color::Red }
        else if v / nproc >= 0.7 { Color::Yellow }
        else { Color::Green }
    };
    lines.push(Line::from(vec![
        Span::styled("Load avg ", theme::dim_style()),
        Span::styled(format!("{:.2}", l1),  Style::default().fg(la_col(l1))),
        Span::styled(" 1m  ",              theme::dim_style()),
        Span::styled(format!("{:.2}", l5),  Style::default().fg(la_col(l5))),
        Span::styled(" 5m  ",              theme::dim_style()),
        Span::styled(format!("{:.2}", l15), Style::default().fg(la_col(l15))),
        Span::styled(" 15m",               theme::dim_style()),
    ]));

    // ── separator ─────────────────────────────────────────────────────────
    lines.push(Line::from(Span::styled("─".repeat(inner_w), Style::default().fg(Color::DarkGray))));

    // ── per-core grid (2 columns) ─────────────────────────────────────────
    let cores = &cpu.cores;
    let freqs = &cpu.freqs;
    let get_freq = |i: usize| freqs.get(i).copied().unwrap_or(0);
    let two_col = inner_w >= 52;

    for i in (0..cores.len()).step_by(if two_col { 2 } else { 1 }) {
        if two_col && i + 1 < cores.len() {
            let left  = core_line(i,     cores[i],     get_freq(i));
            let right = core_line(i + 1, cores[i + 1], get_freq(i + 1));
            let mut spans = left.spans.clone();
            let left_len: usize = spans.iter().map(|s| s.content.chars().count()).sum();
            let pad = (inner_w / 2).saturating_sub(left_len);
            if pad > 0 { spans.push(Span::raw(" ".repeat(pad))); }
            spans.push(Span::styled("│", Style::default().fg(Color::DarkGray)));
            spans.extend(right.spans);
            lines.push(Line::from(spans));
        } else {
            lines.push(core_line(i, cores[i], get_freq(i)));
        }
    }

    // ── history graph with Y-axis ─────────────────────────────────────────
    let core_rows = (cores.len() + if two_col { 1 } else { 0 }) / if two_col { 2 } else { 1 };
    let graph_h   = inner_h.saturating_sub(4 + core_rows).max(3); // 2 header + sep + sep
    let graph_w   = inner_w.saturating_sub(6); // 6 chars for Y-axis "100% |"
    let history   = &cpu.history;

    lines.push(Line::from(Span::styled("─".repeat(inner_w), Style::default().fg(Color::DarkGray))));

    if history.is_empty() {
        lines.push(Line::from(Span::styled("  Collecting CPU history…", theme::dim_style())));
    } else {
        let data_len = history.len();
        for row in 0..graph_h {
            // row 0 = top = 100%, row graph_h-1 = bottom = ~0%
            let row_top = 100.0 * (graph_h - row)     as f32 / graph_h as f32;
            let row_bot = 100.0 * (graph_h - row - 1) as f32 / graph_h as f32;

            let label = if row == 0             { format!("100%│") }
                        else if row == graph_h/2 { format!(" 50%│") }
                        else if row == graph_h-1 { format!("  0%│") }
                        else                     { format!("    │") };

            let mut spans = vec![Span::styled(label, theme::dim_style())];

            for col in 0..graph_w {
                let hist_idx = if data_len >= graph_w {
                    data_len - graph_w + col
                } else {
                    if col < graph_w - data_len {
                        spans.push(Span::raw(" "));
                        continue;
                    }
                    col - (graph_w - data_len)
                };
                let val = history[hist_idx];
                let color = gradient_color(val);
                let ch = if val >= row_top {
                    '█'
                } else if val > row_bot {
                    let frac = (val - row_bot) / (row_top - row_bot);
                    block_char(frac * 100.0)
                } else {
                    ' '
                };
                if ch == ' ' {
                    spans.push(Span::raw(" "));
                } else {
                    spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
                }
            }
            lines.push(Line::from(spans));
        }
    }

    let block = Block::default()
        .title(Span::styled(" CPU ", theme::title_style()))
        .borders(Borders::ALL);

    f.render_widget(Paragraph::new(lines).block(block), area);
}
