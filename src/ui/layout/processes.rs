use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::app::AppState;
use crate::ui::theme;

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let state = app.state.read();
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

    let header = Line::from(vec![
        Span::styled(format!("{:>7}  {:<20} {:<10} {:>7}  {:>7}  {}", "PID", "NAME", "USER", "CPU%", "MEM%", "ST"),
            theme::header_style()),
    ]);

    let visible_rows = (area.height as usize).saturating_sub(4);
    let scroll_off   = app.scroll_offset;

    let mut lines = vec![header];

    for (idx, proc) in procs.iter().skip(scroll_off).take(visible_rows).enumerate() {
        let is_selected = idx + scroll_off == app.selected_proc;
        let row_style = if is_selected {
            theme::highlight_style()
        } else {
            Style::default()
        };
        let cpu_color = theme::pct_color_f32(proc.cpu_pct);
        let mem_color = theme::pct_color_f32(proc.mem_pct);

        lines.push(Line::from(vec![
            Span::styled(
                format!("{:>7}  {:<20} {:<10} ", proc.pid, crate::ui::truncate(&proc.name, 18), crate::ui::truncate(&proc.user, 8)),
                row_style,
            ),
            Span::styled(format!("{:>7.2}  ", proc.cpu_pct), Style::default().fg(cpu_color)),
            Span::styled(format!("{:>7.2}  ", proc.mem_pct), Style::default().fg(mem_color)),
            Span::styled(proc.status.clone(), row_style),
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
