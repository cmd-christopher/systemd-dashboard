use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};
use crate::app::{App, ViewMode};

pub fn draw_ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .split(f.size());

    match app.mode {
        ViewMode::List => draw_list(f, app, chunks[0]),
        ViewMode::Detail => draw_detail(f, app, chunks[0]),
    }

    if let Some(err) = &app.error {
        draw_error(f, err, chunks[0]);
    }
}

fn draw_list(f: &mut Frame, app: &mut App, area: Rect) {
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Blue);
    let header_cells = ["Unit", "Schedule", "Last Run", "Next Run", "Status"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Black).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);

    let rows = app.timers.iter().map(|item| {
        let status_cell = match item.status.as_str() {
            "Active" => Cell::from("✔ Active").style(Style::default().fg(Color::Green)),
            "Waiting" => Cell::from("⏳ Waiting").style(Style::default().fg(Color::DarkGray)),
            "Inactive" => Cell::from("⏸ Inactive").style(Style::default().fg(Color::Gray)),
            _ => Cell::from(item.status.clone()).style(Style::default().fg(Color::Red)),
        };

        let cells = vec![
            Cell::from(item.unit.clone()).style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Cell::from(item.schedule.clone()).style(Style::default().fg(Color::Yellow)),
            Cell::from(item.last_rel.clone()).style(Style::default().fg(Color::DarkGray)),
            Cell::from(item.next_rel.clone()).style(Style::default().fg(Color::Cyan)),
            status_cell,
        ];
        Row::new(cells).height(1)
    });

    let t = Table::new(rows, [
        Constraint::Percentage(25), // Unit
        Constraint::Percentage(25), // Schedule
        Constraint::Percentage(15), // Last Run
        Constraint::Percentage(15), // Next Run
        Constraint::Percentage(20), // Status
    ])
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta))
            .title(" Systemd Timers ")
    )
    .highlight_style(selected_style)
    .highlight_symbol(">> ");

    let mut state = TableState::default();
    state.select(Some(app.selected_index));

    f.render_stateful_widget(t, area, &mut state);
}

fn draw_detail(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    let timer = app.selected_timer().map(|t| t.unit.as_str()).unwrap_or("Unknown");
    
    let status_text = if let Some(t) = app.selected_timer() {
        format!(
            "Unit: {}\nService: {}\nSchedule: {}\n\nLast Run: {} ({})\nNext Run: {} ({})\nStatus: {}",
            t.unit, t.activates, t.schedule, t.last_abs, t.last_rel, t.next_abs, t.next_rel, t.status
        )
    } else {
        app.detail_status.clone()
    };

    let status_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(format!(" Status: {} ", timer));
    let status_para = Paragraph::new(status_text)
        .block(status_block)
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(status_para, chunks[0]);

    let logs_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .title(" Recent Logs ");
    let logs_para = Paragraph::new(app.detail_logs.as_str())
        .block(logs_block)
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(logs_para, chunks[1]);
}

fn draw_error(f: &mut Frame, msg: &str, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Red))
        .title(" Error ");
    let para = Paragraph::new(msg).block(block);
    f.render_widget(para, area);
}
