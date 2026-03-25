# Codebase Concerns

**Analysis Date:** 2026-03-25

## Tech Debt

**External command boundary mixed into view-facing data assembly:**
- Issue: `fetch_timers()` both executes `systemctl` and transforms raw output into UI-ready strings and status labels, so command behavior, parsing, and presentation logic are tightly coupled.
- Files: `src/systemd.rs`
- Impact: Any change in `systemctl` output, timer status semantics, or UI requirements forces edits in one large function and raises regression risk across data loading, formatting, and rendering.
- Fix approach: Split `src/systemd.rs` into command adapters, parsing/normalization, and presentation mapping so timer fetching can be tested independently from relative-time formatting and UI labels.

**Dead error state and silent failure paths:**
- Issue: `App` carries `error: Option<String>` and `draw_ui()` renders it, but no code path sets `app.error`; most fallible operations either ignore errors or drop them on the floor.
- Files: `src/app.rs`, `src/ui.rs`, `src/main.rs`
- Impact: Operational failures present as empty tables or stale data instead of actionable feedback, which makes debugging production issues difficult.
- Fix approach: Route all `Result` failures through a single app-level error handler and render transient or persistent error banners from actual command failures.

## Known Bugs

**Detailed status panel never shows `systemctl show` output:**
- Symptoms: Entering detail view does not expose the promised deep-dive status output; the top pane only shows a summary built from `TimerInfo`.
- Files: `src/main.rs`, `src/ui.rs`
- Trigger: Open any timer detail after `app.detail_status = fetch_timer_status(&unit).await` runs in `src/main.rs`; `draw_detail()` ignores `app.detail_status` whenever `selected_timer()` exists.
- Workaround: None in the UI. The only way to inspect full unit properties is to run `systemctl --user show <timer>` outside the application.

**Command failures in detail views collapse into blank content:**
- Symptoms: Status and log panes can render empty or incomplete text when `systemctl` or `journalctl` exits with a non-zero status.
- Files: `src/systemd.rs`
- Trigger: View a timer whose backing unit no longer exists, has insufficient journal permissions, or returns an error from `systemctl show` / `journalctl`; `fetch_timer_status()` and `fetch_timer_logs()` return stdout even when the command failed and ignore stderr/status.
- Workaround: Run the corresponding system commands manually to see the actual error output.

**Timer toggle failures are invisible to the operator:**
- Symptoms: Pressing space can fail to start or stop a timer with no error indication; the list simply refreshes and continues.
- Files: `src/main.rs`, `src/systemd.rs`
- Trigger: Attempt to toggle a timer that requires different permissions or returns an error from `systemctl`; `toggle_timer()` returns `Err`, but `let _ = crate::systemd::toggle_timer(&unit, !is_active).await;` discards it.
- Workaround: Run `systemctl --user start|stop <timer>` manually to inspect the failure reason.

## Security Considerations

**Untrusted terminal content is rendered from journal output:**
- Risk: Recent logs are passed straight from `journalctl` into `ratatui` list items without sanitizing control characters.
- Files: `src/systemd.rs`, `src/ui.rs`
- Current mitigation: `Command` uses direct argv invocation rather than a shell, so shell injection risk is low.
- Recommendations: Strip or escape ANSI/control sequences from `fetch_timer_logs()` and `fetch_timer_status()` output before rendering to avoid terminal corruption or misleading UI content.

**Operational dependence on local system binaries is unchecked:**
- Risk: The application assumes `systemctl` and `journalctl` are present and usable in the current user session, but startup has no capability check and initial fetch errors are suppressed.
- Files: `src/main.rs`, `src/systemd.rs`
- Current mitigation: Individual command-launch failures are converted into `Err(String)` in `fetch_timers()`.
- Recommendations: Add a startup health check for `systemctl --user` access and surface failures in `app.error` before entering the main TUI loop.

## Performance Bottlenecks

**UI freezes on synchronous async waits inside the event loop:**
- Problem: Input handling and periodic refresh both await command execution inline, so the terminal cannot process user input while `systemctl` or `journalctl` runs.
- Files: `src/main.rs`, `src/systemd.rs`
- Cause: `run_app()` awaits `fetch_timers()`, `fetch_timer_status()`, `fetch_timer_logs()`, and `toggle_timer()` directly from the render/event loop instead of using background tasks and message passing.
- Improvement path: Move command execution to spawned tasks, update app state from task results, and keep rendering responsive while refreshes are in flight.

