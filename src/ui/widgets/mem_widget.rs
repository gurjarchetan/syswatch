use crate::app::AppState;
use crate::collector::memory::MemStats;
use crate::ui::{braille, theme};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

fn gradient_bar(pct: f64, width: usize) -> Vec<Span<'static>> {
    let filled = ((pct / 100.0) * width as f64)
        .round()
        .clamp(0.0, width as f64) as usize;
    let mut spans = Vec::with_capacity(width + 2);
    spans.push(Span::styled("▕", theme::dim_style()));
    for i in 0..width {
        let pos_pct = (i as f64 / width as f64) * 100.0;
        let ch = if i < filled { "█" } else { "░" };
        let color = if i < filled {
            theme::pct_color(pos_pct.max(pct * 0.3))
        } else {
            theme::C_BORDER
        };
        spans.push(Span::styled(ch, Style::default().fg(color)));
    }
    spans.push(Span::styled("▏", theme::dim_style()));
    spans
}

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let state = app.state.read();
    let mem = &state.memory;
    let inner_w = (area.width as usize).saturating_sub(2).max(20);
    let inner_h = (area.height as usize).saturating_sub(2).max(1);
    let bar_w = inner_w.saturating_sub(22).clamp(10, 24);

    let used_pct = mem.used_pct();
    let swap_pct = mem.swap_pct();

    let mut lines: Vec<Line> = Vec::new();

    // ── RAM row ───────────────────────────────────────────────────────────
    let mut ram_spans = vec![Span::styled("RAM  ", theme::header_style())];
    ram_spans.extend(gradient_bar(used_pct, bar_w));
    ram_spans.push(Span::styled(
        format!(" {:5.1}%", used_pct),
        Style::default()
            .fg(theme::pct_color(used_pct))
            .add_modifier(Modifier::BOLD),
    ));
    lines.push(Line::from(ram_spans));

    // ── RAM detail ────────────────────────────────────────────────────────
    lines.push(Line::from(vec![
        Span::styled("     ", theme::dim_style()),
        Span::styled(
            MemStats::fmt_kb(mem.used_kb),
            Style::default().fg(theme::C_WHITE),
        ),
        Span::styled(" / ", theme::dim_style()),
        Span::styled(MemStats::fmt_kb(mem.total_kb), theme::dim_style()),
        Span::styled("  free ", theme::dim_style()),
        Span::styled(
            MemStats::fmt_kb(mem.free_kb),
            Style::default().fg(theme::C_GREEN),
        ),
    ]));

    // ── Swap row ──────────────────────────────────────────────────────────
    let mut swap_spans = vec![Span::styled("Swap ", theme::header_style())];
    swap_spans.extend(gradient_bar(swap_pct, bar_w));
    swap_spans.push(Span::styled(
        format!(" {:5.1}%", swap_pct),
        Style::default()
            .fg(theme::pct_color(swap_pct))
            .add_modifier(Modifier::BOLD),
    ));
    lines.push(Line::from(swap_spans));
    lines.push(Line::from(vec![
        Span::styled("     ", theme::dim_style()),
        Span::styled(
            MemStats::fmt_kb(mem.swap_used_kb),
            Style::default().fg(theme::C_WHITE),
        ),
        Span::styled(" / ", theme::dim_style()),
        Span::styled(MemStats::fmt_kb(mem.swap_total_kb), theme::dim_style()),
    ]));

    // ── Braille RAM history graph ─────────────────────────────────────────
    // 4 content rows (2×bar+detail), 1 sep, border=2 → remaining = graph
    let graph_h = inner_h.saturating_sub(5);
    if graph_h >= 2 && !state.mem_history.is_empty() {
        let used_pct2 = mem.used_pct();
        lines.push(Line::from(Span::styled(
            "─".repeat(inner_w),
            theme::border_style(),
        )));
        let y_w = 4usize;
        let graph_w = inner_w.saturating_sub(y_w + 2).max(4);
        let color = theme::pct_color(used_pct2);
        // Absolute scale: data = pct×100, max = 10000 (=100%)
        let rows = braille::render_absolute(&state.mem_history, 10_000, graph_w, graph_h);
        for (i, row_str) in rows.iter().enumerate() {
            let lbl = if i == 0 {
                "100%".to_string()
            } else if i == graph_h / 2 {
                " 50%".to_string()
            } else if i + 1 == graph_h {
                "  0%".to_string()
            } else {
                "    ".to_string()
            };
            lines.push(Line::from(vec![
                Span::styled(format!("{} │", lbl), theme::dim_style()),
                Span::styled(row_str.clone(), Style::default().fg(color)),
                Span::styled("│", theme::dim_style()),
            ]));
        }
        lines.push(Line::from(Span::styled(
            format!("    └{}┘", "─".repeat(graph_w)),
            theme::dim_style(),
        )));
    }

    let block = Block::default()
        .title(Span::styled(" Memory ", theme::title_style()))
        .borders(Borders::ALL)
        .border_style(theme::border_style());

    f.render_widget(Paragraph::new(lines).block(block), area);
}
