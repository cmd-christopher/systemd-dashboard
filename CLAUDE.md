<!-- GSD:project-start source:PROJECT.md -->
## Project

**Systemd Timer Dashboard**

Systemd Timer Dashboard is a Rust terminal UI for monitoring and controlling user-level systemd timers. It provides a timer list view and a detail view for a selected timer, including timer metadata and recent service logs. This milestone extends the detail screen interaction model so users can actively control what appears in the bottom pane.

**Core Value:** A user can quickly inspect and control a selected timer without leaving the terminal.

### Constraints

- **Tech stack**: Rust + Ratatui + Crossterm + Tokio — keep consistency with current architecture
- **Platform**: Linux user-level systemd environment — feature must rely on `systemctl --user` and related tooling
- **Interaction model**: Keyboard-driven terminal UX — no mouse-only interactions
- **Performance**: Preserve responsive event loop and periodic refresh behavior
<!-- GSD:project-end -->

<!-- GSD:stack-start source:codebase/STACK.md -->
## Technology Stack

## Languages
- Rust (edition 2024) - Application code and binary entrypoint in `Cargo.toml`, `src/main.rs`, `src/app.rs`, `src/systemd.rs`, and `src/ui.rs`
- Markdown - Project documentation in `README.md`
- Shell command interfaces - Runtime integration points invoked from Rust via `tokio::process::Command` in `src/systemd.rs`
## Runtime
- Native Rust binary on Linux user sessions; the app relies on `systemctl --user` and `journalctl --user` calls in `src/systemd.rs`
- Tokio async runtime via `#[tokio::main]` in `src/main.rs`
- Cargo - Rust package management defined by `Cargo.toml`
- Lockfile: present as `Cargo.lock`
## Frameworks
- Ratatui `0.26` - Terminal UI widgets, layout, and rendering in `Cargo.toml` and `src/ui.rs`
- Crossterm `0.27` - Raw terminal mode, alternate screen, and keyboard event handling in `Cargo.toml` and `src/main.rs`
- Tokio `1.37` with `full` feature set - Async runtime and subprocess execution in `Cargo.toml`, `src/main.rs`, and `src/systemd.rs`
- Not detected; no test framework dependency or test files are present in `Cargo.toml` or under `src/`
- Cargo - Build and run workflow described in `README.md` and implied by `Cargo.toml`
- rustc toolchain - Required for development per `README.md`; no pinned `rust-toolchain.toml` or `.rust-version` file is present in the repository root
## Key Dependencies
- `ratatui = "0.26"` - Primary TUI framework used to render tables, blocks, lists, and layout in `src/ui.rs`
- `crossterm = "0.27"` - Terminal backend and event source used to manage raw mode and keyboard input in `src/main.rs`
- `tokio = { version = "1.37", features = ["full"] }` - Async runtime used for the main entrypoint and subprocess I/O in `src/main.rs` and `src/systemd.rs`
- `serde = { version = "1.0", features = ["derive"] }` - Deserializes systemd JSON output into `RawTimerInfo` in `src/systemd.rs`
- `serde_json = "1.0"` - Parses `systemctl --output json` results in `src/systemd.rs`
- `chrono = "0.4"` - Converts microsecond timestamps to local absolute and relative times in `src/systemd.rs`
## Configuration
- No `.env`, `.env.*`, or other env-specific config files were detected at the repository root during analysis
- No environment variables are read in application code; configuration is implicit in the local user environment and availability of `systemctl` and `journalctl` as used in `src/systemd.rs`
- Runtime behavior is driven by hard-coded polling intervals and command arguments in `src/main.rs` and `src/systemd.rs`
- `Cargo.toml` - Package manifest and dependency declarations
- `Cargo.lock` - Resolved dependency lockfile
- No CI, container, Nix, or cross-compilation config files were detected in the repository root
## Platform Requirements
- Recent Rust toolchain with `cargo` and `rustc`, per `README.md`
- Linux environment with user-level systemd and journal access, because `src/systemd.rs` invokes `systemctl --user` and `journalctl --user`
- Interactive terminal that supports alternate screen and raw mode, required by the Crossterm setup in `src/main.rs`
- Local terminal execution of the compiled binary `systemd-dashboard`; there is no server deployment target defined in `Cargo.toml` or `README.md`
- Intended target is a Linux desktop or server session where user systemd timers exist, as described in `README.md`
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

