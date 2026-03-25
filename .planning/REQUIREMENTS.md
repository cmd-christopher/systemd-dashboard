# Requirements: Systemd Timer Dashboard

**Defined:** 2026-03-25
**Core Value:** A user can quickly inspect and control a selected timer without leaving the terminal.

## v1 Requirements

### Detail Pane Content

- [ ] **DTL-01**: User can switch the detail view bottom pane between `Logs` and `Service File` content modes.
- [ ] **DTL-02**: When entering detail view, the default bottom-pane content mode is `Logs`.
- [ ] **DTL-03**: In `Service File` mode, the bottom pane shows readable service unit file contents for the selected timer's service unit.

### Detail Navigation

- [ ] **NAV-01**: User can press `Tab` in detail view to toggle active focus between top pane and bottom pane.
- [ ] **NAV-02**: When top pane is active, `Left/Right` or `Up/Down` arrow keys change the bottom-pane content mode.
- [ ] **NAV-03**: When bottom pane is active, arrow keys scroll content instead of changing content mode.

### Detail UI Feedback

- [ ] **UI-01**: Detail view clearly indicates which pane is active.
- [ ] **UI-02**: Detail view clearly indicates the current bottom-pane content mode (`Logs` or `Service File`).

## v2 Requirements

### Detail Enhancements

- **DTX-01**: User can refresh detail pane content on demand without waiting for periodic refresh.
- **DTX-02**: User can persist last-selected bottom-pane mode per timer across sessions.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Editing and saving service unit files | Unsafe for this dashboard scope; inspection only |
| Split-pane resizing or arbitrary layout edits | Not required to deliver requested behavior |
| Multi-tab detail panes beyond logs/service file | Keeps interaction model focused for v1 |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| DTL-01 | Phase 1 | Pending |
| DTL-02 | Phase 1 | Pending |
| DTL-03 | Phase 1 | Pending |
| NAV-01 | Phase 1 | Pending |
| NAV-02 | Phase 1 | Pending |
| NAV-03 | Phase 1 | Pending |
| UI-01 | Phase 1 | Pending |
| UI-02 | Phase 1 | Pending |

**Coverage:**
- v1 requirements: 8 total
- Mapped to phases: 8
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-25*
*Last updated: 2026-03-25 after initial definition*
