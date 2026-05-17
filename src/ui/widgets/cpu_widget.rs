use crate::app::AppState;
use crate::ui::{braille, theme};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

fn grad(pct: f32) -> Style {
    Style::default().fg(theme::pct_color_f32(pct))
}

/// Thin mini-bar for per-core compact row (8 chars wide)
fn mini_bar(pct: f32, width: usize) -> String {
    let filled = ((pct / 100.0) * width as f32)
        .round()
        .clamp(0.0, width as f32) as usize;
    let mut s = String::with_capacity(width * 3); // '━' and '─' are each 3 bytes
    for _ in 0..filled {
        s.push('━');
    }
    for _ in filled..width {
        s.push('─');
    }
    s
}

fn freq_str(mhz: u64) -> String {
    if mhz >= 1000 {
        format!("{:.1}GHz", mhz as f64 / 1000.0)
    } else if mhz > 0 {
        format!("{}MHz", mhz)
    } else {
        "─────".to_string()
    }
}

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let state = app.state.read();
    let cpu = &state.cpu;
    let inner_w = (area.width as usize).saturating_sub(2).max(1);
    let inner_h = (area.height as usize).saturating_sub(2).max(1);

    let mut lines: Vec<Line> = Vec::new();

    // ── Global header bar ─────────────────────────────────────────────────
    let g = cpu.global;
    let bar_w = (inner_w.saturating_sub(30)).clamp(10, 40);
    let filled = ((g / 100.0) * bar_w as f32)
        .round()
        .clamp(0.0, bar_w as f32) as usize;

    // gradient fill: each char gets its own colour based on position
    let mut bar_spans: Vec<Span> = Vec::with_capacity(bar_w);
    for i in 0..bar_w {
        let position_pct = (i as f32 / bar_w as f32) * 100.0;
        let ch = if i < filled { "█" } else { "░" };
        let color = if i < filled {
            theme::pct_color_f32(position_pct.max(g * 0.3))
        } else {
            theme::C_BORDER
        };
        bar_spans.push(Span::styled(ch, Style::default().fg(color)));
    }

    let brand = crate::ui::truncate(&cpu.brand, inner_w.saturating_sub(bar_w + 22));
    let mut head = vec![
        Span::styled(format!("CPU×{:<2} ", cpu.count), theme::header_style()),
        Span::styled("▕", theme::dim_style()),
    ];
    head.extend(bar_spans);
    head.push(Span::styled("▏", theme::dim_style()));
    head.push(Span::styled(
        format!(" {:5.1}%", g),
        grad(g).add_modifier(Modifier::BOLD),
    ));
    head.push(Span::styled(format!("  {}", brand), theme::dim_style()));
    lines.push(Line::from(head));

    // ── Load average ──────────────────────────────────────────────────────
    let [l1, l5, l15] = cpu.load_avg;
    let nproc = cpu.count as f64;
    let lc = |v: f64| {
        if v / nproc >= 1.0 {
            theme::C_RED
        } else if v / nproc >= 0.7 {
            theme::C_ORANGE
        } else if v / nproc >= 0.4 {
            theme::C_YELLOW
        } else {
            theme::C_GREEN
        }
    };
    lines.push(Line::from(vec![
        Span::styled("Load  ", theme::dim_style()),
        Span::styled(
            format!("{:.2}", l1),
            Style::default().fg(lc(l1)).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" 1m  ", theme::dim_style()),
        Span::styled(format!("{:.2}", l5), Style::default().fg(lc(l5))),
        Span::styled(" 5m  ", theme::dim_style()),
        Span::styled(format!("{:.2}", l15), Style::default().fg(lc(l15))),
        Span::styled(" 15m", theme::dim_style()),
    ]));

    // ── Separator ─────────────────────────────────────────────────────────
    lines.push(Line::from(Span::styled(
        "─".repeat(inner_w),
        theme::border_style(),
    )));

    // ── Per-core rows (2 columns when wide enough) ────────────────────────
    let cores = &cpu.cores;
    let freqs = &cpu.freqs;
    let get_f = |i: usize| freqs.get(i).copied().unwrap_or(0);
    let two_col = inner_w >= 56;
    let col_w = if two_col { inner_w / 2 } else { inner_w };
    let mini_w = col_w.saturating_sub(24).clamp(6, 18);

    let core_line = |idx: usize, usage: f32, mhz: u64| -> Vec<Span<'static>> {
        let bar = mini_bar(usage, mini_w);
        let color = theme::pct_color_f32(usage);
        vec![
            Span::styled(format!("C{:<2} ", idx), theme::dim_style()),
            Span::styled(bar, Style::default().fg(color)),
            Span::styled(
                format!(" {:5.1}%", usage),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" {:>7}", freq_str(mhz)), theme::dim_style()),
        ]
    };

    for i in (0..cores.len()).step_by(if two_col { 2 } else { 1 }) {
        if two_col && i + 1 < cores.len() {
            let mut spans = core_line(i, cores[i], get_f(i));
            // pad left column to col_w
            let used: usize = spans.iter().map(|s| s.content.chars().count()).sum();
            if col_w > used {
                spans.push(Span::raw(" ".repeat(col_w - used)));
            }
            spans.push(Span::styled("│", theme::border_style()));
            spans.extend(core_line(i + 1, cores[i + 1], get_f(i + 1)));
            lines.push(Line::from(spans));
        } else {
            lines.push(Line::from(core_line(i, cores[i], get_f(i))));
        }
    }

    // ── Braille history graph ─────────────────────────────────────────────
    let core_rows = (cores.len() + if two_col { 1 } else { 0 }) / if two_col { 2 } else { 1 };
    // header=2, sep=1, sep_before_graph=1 → 4 fixed rows; fill all remaining height
    let graph_h = inner_h.saturating_sub(4 + core_rows).max(2);
    // Y-axis: "XX% " = 4 chars + "│" = 1 = 5 prefix chars; right side gets "│" too
    let y_w = 4usize; // label width "100%"
    let graph_w = inner_w.saturating_sub(y_w + 2).max(4); // +2 for left│ and right│

    lines.push(Line::from(Span::styled(
        "─".repeat(inner_w),
        theme::border_style(),
    )));

    if cpu.history.is_empty() {
        lines.push(Line::from(Span::styled(
            "  Collecting…",
            theme::dim_style(),
        )));
    } else {
        // render_f32 works directly on the f32 history — no Vec<u64> conversion needed.
        let braille_rows = braille::render_f32(&cpu.history, graph_w, graph_h);
        let graph_color = theme::pct_color_f32(g);

        let window_start = cpu.history.len().saturating_sub(graph_w * 2);
        let peak_pct = cpu.history[window_start..]
            .iter()
            .cloned()
            .fold(0.0_f32, f32::max)
            .min(100.0) as u64;
        let half_pct = peak_pct / 2;

        for (row_idx, row_str) in braille_rows.iter().enumerate() {
            let lbl = if row_idx == 0 {
                format!("{:>3}%", peak_pct)
            } else if row_idx == graph_h / 2 {
                format!("{:>3}%", half_pct)
            } else if row_idx + 1 == graph_h {
                "  0%".to_string()
            } else {
                "    ".to_string()
            };
            lines.push(Line::from(vec![
                Span::styled(format!("{}│", lbl), theme::dim_style()),
                Span::styled(row_str.clone(), Style::default().fg(graph_color)),
                Span::styled("│", theme::dim_style()),
            ]));
        }
        // Bottom axis baseline
        lines.push(Line::from(Span::styled(
            format!("    └{}┘", "─".repeat(graph_w)),
            theme::dim_style(),
        )));
    }

    let block = Block::default()
        .title(Span::styled(" CPU ", theme::title_style()))
        .borders(Borders::ALL)
        .border_style(theme::border_style());

    f.render_widget(Paragraph::new(lines).block(block), area);
}