**Refresh path duplicates expensive command work:**
- Problem: Every 60 seconds the app re-runs `systemctl --user list-timers --all --output json` and a second `systemctl --user show "*.timer"` call, then may also run detail status and log fetches.
- Files: `src/main.rs`, `src/systemd.rs`
- Cause: `fetch_timers()` always performs two subprocess calls and reparses the full timer set even when only one selected timer is visible.
- Improvement path: Cache schedule metadata, diff timer updates, or request only the currently selected unit’s detail data when the list has not changed.

## Fragile Areas

**Schedule parsing depends on undocumented text shape:**
- Files: `src/systemd.rs`
- Why fragile: The code searches for `TimersCalendar={` lines and extracts the substring between `OnCalendar=` and ` ;`, which is tightly coupled to one textual representation from `systemctl show`.
- Safe modification: Replace ad hoc string slicing with a parser that tolerates missing fields and alternative `systemctl` output layouts, or switch to a machine-readable output mode if available.
- Test coverage: No unit tests or fixture-based parser tests exist under `src/` or `tests/`.

**Selection state can drift from refreshed timer data:**
- Files: `src/app.rs`, `src/main.rs`, `src/ui.rs`
- Why fragile: `selected_index` is preserved across refreshes, but there is no clamp or identity-based reselection after `app.timers = timers;`. If the timer list shrinks or reorders, detail view can jump to a different timer or fall back to `"Unknown"`.
- Safe modification: Rebind selection by timer unit name after refresh and clamp the index whenever `app.timers` changes.
- Test coverage: No interaction tests exist to exercise list refresh, detail mode, or selection retention.

**Terminal cleanup is not panic-safe:**
- Files: `src/main.rs`
- Why fragile: Raw mode and alternate screen are restored only after `run_app()` returns normally. Any panic between setup and cleanup can leave the user’s terminal in a broken state.
- Safe modification: Wrap terminal setup in an RAII guard that restores the terminal in `Drop`, including panic paths.
- Test coverage: No integration tests or smoke tests validate startup/teardown behavior.

## Scaling Limits

**Timer count and log volume scale linearly with subprocess output size:**
- Current capacity: The app reads full stdout buffers for complete timer lists and the last 50 log lines into memory on each request.
- Limit: Large user timer sets or verbose services increase refresh latency and make detail view stalls more obvious because parsing and rendering happen on the main loop.
- Scaling path: Stream command output incrementally, paginate logs, and decouple refresh cadence from the render loop.

## Dependencies at Risk

**No test framework or verification dependency present:**
- Risk: `Cargo.toml` declares runtime crates only (`ratatui`, `crossterm`, `tokio`, `serde`, `serde_json`, `chrono`) and the repository contains no `tests/` tree or `#[cfg(test)]` coverage.
- Impact: Regressions in time formatting, parser assumptions, command error handling, and keyboard flow can ship undetected.
- Migration plan: Add focused unit tests around `src/systemd.rs` parsing/formatting first, then add interaction-level tests for `App` state transitions in `src/app.rs`.

## Missing Critical Features

**No observable error reporting in the UI:**
- Problem: The code has an error panel mechanism but no producer for it, so users cannot distinguish “no timers” from “failed to load timers”.
- Blocks: Reliable troubleshooting and safe operation when `systemctl --user` is unavailable or partially failing.

**No automated verification path:**
- Problem: The repository has no tests, CI config, or smoke-test command documented beyond manual `cargo run --release`.
- Blocks: Safe refactoring of subprocess parsing, UI state transitions, and terminal lifecycle behavior.

## Test Coverage Gaps

**System command parsing and time formatting are untested:**
- What's not tested: `fetch_timers()` parsing, `format_time_abs()`, and `format_time_rel()` behavior for missing, invalid, and edge-case timestamps.
- Files: `src/systemd.rs`
- Risk: Small output-format or timestamp regressions can silently corrupt status labels and schedule text.
- Priority: High

**UI state transitions and refresh behavior are untested:**
- What's not tested: Navigation wraparound, detail-mode entry/exit, detail log scrolling, and selection handling after list refresh.
- Files: `src/app.rs`, `src/main.rs`, `src/ui.rs`
- Risk: Input regressions and stale-selection bugs can ship without detection because the behavior is only exercised manually.
- Priority: High

---

*Concerns audit: 2026-03-25*