## Naming Patterns
- Use short, lowercase module names matching the domain they own, as in `src/app.rs`, `src/systemd.rs`, `src/ui.rs`, and `src/main.rs`.
- Use `snake_case` for free functions and methods, including async functions such as `run_app` in `src/main.rs`, `fetch_timers` and `toggle_timer` in `src/systemd.rs`, and render helpers such as `draw_list` in `src/ui.rs`.
- Use `snake_case` for locals and fields, including state names such as `selected_index`, `detail_status`, `detail_logs`, and `detail_scroll` in `src/app.rs`, plus derived locals such as `last_refresh`, `tick_rate`, and `is_active` in `src/main.rs`.
- Use `PascalCase` for structs and enums, as shown by `App` and `ViewMode` in `src/app.rs` and `RawTimerInfo` and `TimerInfo` in `src/systemd.rs`.
## Code Style
- No repo-local formatter config is present; follow standard `rustfmt` defaults for this crate.
- Multi-line builder chains and argument lists are the dominant style, as shown by the `crossterm` setup in `src/main.rs` and widget construction in `src/ui.rs`.
- Struct literals expand one field per line when state is initialized, as in `App::new` in `src/app.rs`.
- No `clippy.toml`, `.clippy.toml`, or workspace lint config is present.
- Follow compiler-clean Rust and keep patterns simple enough to pass default Clippy expectations. The codebase currently relies on direct control flow and explicit matches rather than lint-driven abstractions.
## Import Organization
- No path aliases are used. Import through `crate::...` for local modules and direct crate names for dependencies.
## Error Handling
- Use `Result<_, String>` at the process-bound system layer to convert command and parse failures into user-displayable strings, as in `fetch_timers` and `toggle_timer` in `src/systemd.rs`.
- Use `?` for terminal and IO failures that should unwind to the top-level loop, as in `main` and `run_app` in `src/main.rs`.
- Use `if let Ok(...)` when refresh failures should be ignored and the UI should continue with stale state, as in the initial timer fetch and periodic refresh paths in `src/main.rs`.
- Return fallback strings instead of bubbling errors for detail views, as in `fetch_timer_status` and `fetch_timer_logs` in `src/systemd.rs`.
- Avoid panics: no `unwrap()` or `expect()` calls are present in `src/`.
## Logging
- Error output is limited to a single top-level `println!("{:?}", err)` in `src/main.rs` after terminal restoration.
- No structured logging crate is configured. Keep runtime noise low and route recoverable command failures into `App` state or returned `String` errors instead of adding ad hoc logging.
## Comments
- Use short comments to mark major procedural phases, not to restate obvious code. Current examples include `// Terminal setup`, `// Create app and run it`, and `// Restore terminal` in `src/main.rs`, plus numbered extraction steps in `src/systemd.rs`.
- Inline comments are used sparingly to label non-obvious constants, such as the percentage columns in `src/ui.rs`.
- Not applicable. No Rust doc comments (`///`) are present in `src/`.
## Function Design
## Module Design
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

