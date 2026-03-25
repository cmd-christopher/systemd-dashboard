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
    if let Ok(timers) = fetch_timers().await {
        app.timers = timers;
    }

    loop {
        terminal.draw(|f| ui::draw_ui(f, app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
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
                                
                                let _ = crate::systemd::toggle_timer(&unit, !is_active).await;
                                
                                if let Ok(timers) = fetch_timers().await {
                                    app.timers = timers;
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
                            let max_lines = detail_content_max_lines(app);
                            app.scroll_detail_down(max_lines);
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
            last_tick = Instant::now();
        }

        if last_refresh.elapsed() >= Duration::from_secs(60) {
            if let Ok(timers) = fetch_timers().await {
                app.timers = timers;
            }
            
            if let ViewMode::Detail = app.mode {
                if let Some(timer) = app.selected_timer() {
                    let unit = timer.unit.clone();
                    app.detail_status = fetch_timer_status(&unit).await;
                    let previous_scroll = app.detail_scroll;
                    refresh_detail_content(app, false).await;
                    
                    let new_max = app.detail_logs.lines().count().saturating_sub(1);
                    if previous_scroll >= new_max.saturating_sub(2) {
                        app.detail_scroll = new_max;
                    } else {
                        app.detail_scroll = previous_scroll.min(new_max);
                    }
                }
            }

            last_refresh = Instant::now();
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn detail_content_max_lines(app: &App) -> usize {
    app.detail_logs.lines().count().saturating_sub(1)
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
        }
    }
}
