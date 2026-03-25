# Codebase Structure

**Analysis Date:** 2026-03-25

## Directory Layout

```text
[project-root]/
├── .planning/codebase/  # Generated mapper documents for planning workflows
├── src/                 # Rust crate source for the TUI application
├── target/              # Cargo build artifacts
├── Cargo.toml           # Crate manifest and dependency declarations
└── README.md            # User-facing project overview and usage notes
```

## Directory Purposes

**`src/`:**
- Purpose: Hold all handwritten application code for the binary crate.
- Contains: Entry-point code, application state, systemd integration code, and Ratatui rendering code.
- Key files: `src/main.rs`, `src/app.rs`, `src/systemd.rs`, `src/ui.rs`

**`.planning/codebase/`:**
- Purpose: Hold generated repository reference documents used by GSD planning and execution commands.
- Contains: Markdown analysis files such as `.planning/codebase/ARCHITECTURE.md` and `.planning/codebase/STRUCTURE.md`.
- Key files: `.planning/codebase/ARCHITECTURE.md`, `.planning/codebase/STRUCTURE.md`

**`target/`:**
- Purpose: Hold Cargo-generated build outputs and incremental compilation state.
- Contains: `debug/`, `release/`, fingerprints, dependency build outputs.
- Key files: Not applicable for source edits; treat `target/` as generated output.

## Key File Locations

**Entry Points:**
- `src/main.rs`: Binary entry point, terminal setup/teardown, async event loop, refresh orchestration, and input handling.

**Configuration:**
- `Cargo.toml`: Rust package metadata, edition, and external crate dependencies.
- `README.md`: Human-readable overview, runtime expectations, and keybindings.

**Core Logic:**
- `src/app.rs`: Application session state and navigation helpers.
- `src/systemd.rs`: System command execution and data transformation.
- `src/ui.rs`: Ratatui widget composition for all screens.

**Testing:**
- Not detected. No `tests/` directory, `*.test.*`, `*.spec.*`, or Rust `#[cfg(test)]` modules were found in tracked source files.

## Naming Conventions

**Files:**
- Use flat, snake_case Rust module filenames under `src/`: `app.rs`, `systemd.rs`, `ui.rs`.
- Keep the binary entry point in `src/main.rs`.

**Directories:**
- Use standard Rust crate layout with a single top-level `src/` directory for source.
- Keep generated planning artifacts under `.planning/codebase/`.

## Where to Add New Code

**New Feature:**
- Primary code: Add a new module under `src/` when the behavior forms a distinct concern, then register it with `mod ...;` in `src/main.rs`.
- Tests: Add inline Rust test modules inside the owning source file or create a top-level `tests/` directory if feature coverage expands beyond unit tests.

**New Component/Module:**
- Implementation: Place stateful domain/controller code beside existing modules in `src/`, following the current split of `app` for state, `systemd` for external integration, and `ui` for rendering.

**Utilities:**
- Shared helpers: Keep helper functions inside the owning module until reuse clearly spans multiple modules; if reuse grows, extract a dedicated `src/<concern>.rs` module and import it from consumers.

## Special Directories

**`target/`:**
- Purpose: Cargo-generated build artifacts.
- Generated: Yes
- Committed: No

**`.planning/codebase/`:**
- Purpose: Generated planning reference documents.
- Generated: Yes
- Committed: Yes

## Placement Guidance

- Put terminal bootstrap code, event polling, and top-level orchestration in `src/main.rs`; the current architecture does not use a separate controller module.
- Put persistent UI/session state in `src/app.rs`; extend `App` and `ViewMode` before introducing a new global state abstraction.
- Put all `systemctl` and `journalctl` process interaction in `src/systemd.rs`; do not call shell commands directly from `src/ui.rs`.
- Put widget layout and styling in `src/ui.rs`; keep rendering functions pure over `App` input except for local widget state like `TableState`.
- Keep the source tree flat unless a new concern becomes large enough to justify a submodule tree; the current crate has four top-level source files only.

---

*Structure analysis: 2026-03-25*
