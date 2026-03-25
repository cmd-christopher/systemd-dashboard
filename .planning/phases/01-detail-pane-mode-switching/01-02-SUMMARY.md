---
phase: 01-detail-pane-mode-switching
plan: 02
subsystem: api
tags: [rust, systemd, tokio, detail-view, testing]
requires:
  - phase: 01-detail-pane-mode-switching
    provides: "Detail pane mode state in App for later UI wiring"
provides:
  - "Async service-unit file fetch helper using systemctl --user cat"
  - "Readable fallback formatting for service-file command failures and empty output"
  - "Unit tests for service-file output normalization"
affects: [detail-view, input-handling, ui-rendering]
tech-stack:
  added: []
  patterns: [command-output-normalization, fallback-string-contracts]
key-files:
  created: []
  modified: [src/systemd.rs]
key-decisions:
  - "Service-file fetch errors return readable fallback text instead of empty content so the detail pane never renders blank for DTL-03."
  - "Normalization stays in a pure helper so unit tests can cover command output behavior without requiring live systemctl access."
patterns-established:
  - "System command helpers in src/systemd.rs should convert subprocess output into UI-safe strings."
  - "Fallback contracts for detail-pane data sources should be covered with direct byte-slice unit tests."
requirements-completed: [DTL-03]
duration: 4min
completed: 2026-03-25
---

# Phase 01 Plan 02: Service File Fetch Summary

**Service-unit file content is now fetched through `systemctl --user cat` with explicit fallback text and normalization tests for Service File mode**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-25T15:40:14Z
- **Completed:** 2026-03-25T15:44:14Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- Added `fetch_service_file_content(service_unit: &str) -> String` to read service-unit text for the selected timer service.
- Added `normalize_service_file_output` so empty output and command failures become `Service file unavailable: ...` messages.
- Added focused unit tests that lock successful stdout passthrough, stderr fallback, and empty-success fallback behavior.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement service-file fetching for the selected timer service unit** - `d97c729` (test), `2db901b` (feat)
2. **Task 2: Add unit tests for service-file normalization and fallback behavior** - `49e7335` (test)

## Files Created/Modified
- `src/systemd.rs` - Added the service-file fetch helper, normalization helper, and unit tests for DTL-03 fallback behavior.

## Decisions Made
- Command failure details come from stderr when present, otherwise the helper uses `empty output` to keep the fallback explicit.
- The helper returns the original stdout string when the command succeeds with non-empty output, preserving systemctl formatting.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The first `cargo test systemd::tests -- --nocapture` rerun surfaced transient compile errors from local `src/app.rs` changes already present in the workspace; the same command passed on immediate rerun without changing scope.
- `git add` and `git commit` required escalated permissions because the sandbox cannot write `.git/index.lock`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `src/main.rs` and `src/ui.rs` can now wire `Service File` mode to a stable helper instead of embedding subprocess logic.
- The remaining plan can switch between logs and service-file content without adding new systemd integration work.

## Self-Check: PASSED

---
*Phase: 01-detail-pane-mode-switching*
*Completed: 2026-03-25*
