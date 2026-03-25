# Systemd Timer Dashboard

## What This Is

Systemd Timer Dashboard is a Rust terminal UI for monitoring and controlling user-level systemd timers. It provides a timer list view and a detail view for a selected timer, including timer metadata and recent service logs. This milestone extends the detail screen interaction model so users can actively control what appears in the bottom pane.

## Core Value

A user can quickly inspect and control a selected timer without leaving the terminal.

## Requirements

### Validated

- ✓ User can view all user-level timers and their schedule/next/last/status values — existing
- ✓ User can open a timer detail screen and inspect selected timer metadata — existing
- ✓ User can view recent logs for the selected timer's service unit — existing
- ✓ User can start/stop a timer from the list using keyboard controls — existing

### Active

- [ ] In detail view, the bottom pane can display either service logs or the selected service unit file contents.
- [ ] The default bottom-pane content in detail view is logs.
- [ ] Pressing `Tab` in detail view toggles active selection between top pane and bottom pane.
- [ ] When top pane is active, arrow keys switch bottom-pane source between Logs and Service File.
- [ ] Bottom-pane source selection is visible in the UI so users know which source is currently shown.

### Out of Scope

- Editing service unit files from the TUI — read-only inspection is sufficient for this phase
- Multi-pane resizing or layout reconfiguration — keep existing split layout for now
- Persisting per-timer pane selection across app restarts — session-only behavior is sufficient

## Context

The project is a brownfield Rust TUI codebase using Ratatui + Crossterm + Tokio in a single binary. The detail screen currently renders timer status in the top pane and a scrollable log list in the bottom pane. Input handling is centralized in `src/main.rs`, shared app state lives in `src/app.rs`, rendering is in `src/ui.rs`, and systemd/journal interactions are in `src/systemd.rs`.

## Constraints

- **Tech stack**: Rust + Ratatui + Crossterm + Tokio — keep consistency with current architecture
- **Platform**: Linux user-level systemd environment — feature must rely on `systemctl --user` and related tooling
- **Interaction model**: Keyboard-driven terminal UX — no mouse-only interactions
- **Performance**: Preserve responsive event loop and periodic refresh behavior

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Keep detail layout as top status pane + bottom content pane | Existing users already understand this structure and it fits current UI rendering | — Pending |
| Default bottom-pane source to Logs | Preserves current behavior and avoids surprising existing users | — Pending |
| Use `Tab` to change active pane and arrow keys in top pane to change source | Matches keyboard-first TUI expectations and keeps controls discoverable | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `$gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `$gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-03-25 after initialization*
