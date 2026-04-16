## 2025-04-15 - Contextual Keybinding Footer
**Learning:** Adding a contextual keybinding footer based on the application's view mode (`ViewMode::List` vs. `ViewMode::Detail`) drastically improves the discoverability of terminal UI (TUI) interactions without cluttering the screen or requiring the user to read documentation. Terminal applications heavily rely on keyboard shortcuts; making them explicit and contextual reduces the cognitive load.
**Action:** When designing or modifying terminal applications (or complex desktop/web app views), always consider adding unobtrusive, contextual hints for primary keybindings to improve keyboard accessibility and general UX.

## 2024-04-16 - Popups Need the Clear Widget
**Learning:** When using Ratatui to render UI elements over existing content (such as an error message overlay), rendering just the `Paragraph` and `Block` often causes the background text to bleed through the popup, rendering it unreadable.
**Action:** Always render a `ratatui::widgets::Clear` widget in the exact same `Rect` as the popup before rendering the popup's content. Also, use layout calculations (`Layout`, `Direction`, `Constraint`) to center popups so they do not take up the entire screen unnecessarily.
