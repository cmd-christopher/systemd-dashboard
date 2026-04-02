use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Wrap},
    Frame,
};
use crate::app::{App, DetailContentMode, DetailPaneFocus, ViewMode};

const DETAIL_CONTROLS_TITLE: &str = "Detail Controls [Logs | Service File]";

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

    let timer = app
        .selected_timer()
        .map(|t| t.unit.as_str())
        .unwrap_or("Unknown");
    let top_active = matches!(app.detail_focus, DetailPaneFocus::Top);
    let bottom_active = matches!(app.detail_focus, DetailPaneFocus::Bottom);
    let active_border = Style::default().fg(Color::Yellow);
    let inactive_border = Style::default().fg(Color::DarkGray);
    let logs_style = if matches!(app.detail_content_mode, DetailContentMode::Logs) {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    let service_file_style = if matches!(app.detail_content_mode, DetailContentMode::ServiceFile) {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    let (controls_prefix, controls_suffix) = DETAIL_CONTROLS_TITLE
        .split_once("Logs | Service File")
        .unwrap_or(("Detail Controls [", "]"));
    let status_text = if let Some(t) = app.selected_timer() {
        format!(
            "Unit: {}\nService: {}\nSchedule: {}\n\nLast Run: {} ({})\nNext Run: {} ({})\nStatus: {}\n\n{}",
            t.unit,
            t.activates,
            t.schedule,
            t.last_abs,
            t.last_rel,
            t.next_abs,
            t.next_rel,
            t.status,
            app.detail_status
        )
    } else {
        app.detail_status.clone()
    };

    let status_block = Block::default()
        .borders(Borders::ALL)
        .border_style(if top_active {
            active_border
        } else {
            inactive_border
        })
        .title(Line::from(vec![
            Span::raw(" Status: "),
            Span::styled(timer, Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" "),
        ]))
        .title(Line::from(vec![
            Span::raw(controls_prefix),
            Span::styled("Logs", logs_style),
            Span::raw(" | "),
            Span::styled("Service File", service_file_style),
            Span::raw(controls_suffix),
        ]));
    let status_para = Paragraph::new(status_text)
        .block(status_block)
        .wrap(Wrap { trim: true });
    f.render_widget(status_para, chunks[0]);

    let bottom_title = match app.detail_content_mode {
        DetailContentMode::Logs => "Bottom Pane: Logs",
        DetailContentMode::ServiceFile => "Bottom Pane: Service File",
    };
    let logs_block = Block::default()
        .borders(Borders::ALL)
        .border_style(if bottom_active {
            active_border
        } else {
            inactive_border
        })
        .title(bottom_title);

    if app.auto_scroll && matches!(app.detail_content_mode, DetailContentMode::Logs) {
        let inner_height = chunks[1].height.saturating_sub(2) as usize;
        let line_count = app.detail_logs.lines().count();
        app.detail_scroll = line_count.saturating_sub(inner_height);
    }

    let logs_list = Paragraph::new(app.detail_logs.as_str())
        .block(logs_block)
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll as u16, 0));

    f.render_widget(logs_list, chunks[1]);
}

fn draw_error(f: &mut Frame, msg: &str, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Red))
        .title(" Error ");
    let para = Paragraph::new(msg).block(block);
    f.render_widget(para, area);
}
