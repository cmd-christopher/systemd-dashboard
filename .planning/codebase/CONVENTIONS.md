# Coding Conventions

**Analysis Date:** 2026-03-25

## Naming Patterns

**Files:**
- Use short, lowercase module names matching the domain they own, as in `src/app.rs`, `src/systemd.rs`, `src/ui.rs`, and `src/main.rs`.

**Functions:**
- Use `snake_case` for free functions and methods, including async functions such as `run_app` in `src/main.rs`, `fetch_timers` and `toggle_timer` in `src/systemd.rs`, and render helpers such as `draw_list` in `src/ui.rs`.

**Variables:**
- Use `snake_case` for locals and fields, including state names such as `selected_index`, `detail_status`, `detail_logs`, and `detail_scroll` in `src/app.rs`, plus derived locals such as `last_refresh`, `tick_rate`, and `is_active` in `src/main.rs`.

**Types:**
- Use `PascalCase` for structs and enums, as shown by `App` and `ViewMode` in `src/app.rs` and `RawTimerInfo` and `TimerInfo` in `src/systemd.rs`.

## Code Style

**Formatting:**
- No repo-local formatter config is present; follow standard `rustfmt` defaults for this crate.
- Multi-line builder chains and argument lists are the dominant style, as shown by the `crossterm` setup in `src/main.rs` and widget construction in `src/ui.rs`.
- Struct literals expand one field per line when state is initialized, as in `App::new` in `src/app.rs`.

**Linting:**
- No `clippy.toml`, `.clippy.toml`, or workspace lint config is present.
- Follow compiler-clean Rust and keep patterns simple enough to pass default Clippy expectations. The codebase currently relies on direct control flow and explicit matches rather than lint-driven abstractions.

## Import Organization

**Order:**
1. Internal crate imports, such as `use crate::app::{App, ViewMode};` and `use crate::systemd::TimerInfo;` in `src/main.rs`, `src/app.rs`, and `src/ui.rs`
2. External crate imports, such as `use crossterm::{...};`, `use ratatui::{...};`, `use tokio::process::Command;`, and `use serde::Deserialize;`
3. Standard library imports, such as `use std::{error::Error, io, time::{Duration, Instant}};` in `src/main.rs` and `use std::collections::HashMap;` in `src/systemd.rs`

**Path Aliases:**
- No path aliases are used. Import through `crate::...` for local modules and direct crate names for dependencies.

## Error Handling

**Patterns:**
- Use `Result<_, String>` at the process-bound system layer to convert command and parse failures into user-displayable strings, as in `fetch_timers` and `toggle_timer` in `src/systemd.rs`.
- Use `?` for terminal and IO failures that should unwind to the top-level loop, as in `main` and `run_app` in `src/main.rs`.
- Use `if let Ok(...)` when refresh failures should be ignored and the UI should continue with stale state, as in the initial timer fetch and periodic refresh paths in `src/main.rs`.
- Return fallback strings instead of bubbling errors for detail views, as in `fetch_timer_status` and `fetch_timer_logs` in `src/systemd.rs`.
- Avoid panics: no `unwrap()` or `expect()` calls are present in `src/`.

## Logging

**Framework:** `println!`

**Patterns:**
- Error output is limited to a single top-level `println!("{:?}", err)` in `src/main.rs` after terminal restoration.
- No structured logging crate is configured. Keep runtime noise low and route recoverable command failures into `App` state or returned `String` errors instead of adding ad hoc logging.

## Comments

**When to Comment:**
- Use short comments to mark major procedural phases, not to restate obvious code. Current examples include `// Terminal setup`, `// Create app and run it`, and `// Restore terminal` in `src/main.rs`, plus numbered extraction steps in `src/systemd.rs`.
- Inline comments are used sparingly to label non-obvious constants, such as the percentage columns in `src/ui.rs`.

**JSDoc/TSDoc:**
- Not applicable. No Rust doc comments (`///`) are present in `src/`.

## Function Design

**Size:** Keep functions focused on one responsibility. State transitions in `src/app.rs` stay small, render helpers in `src/ui.rs` split list/detail/error drawing, and system command helpers in `src/systemd.rs` each wrap one external command or formatting task.

**Parameters:** Pass mutable app state explicitly (`&mut App`) in UI and event-loop code, and pass borrowed string slices (`&str`) into system command helpers, as in `fetch_timer_status`, `fetch_timer_logs`, and `toggle_timer` in `src/systemd.rs`.

**Return Values:** Use concrete return types over traits where possible. Examples include `Option<&TimerInfo>` from `App::selected_timer` in `src/app.rs`, `String` from formatting helpers in `src/systemd.rs`, and `io::Result<()>` from `run_app` in `src/main.rs`.

## Module Design

**Exports:** Keep modules flat and export directly from each source file. `src/main.rs` declares `mod app;`, `mod systemd;`, and `mod ui;`, then consumes functions and types through explicit `use crate::...` imports.

**Barrel Files:** Not used. There is no `lib.rs` or re-export module; add new code as a dedicated file under `src/` and import it explicitly from `src/main.rs` or sibling modules.

---

*Convention analysis: 2026-03-25*
