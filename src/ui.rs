use crate::app::{App, DetailContentMode, DetailPaneFocus, ViewMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState, Wrap,
    },
    Frame,
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
            let word_len = if word.is_ascii() {
                word.len()
            } else {
                word.chars().count()
            };

            if line_width + word_len > max_width {
                if line_width > 0 {
                    total_lines += 1;
                }

                if word_len > max_width {
                    total_lines += word_len / max_width;
                    line_width = word_len % max_width;
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
    let selected_style = Style::default()
        .bg(Color::DarkGray)
        .add_modifier(Modifier::BOLD);
    let normal_style = Style::default().bg(Color::Blue);
    let header_cells = ["Unit", "Schedule", "Last Run", "Next Run", "Status"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(Color::White)
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
            "Waiting" => Cell::from("⏳ Waiting").style(Style::default().fg(Color::Cyan)),
            "Inactive" => Cell::from("⏸ Inactive").style(Style::default().fg(Color::White)),
            _ => Cell::from(format!("⚠ {}", item.status)).style(Style::default().fg(Color::Red)),
        };

        let cells = vec![
            Cell::from(item.unit.as_str()).style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(item.schedule.as_str()).style(Style::default().fg(Color::Yellow)),
            Cell::from(item.last_rel.as_str()).style(Style::default().fg(Color::Magenta)),
            Cell::from(item.next_rel.as_str()).style(Style::default().fg(Color::Cyan)),
            status_cell,
        ];
        Row::new(cells).height(1)
    });

    if app.timers.is_empty() {
        let empty_msg = "\n\nNo user systemd timers found.\n\nCreate a timer in ~/.config/systemd/user/ to see it here.\n\nPress [r] to refresh or [q] to quit.";
        let empty_para = Paragraph::new(empty_msg)
            .style(Style::default().fg(Color::Gray))
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta))
                    .title(Line::from(vec![Span::styled(" Systemd Timers ", Style::default().fg(Color::White))])),
            );
        f.render_widget(empty_para, area);
        return;
    }

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
            .title(Line::from(vec![Span::styled(" Systemd Timers ", Style::default().fg(Color::White))])),
    )
    .highlight_style(selected_style)
    .highlight_symbol("▶  ");

    let mut state = TableState::default();
    state.select(Some(app.selected_index));

    f.render_stateful_widget(t, area, &mut state);

    let mut scrollbar_state =
        ScrollbarState::new(app.timers.len().saturating_sub(1)).position(app.selected_index);
    f.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        area.inner(&ratatui::layout::Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
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
    let status_text = app.detail_status_text.as_str();

    let logs_label = if matches!(app.detail_content_mode, DetailContentMode::Logs) {
        "▶ Logs"
    } else {
        "  Logs"
    };
    let service_file_label = if matches!(app.detail_content_mode, DetailContentMode::ServiceFile) {
        "▶ Service File"
    } else {
        "  Service File"
    };

    let top_prefix = if top_active {
        " ▶ Status: "
    } else {
        "   Status: "
    };
    let status_block = Block::default()
        .borders(Borders::ALL)
        .border_style(if top_active {
            active_border
        } else {
            inactive_border
        })
        .title(Line::from(vec![
            Span::styled(top_prefix, Style::default().fg(Color::White)),
            Span::styled(timer, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" ", Style::default().fg(Color::White)),
        ]))
        .title(Line::from(vec![
            Span::styled(controls_prefix, Style::default().fg(Color::White)),
            Span::styled(logs_label, logs_style),
            Span::styled(" | ", Style::default().fg(Color::White)),
            Span::styled(service_file_label, service_file_style),
            Span::styled(controls_suffix, Style::default().fg(Color::White)),
        ]));
    let status_para = Paragraph::new(status_text)
        .block(status_block)
        .wrap(Wrap { trim: true });
    f.render_widget(status_para, chunks[0]);

    let bottom_prefix = if bottom_active {
        " ▶ Bottom Pane: "
    } else {
        "   Bottom Pane: "
    };
    let bottom_title = match app.detail_content_mode {
        DetailContentMode::Logs => {
            if app.auto_scroll {
                Line::from(vec![
                    Span::styled(format!("{}Logs ", bottom_prefix), Style::default().fg(Color::White)),
                    Span::styled("[Auto-scroll: On] ", Style::default().fg(Color::Green)),
                ])
            } else {
                Line::from(vec![
                    Span::styled(format!("{}Logs ", bottom_prefix), Style::default().fg(Color::White)),
                    Span::styled("[Auto-scroll: Off] ", Style::default().fg(Color::Gray)),
                ])
            }
        }
        DetailContentMode::ServiceFile => Line::from(vec![
            Span::styled(format!("{}Service File ", bottom_prefix), Style::default().fg(Color::White)),
        ]),
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

    let is_empty_logs =
        app.detail_logs.trim().is_empty() || app.detail_logs.trim() == "-- No entries --";

    if is_empty_logs {
        let empty_msg = match app.detail_content_mode {
            DetailContentMode::Logs => {
                "\n\nNo logs found for this service. It may not have run yet.\nPress [Space] to start the timer and generate logs."
            }
            DetailContentMode::ServiceFile => "\n\nService file is empty or unavailable.\nPress [Esc] to return.",
        };
        let empty_para = Paragraph::new(empty_msg)
            .style(Style::default().fg(Color::Gray))
            .alignment(ratatui::layout::Alignment::Center)
            .block(logs_block);
        f.render_widget(empty_para, chunks[1]);
    } else {
        let logs_list = Paragraph::new(app.detail_logs.as_str())
            .block(logs_block)
            .wrap(Wrap { trim: false })
            .scroll((app.detail_scroll as u16, 0));

        f.render_widget(logs_list, chunks[1]);
    }

    if !is_empty_logs && app.detail_max_scroll > 0 {
        let mut scrollbar_state =
            ScrollbarState::new(app.detail_max_scroll).position(app.detail_scroll);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            chunks[1].inner(&ratatui::layout::Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
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
        .border_style(Style::default().fg(Color::Red))
        .title(Line::from(vec![Span::styled(" Error (Press any key to dismiss) ", Style::default().fg(Color::White))]));
    let para = Paragraph::new(msg)
        .style(Style::default().fg(Color::White))
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(Clear, popup_area);
    f.render_widget(para, popup_area);
}

fn draw_footer(f: &mut Frame, app: &mut App, area: Rect) {
    let toggle_desc = if let Some(timer) = app.selected_timer() {
        if timer.status == "Active" || timer.status == "Waiting" {
            "Stop Timer"
        } else {
            "Start Timer"
        }
    } else {
        "Toggle Timer"
    };

    let bindings = match app.mode {
        ViewMode::List => {
            if app.timers.is_empty() {
                vec![("q", "Quit"), ("r", "Refresh")]
            } else {
                vec![
                    ("q", "Quit"),
                    ("r", "Refresh"),
                    ("\u{2191}\u{2193}/j/k", "Navigate"),
                    ("Enter", "Details"),
                    ("Space", toggle_desc),
                ]
            }
        }
        ViewMode::Detail => match app.detail_focus {
            DetailPaneFocus::Top => vec![
                ("q", "Quit"),
                ("r", "Refresh"),
                ("Esc", "Back"),
                ("Tab", "Focus Bottom"),
                ("\u{2190}\u{2192}", "Switch Mode"),
                ("Space", toggle_desc),
            ],
            DetailPaneFocus::Bottom => {
                let mut binds = vec![("q", "Quit"), ("r", "Refresh"), ("Esc", "Back"), ("Tab", "Focus Top")];
                if app.detail_max_scroll > 0 {
                    binds.push(("\u{2191}\u{2193}/j/k", "Scroll"));
                }
                binds.push(("Space", toggle_desc));
                binds
            }
        },
    };

    let mut spans = Vec::new();
    let key_style = Style::default()
        .fg(Color::Black)
        .bg(Color::Gray)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(Color::White).bg(Color::DarkGray);

    for (i, (key, desc)) in bindings.iter().enumerate() {
        spans.push(Span::styled(format!(" {} ", key), key_style));
        spans.push(Span::styled(format!(" {} ", desc), desc_style));
        if i < bindings.len() - 1 {
            spans.push(Span::raw("  "));
        }
    }

    let footer = Paragraph::new(Line::from(spans)).alignment(ratatui::layout::Alignment::Center);

    f.render_widget(footer, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systemd::TimerInfo;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_draw_list_empty() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new();

        terminal
            .draw(|f| {
                draw_list(f, &mut app, f.size());
            })
            .unwrap();
    }

    #[test]
    fn test_draw_error() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let error_msg = "Critical failure";

        terminal
            .draw(|f| {
                draw_error(f, error_msg, f.size());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer.get(x, y).symbol())
                    .collect::<String>()
            })
            .collect::<Vec<String>>()
            .join("\n");

        assert!(content.contains("Error (Press any key to dismiss)"));
        assert!(content.contains(error_msg));
    }

    #[test]
    fn test_draw_list_with_items() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new();
        app.timers.push(TimerInfo {
            unit: "test.timer".into(),
            activates: "test.service".into(),
            next_abs: "n/a".into(),
            last_abs: "n/a".into(),
            next_rel: "n/a".into(),
            last_rel: "n/a".into(),
            status: "Active".into(),
            schedule: "daily".into(),
        });

        terminal
            .draw(|f| {
                draw_list(f, &mut app, f.size());
            })
            .unwrap();
    }

    #[test]
    fn test_draw_detail_logs_mode() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new();
        app.timers.push(TimerInfo {
            unit: "test.timer".into(),
            activates: "test.service".into(),
            next_abs: "tomorrow".into(),
            last_abs: "yesterday".into(),
            next_rel: "1d".into(),
            last_rel: "1d".into(),
            status: "Active".into(),
            schedule: "daily".into(),
        });
        app.enter_detail();
        app.detail_content_mode = DetailContentMode::Logs;
        app.detail_logs = "Sample log output\nLine 2".into();

        terminal
            .draw(|f| {
                draw_detail(f, &mut app, f.size());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer.get(x, y).symbol())
                    .collect::<String>()
            })
            .collect::<Vec<String>>()
            .join("\n");

        assert!(content.contains("test.timer"));
        assert!(content.contains("Bottom Pane: Logs"));
        assert!(content.contains("Sample log output"));
    }

    #[test]
    fn test_draw_detail_service_file_mode() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new();
        app.timers.push(TimerInfo {
            unit: "test.timer".into(),
            activates: "test.service".into(),
            next_abs: "n/a".into(),
            last_abs: "n/a".into(),
            next_rel: "n/a".into(),
            last_rel: "n/a".into(),
            status: "Active".into(),
            schedule: "daily".into(),
        });
        app.enter_detail();
        app.detail_content_mode = DetailContentMode::ServiceFile;
        app.detail_logs = "[Unit]\nDescription=Test".into();

        terminal
            .draw(|f| {
                draw_detail(f, &mut app, f.size());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer.get(x, y).symbol())
                    .collect::<String>()
            })
            .collect::<Vec<String>>()
            .join("\n");

        assert!(content.contains("Bottom Pane: Service File"));
        assert!(content.contains("Description=Test"));
    }

    #[test]
    fn test_draw_detail_empty_timer() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new();

        app.enter_detail();

        terminal
            .draw(|f| {
                draw_detail(f, &mut app, f.size());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer.get(x, y).symbol())
                    .collect::<String>()
            })
            .collect::<Vec<String>>()
            .join("\n");

        assert!(content.contains("Unknown"));
    }

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

    #[test]
    fn test_draw_ui_list_mode() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new();

        terminal
            .draw(|f| {
                draw_ui(f, &mut app);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer.get(x, y).symbol())
                    .collect::<String>()
            })
            .collect::<Vec<String>>()
            .join("\n");

        assert!(content.contains("Systemd Timers"));
    }

    #[test]
    fn test_draw_ui_detail_mode() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new();
        app.enter_detail();

        terminal
            .draw(|f| {
                draw_ui(f, &mut app);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer.get(x, y).symbol())
                    .collect::<String>()
            })
            .collect::<Vec<String>>()
            .join("\n");

        assert!(content.contains("Unknown"));
    }

    #[test]
    fn test_draw_ui_with_error() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new();
        app.error = Some("Test Error Message".into());

        terminal
            .draw(|f| {
                draw_ui(f, &mut app);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer.get(x, y).symbol())
                    .collect::<String>()
            })
            .collect::<Vec<String>>()
            .join("\n");

        assert!(content.contains("Error (Press any key to dismiss)"));
        assert!(content.contains("Test Error Message"));
    }

    #[test]
    fn test_draw_footer_list_mode() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new();
        app.mode = ViewMode::List;
        app.timers.push(TimerInfo {
            unit: "test.timer".into(),
            activates: "test.service".into(),
            next_abs: "n/a".into(),
            last_abs: "n/a".into(),
            next_rel: "n/a".into(),
            last_rel: "n/a".into(),
            status: "Active".into(),
            schedule: "daily".into(),
        });

        terminal
            .draw(|f| {
                draw_footer(f, &mut app, f.size());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer.get(x, y).symbol())
                    .collect::<String>()
            })
            .collect::<Vec<String>>()
            .join("\n");

        assert!(content.contains("Quit"));
        assert!(content.contains("Refresh"));
        assert!(content.contains("Navigate"));
        assert!(content.contains("Details"));
        assert!(content.contains("Stop"));
    }

    #[test]
    fn test_draw_footer_list_mode_empty() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new();
        app.mode = ViewMode::List;
        // app.timers is empty

        terminal
            .draw(|f| {
                draw_footer(f, &mut app, f.size());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer.get(x, y).symbol())
                    .collect::<String>()
            })
            .collect::<Vec<String>>()
            .join("\n");

        assert!(content.contains("Quit"));
        assert!(content.contains("Refresh"));
        assert!(!content.contains("Navigate"));
        assert!(!content.contains("Details"));
        assert!(!content.contains("Toggle Timer"));
        assert!(!content.contains("Stop Timer"));
    }

    #[test]
    fn test_draw_footer_detail_top_focus() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new();
        app.mode = ViewMode::Detail;
        app.detail_focus = DetailPaneFocus::Top;

        terminal
            .draw(|f| {
                draw_footer(f, &mut app, f.size());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer.get(x, y).symbol())
                    .collect::<String>()
            })
            .collect::<Vec<String>>()
            .join("\n");

        assert!(content.contains("Quit"));
        assert!(content.contains("Back"));
        assert!(content.contains("Focus Bottom"));
        assert!(content.contains("Switch Mode"));
    }

    #[test]
    fn test_draw_footer_detail_bottom_focus_with_scroll() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new();
        app.mode = ViewMode::Detail;
        app.detail_focus = DetailPaneFocus::Bottom;
        app.detail_max_scroll = 5;

        terminal
            .draw(|f| {
                draw_footer(f, &mut app, f.size());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer.get(x, y).symbol())
                    .collect::<String>()
            })
            .collect::<Vec<String>>()
            .join("\n");

        assert!(content.contains("Quit"));
        assert!(content.contains("Back"));
        assert!(content.contains("Focus Top"));
        assert!(content.contains("Scroll"));
    }

    #[test]
    fn test_draw_footer_detail_bottom_focus_without_scroll() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new();
        app.mode = ViewMode::Detail;
        app.detail_focus = DetailPaneFocus::Bottom;
        app.detail_max_scroll = 0;

        terminal
            .draw(|f| {
                draw_footer(f, &mut app, f.size());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer.get(x, y).symbol())
                    .collect::<String>()
            })
            .collect::<Vec<String>>()
            .join("\n");

        assert!(content.contains("Quit"));
        assert!(content.contains("Back"));
        assert!(content.contains("Focus Top"));
        assert!(!content.contains("Scroll"));
    }

    #[test]
    fn test_centered_rect() {
        let parent = Rect::new(0, 0, 100, 100);

        // 50% width and height
        let child = centered_rect(50, 50, parent);
        assert_eq!(child, Rect::new(25, 25, 50, 50));

        // 80% width, 20% height
        let child2 = centered_rect(80, 20, parent);
        assert_eq!(child2, Rect::new(10, 40, 80, 20));

        let parent2 = Rect::new(0, 0, 200, 100);

        // 50% width and height on wider parent
        let child3 = centered_rect(50, 50, parent2);
        assert_eq!(child3, Rect::new(50, 25, 100, 50));
    }
}
