mod app;
mod systemd;
mod ui;

use crate::app::{App, ViewMode};
use crate::systemd::{fetch_timer_logs, fetch_timer_status, fetch_timers};
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
                        _ => {}
                    },
                    ViewMode::Detail => match key.code {
                        KeyCode::Esc | KeyCode::Left | KeyCode::Backspace => {
                            app.exit_detail();
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
                    let activates = timer.activates.clone();
                    app.detail_status = fetch_timer_status(&unit).await;
                    app.detail_logs = fetch_timer_logs(&activates).await;
                }
            }

            last_refresh = Instant::now();
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
