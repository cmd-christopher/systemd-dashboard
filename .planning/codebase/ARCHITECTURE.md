# Architecture

**Analysis Date:** 2026-03-25

## Pattern Overview

**Overall:** Single-binary, module-oriented terminal UI application with a centralized event loop.

**Key Characteristics:**
- `src/main.rs` owns application bootstrap, terminal lifecycle, input polling, periodic refresh, and async orchestration.
- `src/app.rs` is the in-memory state container; UI rendering and event handling both mutate the same `App` instance.
- `src/systemd.rs` isolates all external process interaction with `systemctl` and `journalctl`, then transforms raw command output into display-ready models.

## Layers

**Bootstrap and Runtime Layer:**
- Purpose: Initialize the Tokio runtime and terminal backend, then run the main loop.
- Location: `src/main.rs`
- Contains: `main`, `run_app`, raw-mode setup, alternate-screen setup, event polling, refresh timers.
- Depends on: `tokio`, `crossterm`, `ratatui`, `crate::app`, `crate::systemd`, `crate::ui`.
- Used by: The compiled binary entry point from `Cargo.toml`.

**Application State Layer:**
- Purpose: Hold UI state and selection state across list/detail modes.
- Location: `src/app.rs`
- Contains: `App`, `ViewMode`, navigation helpers, detail-mode transitions, selected-item lookup.
- Depends on: `crate::systemd::TimerInfo`.
- Used by: `src/main.rs` for mutation and `src/ui.rs` for rendering.

**System Integration Layer:**
- Purpose: Fetch timer data, logs, and status from systemd/journald and normalize it for the UI.
- Location: `src/systemd.rs`
- Contains: `RawTimerInfo`, `TimerInfo`, `fetch_timers`, `fetch_timer_status`, `fetch_timer_logs`, `toggle_timer`, time-format helpers.
- Depends on: `tokio::process::Command`, `serde`, `serde_json`, `chrono`, `std::collections::HashMap`.
- Used by: `src/main.rs` and indirectly `src/app.rs` through the `TimerInfo` type.

**Presentation Layer:**
- Purpose: Convert `App` state into Ratatui widgets for list, detail, and error views.
- Location: `src/ui.rs`
- Contains: `draw_ui`, `draw_list`, `draw_detail`, `draw_error`.
- Depends on: `ratatui`, `crate::app::{App, ViewMode}`.
- Used by: `src/main.rs` inside `terminal.draw(...)`.

## Data Flow

**Startup and Refresh Flow:**

1. `src/main.rs` creates `App::new()` and performs an initial `fetch_timers().await`.
2. `src/systemd.rs` runs `systemctl --user list-timers ... --output json`, enriches results with `systemctl --user show "*.timer"`, and returns `Vec<TimerInfo>`.
3. `src/main.rs` stores the returned timers in `app.timers` and redraws through `ui::draw_ui`.

**Detail Inspection Flow:**

1. `src/main.rs` handles `KeyCode::Enter` while in `ViewMode::List`.
2. The selected `TimerInfo` supplies `unit` and `activates`, which are passed to `fetch_timer_status` and `fetch_timer_logs` in `src/systemd.rs`.
3. `src/main.rs` writes the returned strings into `app.detail_status` and `app.detail_logs`, then flips mode with `app.enter_detail()`.

**Toggle Flow:**

1. `src/main.rs` handles `KeyCode::Char(' ')` while a timer row is selected.
2. `src/systemd.rs::toggle_timer` issues `systemctl --user start|stop <timer>`.
3. `src/main.rs` immediately re-fetches timers with `fetch_timers()` to refresh UI state.

**State Management:**
- All mutable application state lives in one `App` struct in `src/app.rs`.
- There is no message bus, reducer, or store abstraction; `src/main.rs` mutates fields directly.
- Render functions in `src/ui.rs` read from `App` and create ephemeral widget state such as `TableState` and `ListState`.

## Key Abstractions

**`App`:**
- Purpose: Represent the full interactive UI session state.
- Examples: `src/app.rs`
- Pattern: Central mutable state object shared between controller logic in `src/main.rs` and render logic in `src/ui.rs`.

**`ViewMode`:**
- Purpose: Switch rendering and key handling between the list screen and detail screen.
- Examples: `src/app.rs`, `src/main.rs`, `src/ui.rs`
- Pattern: Small enum-driven screen state.

**`TimerInfo`:**
- Purpose: Carry display-ready timer data including absolute/relative times, schedule, and status.
- Examples: `src/systemd.rs`, `src/app.rs`
- Pattern: View model assembled in the integration layer before reaching the UI.

**`RawTimerInfo`:**
- Purpose: Match the JSON shape returned by `systemctl --user list-timers --output json`.
- Examples: `src/systemd.rs`
- Pattern: Deserialize-then-transform DTO used only at the integration boundary.

## Entry Points

**Binary Entry Point:**
- Location: `src/main.rs`
- Triggers: `cargo run`, `cargo build`, or the compiled `systemd-dashboard` binary.
- Responsibilities: Initialize terminal state, construct `App`, fetch initial data, run the event loop, and restore the terminal on exit.

**Render Entry Point:**
- Location: `src/ui.rs`
- Triggers: `terminal.draw(|f| ui::draw_ui(f, app))` from `src/main.rs`.
- Responsibilities: Branch on `ViewMode`, render the timer table or detail screen, and overlay any error panel.

**System Command Entry Points:**
- Location: `src/systemd.rs`
- Triggers: User actions and periodic refreshes initiated from `src/main.rs`.
- Responsibilities: Read timer lists, status, and logs, and execute start/stop operations.

## Error Handling

**Strategy:** Mixed `Result`-based handling for startup and data fetches, with lossy string conversion for system command failures.

**Patterns:**
- `src/main.rs` propagates terminal setup failures with `?`, but logs runtime loop errors via `println!("{:?}", err)`.
- `src/systemd.rs` converts process and parse failures into `String` values, especially in `fetch_timers` and `toggle_timer`.
- `src/main.rs` often ignores fetch failures with `if let Ok(...) = ...`, which preserves UI responsiveness but drops error details instead of storing them in `app.error`.

## Cross-Cutting Concerns

**Logging:** No structured logging is implemented; runtime errors are printed from `src/main.rs`, while service logs are fetched from `journalctl` in `src/systemd.rs`.
**Validation:** Input validation is minimal and command arguments are built from selected timer fields already returned by systemd in `src/systemd.rs`.
**Authentication:** No application-level authentication exists; access is delegated to the current OS user via `systemctl --user` and `journalctl --user`.

---

*Architecture analysis: 2026-03-25*
