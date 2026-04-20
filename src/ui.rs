use crate::app::{App, DetailContentMode, DetailPaneFocus, ViewMode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState, Wrap},
};

const DETAIL_CONTROLS_TITLE: &str = "Detail Controls [Logs | Service File]";

fn count_visual_lines(text: &str, max_width: u16) -> usize {
    let max_width = max_width as usize;
    if max_width == 0 {
        return 0;
    }
    let mut total_lines = 0;

    for line in text.lines() {
        if line.is_empty() {
            total_lines += 1;
            continue;
        }

        let mut line_width = 0;
        let words = line.split_inclusive(' ');

        for word in words {
            let word_len = word.chars().count();

            if line_width + word_len > max_width {
                if line_width > 0 {
                    total_lines += 1;
                    line_width = 0;
                }

                if word_len > max_width {
                    let full_lines = word_len / max_width;
                    let remainder = word_len % max_width;
                    if remainder == 0 {
                        total_lines += full_lines;
                        // Avoid unused assignment warning; line_width is naturally 0 here,
                        // but setting it triggers unused_assignment if it's the last token of the line
                    } else {
                        total_lines += full_lines;
                        line_width = remainder;
                    }
                } else {
                    line_width = word_len;
                }
            } else {
                line_width += word_len;
            }
        }

        if line_width > 0 {
            total_lines += 1;
        }
    }

    total_lines
}

pub fn draw_ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.size());

    match app.mode {
        ViewMode::List => draw_list(f, app, chunks[0]),
        ViewMode::Detail => draw_detail(f, app, chunks[0]),
    }

    draw_footer(f, app, chunks[1]);

    if let Some(err) = &app.error {
        draw_error(f, err, chunks[0]);
    }
}

fn draw_list(f: &mut Frame, app: &mut App, area: Rect) {
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Blue);
    let header_cells = ["Unit", "Schedule", "Last Run", "Next Run", "Status"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
        });
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);

    let rows = app.timers.iter().map(|item| {
        let status_cell = match item.status.as_str() {
            "Active" => Cell::from("✔ Active").style(Style::default().fg(Color::Green)),
            "Waiting" => Cell::from("⏳ Waiting").style(Style::default().fg(Color::DarkGray)),
            "Inactive" => Cell::from("⏸ Inactive").style(Style::default().fg(Color::Gray)),
            _ => Cell::from(item.status.as_str()).style(Style::default().fg(Color::Red)),
        };

        let cells = vec![
            Cell::from(item.unit.as_str()).style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(item.schedule.as_str()).style(Style::default().fg(Color::Yellow)),
            Cell::from(item.last_rel.as_str()).style(Style::default().fg(Color::DarkGray)),
            Cell::from(item.next_rel.as_str()).style(Style::default().fg(Color::Cyan)),
            status_cell,
        ];
        Row::new(cells).height(1)
    });

    let t = Table::new(
        rows,
        [
            Constraint::Percentage(25), // Unit
            Constraint::Percentage(25), // Schedule
            Constraint::Percentage(15), // Last Run
            Constraint::Percentage(15), // Next Run
            Constraint::Percentage(20), // Status
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta))
            .title(" Systemd Timers "),
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
        app.detail_status.as_str().into()
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

    let inner_width = chunks[1].width.saturating_sub(2);
    let inner_height = chunks[1].height.saturating_sub(2) as usize;
    let visual_lines = count_visual_lines(&app.detail_logs, inner_width);

    app.detail_max_scroll = visual_lines.saturating_sub(inner_height);

    if app.detail_scroll > app.detail_max_scroll {
        app.detail_scroll = app.detail_max_scroll;
    }

    if app.auto_scroll && matches!(app.detail_content_mode, DetailContentMode::Logs) {
        app.detail_scroll = app.detail_max_scroll;
    }

    let logs_list = Paragraph::new(app.detail_logs.as_str())
        .block(logs_block)
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll as u16, 0));

    f.render_widget(logs_list, chunks[1]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn draw_error(f: &mut Frame, msg: &str, area: Rect) {
    let popup_area = centered_rect(60, 20, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Red))
        .title(" Error (Press any key to dismiss) ");
    let para = Paragraph::new(msg).block(block).wrap(Wrap { trim: true });

    f.render_widget(Clear, popup_area);
    f.render_widget(para, popup_area);
}

fn draw_footer(f: &mut Frame, app: &mut App, area: Rect) {
    let keybindings: std::borrow::Cow<'static, str> = match app.mode {
        ViewMode::List => {
            let space_action = if let Some(timer) = app.selected_timer() {
                if timer.status == "Active" || timer.status == "Waiting" {
                    "Stop"
                } else {
                    "Start"
                }
            } else {
                "Toggle"
            };
            format!("q: Quit | j/k or \u{2191}/\u{2193}: Navigate | Enter: Detail | Space: {}", space_action).into()
        }
        ViewMode::Detail => match app.detail_focus {
            DetailPaneFocus::Top => {
                "q: Quit | Esc/Backspace: Back | Tab: Focus Bottom | Arrows: Switch Mode".into()
            }
            DetailPaneFocus::Bottom => {
                "q: Quit | Esc/Backspace: Back | Tab: Focus Top | Arrows or j/k: Scroll".into()
            }
        },
    };

    let footer = Paragraph::new(keybindings)
        .style(Style::default().fg(Color::White).bg(Color::DarkGray))
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(footer, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_visual_lines_zero_width() {
        assert_eq!(count_visual_lines("hello world", 0), 0);
    }

    #[test]
    fn test_count_visual_lines_empty_string() {
        assert_eq!(count_visual_lines("", 10), 0);
    }

    #[test]
    fn test_count_visual_lines_no_wrap() {
        assert_eq!(count_visual_lines("hello", 10), 1);
        assert_eq!(count_visual_lines("hello world", 11), 1);
    }

    #[test]
    fn test_count_visual_lines_with_wrap() {
        // "hello world" fits on line 1: "hello " (len 6) + "world" (len 5) = 11.
        // If max_width is 10, "hello " (6) + "world" (5) = 11 > 10.
        // Line 1: "hello " (6) -> total_lines=1, line_width=5 ("world")
        // total_lines=2
        assert_eq!(count_visual_lines("hello world", 10), 2);

        // "hello world" max_width=5
        // "hello " (len 6) > 5. word_len=6.
        // full_lines=1, remainder=1. total_lines=1, line_width=1.
        // "world" (len 5) = 5.
        // line_width(1) + 5 = 6 > 5.
        // total_lines=2, line_width=0.
        // "world" (5) <= 5. line_width=5.
        // end loop: total_lines=3.
        assert_eq!(count_visual_lines("hello world", 5), 3);
    }

    #[test]
    fn test_count_visual_lines_long_words() {
        // "supercalifragilisticexpialidocious" len=34
        // max_width=10 -> full_lines=3, remainder=4.
        // loop ends, line_width=4 -> +1 -> 4
        assert_eq!(
            count_visual_lines("supercalifragilisticexpialidocious", 10),
            4
        );

        // "12345678901234567890" len=20
        // max_width=10 -> full_lines=2, remainder=0.
        // total_lines=2, line_width=0. loop ends. total=2.
        assert_eq!(count_visual_lines("12345678901234567890", 10), 2);
    }

    #[test]
    fn test_count_visual_lines_multiline() {
        // "line1\nline2\n\nline4"
        // line1: len=5 -> total=1
        // line2: len=5 -> total=2
        // empty: total=3
        // line4: len=5 -> total=4
        assert_eq!(count_visual_lines("line1\nline2\n\nline4", 10), 4);
    }
}
