use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};
use crate::app::AppState;
use crate::ui::widgets;

/// Overview: CPU (left) | Memory + quick net (right)
pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    let right_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(cols[1]);

    widgets::cpu_widget::render(f, cols[0], app);
    widgets::mem_widget::render(f, right_rows[0], app);
    render_net_summary(f, right_rows[1], app);
}

fn render_net_summary(f: &mut Frame, area: Rect, app: &AppState) {
    use ratatui::{
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph},
    };
    use crate::collector::network::NetStats;
    use crate::ui::theme;

    let state = app.state.read();
    let net = &state.network;

    let lines = vec![
        Line::from(vec![
            Span::styled("▲ TX ", theme::header_style()),
            Span::raw(NetStats::fmt_speed(net.tx_bps)),
            Span::raw(format!("  Total: {}", NetStats::fmt_bytes(net.total_tx))),
        ]),
        Line::from(vec![
            Span::styled("▼ RX ", theme::header_style()),
            Span::raw(NetStats::fmt_speed(net.rx_bps)),
            Span::raw(format!("  Total: {}", NetStats::fmt_bytes(net.total_rx))),
        ]),
    ];

    let block = Block::default()
        .title(Span::styled(" Network ", theme::title_style()))
        .borders(Borders::ALL);

    f.render_widget(Paragraph::new(lines).block(block), area);
}
