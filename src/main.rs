mod app;
mod systemd;
mod ui;

use crate::app::{App, DetailContentMode, DetailPaneFocus, ViewMode};
use crate::systemd::{
    fetch_service_file_content, fetch_timer_logs, fetch_timer_status, fetch_timers,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{error::Error, io, time::{Duration, Instant}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let mut last_refresh = Instant::now();
    let tick_rate = Duration::from_millis(250);
    
    // Initial fetch
    match fetch_timers().await {
        Ok(timers) => {
            app.timers = timers;
            app.error = None;
        }
        Err(e) => app.error = Some(e),
    }

    loop {
        terminal.draw(|f| ui::draw_ui(f, app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if app.error.is_some() {
                    app.error = None;
                }
                match app.mode {
                    ViewMode::List => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::Enter => {
                            if let Some(timer) = app.selected_timer() {
                                let unit = timer.unit.clone();
                                let activates = timer.activates.clone();
                                app.detail_status = fetch_timer_status(&unit).await;
                                app.detail_logs = fetch_timer_logs(&activates).await;
                                app.enter_detail();
                            }
                        }
                        KeyCode::Char(' ') => {
                            if let Some(timer) = app.selected_timer() {
                                let unit = timer.unit.clone();
                                let is_active = timer.status == "Active" || timer.status == "Waiting";

                                match crate::systemd::toggle_timer(&unit, !is_active).await {
                                    Ok(_) => {
                                        app.error = None;
                                        match fetch_timers().await {
                                            Ok(timers) => app.timers = timers,
                                            Err(e) => app.error = Some(e),
                                        }
                                    }
                                    Err(e) => app.error = Some(e),
                                }
                            }
                        }
                        _ => {}
                    },
                    ViewMode::Detail => match key.code {
                        KeyCode::Esc | KeyCode::Backspace => app.exit_detail(),
                        KeyCode::Tab => app.toggle_detail_focus(),
                        KeyCode::Left | KeyCode::Up
                            if matches!(app.detail_focus, DetailPaneFocus::Top) =>
                        {
                            app.select_previous_detail_content();
                            refresh_detail_content(app, true).await;
                        }
                        KeyCode::Right | KeyCode::Down
                            if matches!(app.detail_focus, DetailPaneFocus::Top) =>
                        {
                            app.select_next_detail_content();
                            refresh_detail_content(app, true).await;
                        }
                        KeyCode::Left | KeyCode::Up
                            if matches!(app.detail_focus, DetailPaneFocus::Bottom) =>
                        {
                            app.scroll_detail_up();
                        }
                        KeyCode::Right | KeyCode::Down | KeyCode::Char('j')
                            if matches!(app.detail_focus, DetailPaneFocus::Bottom) =>
                        {
                            app.scroll_detail_down();
                        }
                        KeyCode::Char('k')
                            if matches!(app.detail_focus, DetailPaneFocus::Bottom) =>
                        {
                            app.scroll_detail_up();
                        }
                        KeyCode::Char('q') => return Ok(()),
                        _ => {}
                    },
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            if let ViewMode::Detail = app.mode {
                // More frequent refresh for detail view to support "real-time" logs
                if last_refresh.elapsed() >= Duration::from_secs(2) {
                    if let Some(timer) = app.selected_timer() {
                        let unit = timer.unit.clone();
                        app.detail_status = fetch_timer_status(&unit).await;
                        refresh_detail_content(app, false).await;
                    }
                    last_refresh = Instant::now();
                }
            } else if last_refresh.elapsed() >= Duration::from_secs(60) {
                // Slower refresh for the main list
                match fetch_timers().await {
                    Ok(timers) => {
                        app.timers = timers;
                        app.error = None;
                    }
                    Err(e) => app.error = Some(e),
                }
                last_refresh = Instant::now();
            }

            last_tick = Instant::now();
        }

    }
}

async fn refresh_detail_content(app: &mut App, reset_scroll: bool) {
    if let Some(timer) = app.selected_timer() {
        let activates = timer.activates.clone();
        app.detail_logs = match app.detail_content_mode {
            DetailContentMode::Logs => fetch_timer_logs(&activates).await,
            DetailContentMode::ServiceFile => fetch_service_file_content(&activates).await,
        };
        if reset_scroll {
            app.detail_scroll = 0;
            // Auto-scroll only makes sense for logs
            app.auto_scroll = matches!(app.detail_content_mode, DetailContentMode::Logs);
        }
    }
}
