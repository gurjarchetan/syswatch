use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::app::AppState;
use crate::collector::network::NetStats;
use crate::ui::{braille, theme};

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let state = app.state.read();
    let net = &state.network;

    let inner_w = (area.width as usize).saturating_sub(2).max(1);
    let graph_h = (area.height as usize).saturating_sub(6).max(1);

    let rx_spark = braille::sparkline_u64(&net.rx_history, inner_w / 2);
    let tx_spark = braille::sparkline_u64(&net.tx_history, inner_w / 2);

    let mut lines = vec![
        Line::from(vec![
            Span::styled("▼ Download  ", theme::header_style()),
            Span::raw(NetStats::fmt_speed(net.rx_bps)),
            Span::raw(format!("  Total: {}", NetStats::fmt_bytes(net.total_rx))),
        ]),
        Line::from(vec![
            Span::styled("▲ Upload    ", theme::header_style()),
            Span::raw(NetStats::fmt_speed(net.tx_bps)),
            Span::raw(format!("  Total: {}", NetStats::fmt_bytes(net.total_tx))),
        ]),
        Line::from(vec![Span::raw("")]),
    ];

    // Sparklines side by side
    lines.push(Line::from(vec![
        Span::styled(format!("{:<width$}", "RX History", width = inner_w / 2), theme::dim_style()),
        Span::styled(format!("{:<width$}", "TX History", width = inner_w / 2), theme::dim_style()),
    ]));
    lines.push(Line::from(vec![
        Span::styled(rx_spark, Style::default().fg(ratatui::style::Color::Cyan)),
        Span::styled(tx_spark, Style::default().fg(ratatui::style::Color::Magenta)),
    ]));

    // Interface list
    lines.push(Line::from(vec![Span::raw("")]));
    for iface in &net.interfaces {
        lines.push(Line::from(vec![
            Span::raw(format!("  {:<12}", iface.name)),
            Span::styled(format!("RX {:>10}  ", NetStats::fmt_speed(iface.rx_bps)), Style::default().fg(ratatui::style::Color::Cyan)),
            Span::styled(format!("TX {:>10}", NetStats::fmt_speed(iface.tx_bps)), Style::default().fg(ratatui::style::Color::Magenta)),
        ]));
    }

    let block = Block::default()
        .title(Span::styled(" Network Deep Dive ", theme::title_style()))
        .borders(Borders::ALL);

    f.render_widget(Paragraph::new(lines).block(block), area);
}
