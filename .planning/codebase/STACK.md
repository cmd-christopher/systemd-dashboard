# Technology Stack

**Analysis Date:** 2026-03-25

## Languages

**Primary:**
- Rust (edition 2024) - Application code and binary entrypoint in `Cargo.toml`, `src/main.rs`, `src/app.rs`, `src/systemd.rs`, and `src/ui.rs`

**Secondary:**
- Markdown - Project documentation in `README.md`
- Shell command interfaces - Runtime integration points invoked from Rust via `tokio::process::Command` in `src/systemd.rs`

## Runtime

**Environment:**
- Native Rust binary on Linux user sessions; the app relies on `systemctl --user` and `journalctl --user` calls in `src/systemd.rs`
- Tokio async runtime via `#[tokio::main]` in `src/main.rs`

**Package Manager:**
- Cargo - Rust package management defined by `Cargo.toml`
- Lockfile: present as `Cargo.lock`

## Frameworks

**Core:**
- Ratatui `0.26` - Terminal UI widgets, layout, and rendering in `Cargo.toml` and `src/ui.rs`
- Crossterm `0.27` - Raw terminal mode, alternate screen, and keyboard event handling in `Cargo.toml` and `src/main.rs`
- Tokio `1.37` with `full` feature set - Async runtime and subprocess execution in `Cargo.toml`, `src/main.rs`, and `src/systemd.rs`

**Testing:**
- Not detected; no test framework dependency or test files are present in `Cargo.toml` or under `src/`

**Build/Dev:**
- Cargo - Build and run workflow described in `README.md` and implied by `Cargo.toml`
- rustc toolchain - Required for development per `README.md`; no pinned `rust-toolchain.toml` or `.rust-version` file is present in the repository root

## Key Dependencies

**Critical:**
- `ratatui = "0.26"` - Primary TUI framework used to render tables, blocks, lists, and layout in `src/ui.rs`
- `crossterm = "0.27"` - Terminal backend and event source used to manage raw mode and keyboard input in `src/main.rs`
- `tokio = { version = "1.37", features = ["full"] }` - Async runtime used for the main entrypoint and subprocess I/O in `src/main.rs` and `src/systemd.rs`

**Infrastructure:**
- `serde = { version = "1.0", features = ["derive"] }` - Deserializes systemd JSON output into `RawTimerInfo` in `src/systemd.rs`
- `serde_json = "1.0"` - Parses `systemctl --output json` results in `src/systemd.rs`
- `chrono = "0.4"` - Converts microsecond timestamps to local absolute and relative times in `src/systemd.rs`

## Configuration

**Environment:**
- No `.env`, `.env.*`, or other env-specific config files were detected at the repository root during analysis
- No environment variables are read in application code; configuration is implicit in the local user environment and availability of `systemctl` and `journalctl` as used in `src/systemd.rs`
- Runtime behavior is driven by hard-coded polling intervals and command arguments in `src/main.rs` and `src/systemd.rs`

**Build:**
- `Cargo.toml` - Package manifest and dependency declarations
- `Cargo.lock` - Resolved dependency lockfile
- No CI, container, Nix, or cross-compilation config files were detected in the repository root

## Platform Requirements

**Development:**
- Recent Rust toolchain with `cargo` and `rustc`, per `README.md`
- Linux environment with user-level systemd and journal access, because `src/systemd.rs` invokes `systemctl --user` and `journalctl --user`
- Interactive terminal that supports alternate screen and raw mode, required by the Crossterm setup in `src/main.rs`

**Production:**
- Local terminal execution of the compiled binary `systemd-dashboard`; there is no server deployment target defined in `Cargo.toml` or `README.md`
- Intended target is a Linux desktop or server session where user systemd timers exist, as described in `README.md`

---

*Stack analysis: 2026-03-25*
