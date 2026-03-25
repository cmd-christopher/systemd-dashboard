# Roadmap: Systemd Timer Dashboard

## Overview

This milestone delivers a complete detail-view interaction upgrade so users can inspect either logs or service unit contents, move focus between panes, and understand what is active without leaving the terminal.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Detail Pane Mode Switching** - Deliver the full detail-view pane switching workflow, including navigation, content rendering, and UI feedback.

## Phase Details

### Phase 1: Detail Pane Mode Switching
**Goal**: Users can inspect and control detail-view bottom-pane content from the keyboard while always knowing which pane and content mode are active.
**Depends on**: Nothing (first phase)
**Requirements**: DTL-01, DTL-02, DTL-03, NAV-01, NAV-02, NAV-03, UI-01, UI-02
**Success Criteria** (what must be TRUE):
  1. User can open detail view and immediately see recent logs in the bottom pane by default.
  2. User can switch the bottom pane between `Logs` and `Service File` for the selected timer and the shown content updates accordingly.
  3. User can press `Tab` to move active focus between the top and bottom panes, with the active pane clearly indicated in the UI.
  4. User can use arrow keys to change content mode only while the top pane is active, and can scroll the current bottom-pane content while the bottom pane is active.
  5. User can always tell whether the bottom pane is showing `Logs` or `Service File`.
**Plans**: 3 plans
Plans:
- [x] 01-01-PLAN.md — Define detail-pane state contracts and app-level navigation tests
- [x] 01-02-PLAN.md — Add service-file content fetching and fallback tests
- [ ] 01-03-PLAN.md — Wire pane focus, mode switching, and detail UI indicators
**UI hint**: yes

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 1.1 → 1.2 → 2

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Detail Pane Mode Switching | 1/3 | In Progress | 2026-03-25 |
