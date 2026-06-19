mod app;
mod logging;
mod systemd;
mod ui;

use crate::app::{App, DetailContentMode, DetailPaneFocus, ViewMode};
use crate::logging::init_file_logging;
use crate::systemd::{
    fetch_service_file_content, fetch_timer_logs, fetch_timer_status, fetch_timers,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use tracing::{debug, error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Logging: file-backed sink so stdout/stderr stay clean for the TUI.
    let log_dir = crate::logging::default_log_dir().map_err(|e| -> Box<dyn Error> { e.into() })?;
    let _log_guard = init_file_logging(&log_dir).map_err(|e| -> Box<dyn Error> { e.into() })?;
    info!(log_dir = ?log_dir, "systemd-dashboard starting up");

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    debug!("terminal backend initialized");

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

    match &res {
        Ok(()) => info!("event loop exited cleanly"),
        Err(err) => error!(error = %err, "event loop returned error"),
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
    debug!(tick_rate_ms = 250, "event loop starting");

    // Initial fetch
    match fetch_timers().await {
        Ok(timers) => {
            info!(count = timers.len(), "initial timer fetch succeeded");
            app.replace_timers(timers);
            app.error = None;
        }
        Err(e) => {
            error!(error = %e, "initial timer fetch failed");
            app.error = Some(e);
        }
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
                    ViewMode::List => {
                        if handle_list_input(app, key.code).await {
                            info!(key = ?key.code, "list view: quit requested");
                            return Ok(());
                        }
                    }
                    ViewMode::Detail => {
                        if handle_detail_input(app, key.code).await {
                            info!(key = ?key.code, "detail view: quit requested");
                            return Ok(());
                        }
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            if let ViewMode::Detail = app.mode {
                // More frequent refresh for detail view to support "real-time" logs
                if last_refresh.elapsed() >= Duration::from_secs(2) {
                    if let Some(timer) = app.selected_timer() {
                        let unit = timer.unit.clone();
                        debug!(unit = %unit, "periodic detail refresh");
                        match fetch_timer_status(&unit).await {
                            Ok(status) => {
                                app.detail_status = status;
                                app.update_status_text();
                            }
                            Err(e) => {
                                warn!(unit = %unit, error = %e, "periodic status fetch failed");
                                app.error = Some(e);
                                app.detail_status = "Error fetching status".to_string();
                                app.update_status_text();
                            }
                        }
                        refresh_detail_content(app, false).await;
                    }
                    last_refresh = Instant::now();
                }
            } else if last_refresh.elapsed() >= Duration::from_secs(60) {
                // Slower refresh for the main list
                debug!("periodic list refresh");
                match fetch_timers().await {
                    Ok(timers) => {
                        app.replace_timers(timers);
                        app.error = None;
                    }
                    Err(e) => {
                        warn!(error = %e, "periodic list refresh failed");
                        app.error = Some(e);
                    }
                }
                last_refresh = Instant::now();
            }

            last_tick = Instant::now();
        }
    }
}

async fn handle_toggle_timer(app: &mut App) {
    let toggle_op = if let Some(timer) = app.selected_timer() {
        let is_active = timer.status == "Active" || timer.status == "Waiting";
        let unit = timer.unit.clone();
        info!(unit = %unit, start = !is_active, "toggle timer requested");
        Some((
            unit,
            crate::systemd::toggle_timer(&timer.unit, !is_active).await,
        ))
    } else {
        warn!("toggle requested with no selected timer");
        None
    };

    if let Some((unit, result)) = toggle_op {
        match result {
            Ok(_) => {
                info!(unit = %unit, "toggle timer succeeded");
                app.error = None;
                match fetch_timers().await {
                    Ok(timers) => app.replace_timers(timers),
                    Err(e) => {
                        warn!(unit = %unit, error = %e, "post-toggle refresh failed");
                        app.error = Some(e);
                    }
                }
            }
            Err(e) => {
                error!(unit = %unit, error = %e, "toggle timer failed");
                app.error = Some(e);
            }
        }
    }
}

async fn handle_list_input(app: &mut App, key_code: KeyCode) -> bool {
    match key_code {
        KeyCode::Char('q') => return true,
        KeyCode::Down | KeyCode::Char('j') => app.next(),
        KeyCode::Up | KeyCode::Char('k') => app.previous(),
        KeyCode::Enter => {
            let (unit, activates) = app
                .selected_timer()
                .map(|t| (t.unit.clone(), t.activates.clone()))
                .unwrap_or_default();
            debug!(unit = %unit, "enter: opening detail view");
            if !unit.is_empty() {
                let (status_res, logs_res) =
                    tokio::join!(fetch_timer_status(&unit), fetch_timer_logs(&activates));
                match status_res {
                    Ok(status) => {
                        app.detail_status = status;
                        app.update_status_text();
                    }
                    Err(e) => {
                        error!(unit = %unit, error = %e, "enter: status fetch failed");
                        app.error = Some(e);
                        app.detail_status = "Error fetching status".to_string();
                        app.update_status_text();
                    }
                }
                match logs_res {
                    Ok(logs) => app.detail_logs = logs,
                    Err(e) => {
                        error!(unit = %unit, error = %e, "enter: logs fetch failed");
                        app.error = Some(e);
                        app.detail_logs = "Error fetching logs".to_string();
                    }
                }

                app.enter_detail();
            }
        }
        KeyCode::Char(' ') => {
            handle_toggle_timer(app).await;
        }
        KeyCode::Char('r') => match fetch_timers().await {
            Ok(timers) => app.replace_timers(timers),
            Err(e) => {
                warn!(error = %e, "manual list refresh failed");
                app.error = Some(e);
            }
        },
        _ => {}
    }
    false
}

async fn handle_detail_input(app: &mut App, key_code: KeyCode) -> bool {
    match key_code {
        KeyCode::Esc | KeyCode::Backspace => {
            debug!("detail view: exit to list");
            app.exit_detail();
        }
        KeyCode::Tab => {
            debug!("detail view: toggle pane focus");
            app.toggle_detail_focus();
        }
        KeyCode::Left | KeyCode::Up if matches!(app.detail_focus, DetailPaneFocus::Top) => {
            app.select_previous_detail_content();
            refresh_detail_content(app, true).await;
        }
        KeyCode::Right | KeyCode::Down if matches!(app.detail_focus, DetailPaneFocus::Top) => {
            app.select_next_detail_content();
            refresh_detail_content(app, true).await;
        }
        KeyCode::Left | KeyCode::Up if matches!(app.detail_focus, DetailPaneFocus::Bottom) => {
            app.scroll_detail_up();
        }
        KeyCode::Right | KeyCode::Down | KeyCode::Char('j')
            if matches!(app.detail_focus, DetailPaneFocus::Bottom) =>
        {
            app.scroll_detail_down();
        }
        KeyCode::Char('k') if matches!(app.detail_focus, DetailPaneFocus::Bottom) => {
            app.scroll_detail_up();
        }
        KeyCode::Char(' ') => {
            handle_toggle_timer(app).await;
            refresh_detail_content(app, false).await;
        }
        KeyCode::Char('q') => return true,
        KeyCode::Char('r') => {
            debug!("detail view: manual refresh");
            match fetch_timers().await {
                Ok(timers) => app.replace_timers(timers),
                Err(e) => {
                    warn!(error = %e, "detail manual refresh: timer list fetch failed");
                    app.error = Some(e);
                }
            }
            if let Some(timer) = app.selected_timer() {
                match fetch_timer_status(&timer.unit).await {
                    Ok(status) => {
                        app.detail_status = status;
                        app.update_status_text();
                    }
                    Err(e) => {
                        warn!(error = %e, "detail manual refresh: status fetch failed");
                        app.error = Some(e);
                    }
                }
            }
            refresh_detail_content(app, false).await;
        }
        _ => {}
    }
    false
}

async fn refresh_detail_content(app: &mut App, reset_scroll: bool) {
    if let Some(timer) = app.selected_timer() {
        let activates = timer.activates.clone();
        debug!(activates = %activates, mode = ?app.detail_content_mode, "refresh detail content");
        match app.detail_content_mode {
            DetailContentMode::Logs => match fetch_timer_logs(&activates).await {
                Ok(logs) => app.detail_logs = logs,
                Err(e) => {
                    warn!(activates = %activates, error = %e, "detail logs fetch failed");
                    app.error = Some(e);
                    app.detail_logs = "Error fetching logs".to_string();
                }
            },
            DetailContentMode::ServiceFile => match fetch_service_file_content(&activates).await {
                Ok(content) => app.detail_logs = content,
                Err(e) => {
                    warn!(activates = %activates, error = %e, "service file fetch failed");
                    app.error = Some(e);
                    app.detail_logs = "Error fetching service file".to_string();
                }
            },
        }
        if reset_scroll {
            app.detail_scroll = 0;
            // Auto-scroll only makes sense for logs
            app.auto_scroll = matches!(app.detail_content_mode, DetailContentMode::Logs);
        }
    }
}