## Pattern Overview
- `src/main.rs` owns application bootstrap, terminal lifecycle, input polling, periodic refresh, and async orchestration.
- `src/app.rs` is the in-memory state container; UI rendering and event handling both mutate the same `App` instance.
- `src/systemd.rs` isolates all external process interaction with `systemctl` and `journalctl`, then transforms raw command output into display-ready models.
## Layers
- Purpose: Initialize the Tokio runtime and terminal backend, then run the main loop.
- Location: `src/main.rs`
- Contains: `main`, `run_app`, raw-mode setup, alternate-screen setup, event polling, refresh timers.
- Depends on: `tokio`, `crossterm`, `ratatui`, `crate::app`, `crate::systemd`, `crate::ui`.
- Used by: The compiled binary entry point from `Cargo.toml`.
- Purpose: Hold UI state and selection state across list/detail modes.
- Location: `src/app.rs`
- Contains: `App`, `ViewMode`, navigation helpers, detail-mode transitions, selected-item lookup.
- Depends on: `crate::systemd::TimerInfo`.
- Used by: `src/main.rs` for mutation and `src/ui.rs` for rendering.
- Purpose: Fetch timer data, logs, and status from systemd/journald and normalize it for the UI.
- Location: `src/systemd.rs`
- Contains: `RawTimerInfo`, `TimerInfo`, `fetch_timers`, `fetch_timer_status`, `fetch_timer_logs`, `toggle_timer`, time-format helpers.
- Depends on: `tokio::process::Command`, `serde`, `serde_json`, `chrono`, `std::collections::HashMap`.
- Used by: `src/main.rs` and indirectly `src/app.rs` through the `TimerInfo` type.
- Purpose: Convert `App` state into Ratatui widgets for list, detail, and error views.
- Location: `src/ui.rs`
- Contains: `draw_ui`, `draw_list`, `draw_detail`, `draw_error`.
- Depends on: `ratatui`, `crate::app::{App, ViewMode}`.
- Used by: `src/main.rs` inside `terminal.draw(...)`.
## Data Flow
- All mutable application state lives in one `App` struct in `src/app.rs`.
- There is no message bus, reducer, or store abstraction; `src/main.rs` mutates fields directly.
- Render functions in `src/ui.rs` read from `App` and create ephemeral widget state such as `TableState` and `ListState`.
## Key Abstractions
- Purpose: Represent the full interactive UI session state.
- Examples: `src/app.rs`
- Pattern: Central mutable state object shared between controller logic in `src/main.rs` and render logic in `src/ui.rs`.
- Purpose: Switch rendering and key handling between the list screen and detail screen.
- Examples: `src/app.rs`, `src/main.rs`, `src/ui.rs`
- Pattern: Small enum-driven screen state.
- Purpose: Carry display-ready timer data including absolute/relative times, schedule, and status.
- Examples: `src/systemd.rs`, `src/app.rs`
- Pattern: View model assembled in the integration layer before reaching the UI.
- Purpose: Match the JSON shape returned by `systemctl --user list-timers --output json`.
- Examples: `src/systemd.rs`
- Pattern: Deserialize-then-transform DTO used only at the integration boundary.
## Entry Points
- Location: `src/main.rs`
- Triggers: `cargo run`, `cargo build`, or the compiled `systemd-dashboard` binary.
- Responsibilities: Initialize terminal state, construct `App`, fetch initial data, run the event loop, and restore the terminal on exit.
- Location: `src/ui.rs`
- Triggers: `terminal.draw(|f| ui::draw_ui(f, app))` from `src/main.rs`.
- Responsibilities: Branch on `ViewMode`, render the timer table or detail screen, and overlay any error panel.
- Location: `src/systemd.rs`
- Triggers: User actions and periodic refreshes initiated from `src/main.rs`.
- Responsibilities: Read timer lists, status, and logs, and execute start/stop operations.
## Error Handling
- `src/main.rs` propagates terminal setup failures with `?`, but logs runtime loop errors via `println!("{:?}", err)`.
- `src/systemd.rs` converts process and parse failures into `String` values, especially in `fetch_timers` and `toggle_timer`.
- `src/main.rs` often ignores fetch failures with `if let Ok(...) = ...`, which preserves UI responsiveness but drops error details instead of storing them in `app.error`.
## Cross-Cutting Concerns
<!-- GSD:architecture-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd:quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd:debug` for investigation and bug fixing
- `/gsd:execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd:profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
