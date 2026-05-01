use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::app::AppState;
use crate::ui::{braille, theme};

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let state = app.state.read();
    let cpu = &state.cpu;

    let graph_width = (area.width as usize).saturating_sub(2).max(1);
    let graph_rows  = (area.height as usize).saturating_sub(3).max(1);
    let spark_data: Vec<u64> = cpu.history.iter().map(|&v| v as u64).collect();
    let graph_lines = braille::render(&spark_data, graph_width, graph_rows);

    let mut lines: Vec<Line> = Vec::new();

    // Header row: global usage + freq
    let global_pct = cpu.global;
    let bar_width = 20usize;
    let filled = ((global_pct / 100.0) * bar_width as f32) as usize;
    let bar_color = theme::pct_color_f32(global_pct);
    let bar: String = format!(
        "[{}{}] {:5.1}%",
        "█".repeat(filled),
        "░".repeat(bar_width - filled),
        global_pct
    );
    lines.push(Line::from(vec![
        Span::styled(format!("CPU({}) ", cpu.count), theme::header_style()),
        Span::styled(bar, Style::default().fg(bar_color)),
        Span::raw(format!("  {}", cpu.brand)),
    ]));

    // Per-core mini-bars (up to 2 per line to save space)
    let core_chunks: Vec<_> = cpu.cores.chunks(4).collect();
    for chunk in &core_chunks {
        let mut spans: Vec<Span> = Vec::new();
        for (i, &usage) in chunk.iter().enumerate() {
            let color = theme::pct_color_f32(usage);
            spans.push(Span::styled(
                format!(" {:5.1}% ", usage),
                Style::default().fg(color),
            ));
            if i < chunk.len() - 1 { spans.push(Span::raw("│")); }
        }
        lines.push(Line::from(spans));
    }

    // Braille graph
    for gl in &graph_lines {
        lines.push(Line::from(vec![
            Span::styled(gl.clone(), Style::default().fg(ratatui::style::Color::Green)),
        ]));
    }

    let block = Block::default()
        .title(Span::styled(" CPU ", theme::title_style()))
        .borders(Borders::ALL);

    f.render_widget(Paragraph::new(lines).block(block), area);
}
