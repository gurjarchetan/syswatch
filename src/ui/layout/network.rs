use crate::app::AppState;
use crate::collector::network::NetStats;
use crate::ui::{braille, theme};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let inner_w = (area.width as usize).saturating_sub(2).max(4);
    // Clone the network snapshot under the lock, then release it before building
    // lines and computing braille — this frees the collector to write concurrently.
    let net = app.state.read().network.clone();
    // Fixed rows: 2 speed lines + sep + iface header + (n ifaces) + sep_before_graph + graph_label
    let iface_count = net.interfaces.len();
    let fixed_rows = 5 + iface_count;
    let graph_h = (area.height as usize)
        .saturating_sub(2 + fixed_rows) // border
        .clamp(2, 8);

    let half_w = inner_w / 2;
    let y_lbl_w = 5usize;
    let col_w = half_w.saturating_sub(y_lbl_w + 1).max(4);

    let mut lines: Vec<Line> = Vec::new();

    // ── Speed header ──────────────────────────────────────────────────────
    lines.push(Line::from(vec![
        Span::styled(
            "▼ RX ",
            Style::default()
                .fg(theme::C_TEAL)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            NetStats::fmt_speed(net.rx_bps),
            Style::default()
                .fg(theme::C_WHITE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  total {} ", NetStats::fmt_bytes(net.total_rx)),
            theme::dim_style(),
        ),
        Span::styled(
            "  ▲ TX ",
            Style::default()
                .fg(theme::C_MAGENTA)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            NetStats::fmt_speed(net.tx_bps),
            Style::default()
                .fg(theme::C_WHITE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  total {}", NetStats::fmt_bytes(net.total_tx)),
            theme::dim_style(),
        ),
    ]));

    // ── Separator ─────────────────────────────────────────────────────────
    lines.push(Line::from(Span::styled(
        "─".repeat(inner_w),
        theme::border_style(),
    )));

    // ── Interface table header ────────────────────────────────────────────
    lines.push(Line::from(vec![
        Span::styled(format!("  {:<14}", "Interface"), theme::header_style()),
        Span::styled(
            format!("{:>12}  ", "▼ RX/s"),
            Style::default()
                .fg(theme::C_TEAL)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:>12}", "▲ TX/s"),
            Style::default()
                .fg(theme::C_MAGENTA)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    for iface in &net.interfaces {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:<14}", crate::ui::truncate(&iface.name, 12)),
                theme::dim_style(),
            ),
            Span::styled(
                format!("{:>12}  ", NetStats::fmt_speed(iface.rx_bps)),
                Style::default().fg(theme::C_TEAL),
            ),
            Span::styled(
                format!("{:>12}", NetStats::fmt_speed(iface.tx_bps)),
                Style::default().fg(theme::C_MAGENTA),
            ),
        ]));
    }

    // ── Separator + graph labels ──────────────────────────────────────────
    lines.push(Line::from(Span::styled(
        "─".repeat(inner_w),
        theme::border_style(),
    )));
    lines.push(Line::from(vec![
        Span::styled(
            format!("{:>y_lbl_w$} │{:<col_w$}", "RX", " Download History"),
            Style::default().fg(theme::C_TEAL),
        ),
        Span::styled(" │", theme::border_style()),
        Span::styled(
            format!("{:>y_lbl_w$} │{:<col_w$}", "TX", " Upload History"),
            Style::default().fg(theme::C_MAGENTA),
        ),
    ]));

    // ── Braille graphs side-by-side ───────────────────────────────────────
    let rx_data: Vec<u64> = net.rx_history.iter().copied().collect();
    let tx_data: Vec<u64> = net.tx_history.iter().copied().collect();
    let rx_rows = braille::render(&rx_data, col_w / 2, graph_h);
    let tx_rows = braille::render(&tx_data, col_w / 2, graph_h);

    for row_idx in 0..graph_h {
        let pct_lbl = if row_idx == 0 {
            "max".to_string()
        } else if row_idx == graph_h / 2 {
            " 50%".to_string()
        } else if row_idx + 1 == graph_h {
            "  0%".to_string()
        } else {
            "    ".to_string()
        };
        let rx_str = rx_rows.get(row_idx).cloned().unwrap_or_default();
        let tx_str = tx_rows.get(row_idx).cloned().unwrap_or_default();
        lines.push(Line::from(vec![
            Span::styled(format!("{:>y_lbl_w$} │", pct_lbl), theme::dim_style()),
            Span::styled(rx_str, Style::default().fg(theme::C_TEAL)),
            Span::styled(" │", theme::border_style()),
            Span::styled(format!("{:>y_lbl_w$} │", pct_lbl), theme::dim_style()),
            Span::styled(tx_str, Style::default().fg(theme::C_MAGENTA)),
        ]));
    }

    let block = Block::default()
        .title(Span::styled(" Network ", theme::title_style()))
        .borders(Borders::ALL)
        .border_style(theme::border_style());

    f.render_widget(Paragraph::new(lines).block(block), area);
}
