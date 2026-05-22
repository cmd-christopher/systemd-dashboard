## 2025-04-17 - Text bleed-through in Ratatui Overlays
**Learning:** In Ratatui TUI applications, popup overlays (like error dialogues) drawn over existing text will allow the background text to bleed through unless the target area is explicitly cleared first. Long messages also require explicit `.wrap(Wrap { trim: true })` configurations to ensure they remain readable inside constrained block boundaries.
**Action:** When rendering popups or overlay components in Ratatui, always call `f.render_widget(Clear, area);` before rendering the actual overlay widget, and ensure potentially long text has proper wrap configurations. Additionally, error dialogue titles should clearly instruct the user on how to dismiss them (e.g., "Press any key to dismiss").
## 2025-02-12 - Error View Modal Redesign
**Learning:** In terminal UIs with a split-pane layout, rendering error messages as a full-size override block obscures the background content, making the error feel disjointed from the action being taken. Using a centered layout to create a "modal" over the existing UI provides better context for the user.
**Action:** Use ratatui's Layout constraints with `Clear` widget to build centered overlay popups instead of full-screen error components.
## 2023-10-27 - Added Empty State for Timer List
**Learning:** Adding empty states to list views helps users understand how to start using the app.
**Action:** When a TUI list view can be empty, provide actionable instructions on how to populate it.
## 2025-05-15 - Added visual scrollbar to log and service file views
**Learning:** Terminal UIs showing scrollable content like logs or service files should provide a visual cue when the content overflows the view area. This acts as an affordance, helping users immediately understand that there is more content to explore and roughly how much.
**Action:** When creating text views with scrolling functionality, append a `Scrollbar` from Ratatui to the view. Make sure to render it after the main text widget, bound to the same `Rect`, to correctly position it.
## 2025-05-16 - Added visual scrollbar to list views
**Learning:** Terminal UIs showing list content like a table of items should provide a visual cue when the content overflows the view area. This acts as an affordance, helping users immediately understand that there is more content to explore and roughly how much.
**Action:** When creating list views with scrolling functionality, append a `Scrollbar` from Ratatui to the view. Make sure to render it after the main widget, bound to the same `Rect`, to correctly position it.
## 2025-05-17 - Styled visual pills for keybindings
**Learning:** Providing an unobtrusive, contextual keybinding footer in TUI apps using styled visual pills (separating the key command with a highlighted background from the action description) improves clarity, aesthetics, and user guidance dynamically based on the current view mode.
**Action:** When implementing keybindings footers, use separate styles for keys and descriptions to create visual pills, making it easier for users to quickly scan and understand available actions.
## 2026-05-02 - Textual focus indicators in TUI
**Learning:** Relying solely on color changes (like border colors) to indicate focus violates accessibility guidelines and can be hard to see. Adding a textual indicator like '▶' improves visibility.
**Action:** Always provide a non-color visual indicator for focus states in terminal interfaces.

## 2025-02-18 - Non-Color Affordances in TUI Navigation
**Learning:** TUI tab indicators (like the Detail Controls: Logs vs Service File) that rely solely on color and font weight (bold) to show the active state are inaccessible, similar to web elements.
**Action:** Added a `▶ ` textual symbol prefix to clearly denote the active tab option independently of terminal color support.
## 2025-05-18 - Consistent symbol indicators
**Learning:** Terminal UIs relying on text symbols should maintain consistency across views. Mixing indicator styles like `>> ` for selection and `▶ ` for focus creates a visually disjointed experience. Standardizing symbols (e.g., using `▶ ` universally) improves visual polish with minimal code changes.
**Action:** When adding or modifying selection or focus indicators, audit existing symbols in the application to ensure visual consistency.
## 2025-05-19 - Inconsistent 'rainbow' background effect with Modifier::REVERSED
**Learning:** Using `Modifier::REVERSED` as a row highlight style in TUI `Table` widgets swaps the foreground and background colors. If individual cells within a row have distinct foreground colors, this creates an inconsistent 'rainbow' background effect, which can be visually jarring and reduce readability.
**Action:** Avoid using `Modifier::REVERSED` for highlighting rows where cells have varying foreground colors. Instead, explicitly set a solid background color (e.g., `Style::default().bg(Color::DarkGray)`) to ensure a consistent and clean highlight state.
## 2025-05-18 - Consistent symbol indicators
**Learning:** Terminal UIs relying on text symbols should maintain consistency across views. Mixing indicator styles like `>> ` for selection and `▶ ` for focus creates a visually disjointed experience. Standardizing symbols (e.g., using `▶ ` universally) improves visual polish with minimal code changes.
**Action:** When adding or modifying selection or focus indicators, audit existing symbols in the application to ensure visual consistency.
## 2025-05-18 - Improve Table Row Selection Visibility
**Learning:** Adding a subtle bold modifier to the selected row in TUI tables helps visually distinguish the selected row without introducing inconsistent 'rainbow' background effects often caused by `Modifier::REVERSED` on multi-colored text.
**Action:** When styling selected table rows, prefer using a solid background color paired with `Modifier::BOLD` rather than reversed colors.

## 2025-05-20 - Styling Keybindings as Visual Pills
**Learning:** In a terminal UI, styling the key and its description together as a single visual element (a "pill") by bridging their background colors instead of using simple text separators significantly improves scannability and visual hierarchy in the footer.
**Action:** When creating keybinding footers, remove the background color from the parent container and instead apply background colors directly to the key and description parts. Use simple spacing between these combined elements instead of line separators like `│` to make them look like individual UI components.
## 2025-05-21 - Dynamic Action Labels for Toggle States
**Learning:** Generic action labels like "Toggle Timer" require users to mentally calculate the current state and the resulting action. Dynamically updating the label to explicit verbs (e.g., "Start Timer" or "Stop Timer") based on the selected item's current state significantly reduces cognitive load and improves confidence.
**Action:** When providing interactive toggle actions in a UI, dynamically update the action label text to reflect the exact outcome (e.g., "Start" vs "Stop", "Enable" vs "Disable") instead of using a generic "Toggle" description.

## 2026-05-16 - Graceful empty states in detail panes
**Learning:** When journalctl returns '-- No entries --' or a file is empty, displaying raw blank/system text in a TUI lacks polish. Providing centered, context-aware empty states gives users immediate understanding of why the view is blank.
**Action:** Always parse empty states or default system responses ('-- No entries --') and replace them with formatted, centered empty-state messages explaining the context.
## 2025-05-22 - Hide inapplicable contextual keybindings
**Learning:** In a TUI application, displaying keybinding options for actions that are not currently applicable (e.g., "Navigate" or "Toggle" in an empty list) creates a broken UX feel and confusing experience for users. Hiding these non-applicable options reduces cognitive load.
**Action:** When rendering footer keybindings dynamically based on the current view mode, explicitly check the data state (e.g., `is_empty()`) to only show keybindings for actions that the user can actually perform.
## 2025-05-22 - Contrast for highlighted states
**Learning:** Using `Color::DarkGray` for text inside a table creates a contrast issue when the row selection highlight is also `Color::DarkGray`.
**Action:** Always ensure that text colors within selectable rows have sufficient contrast against both the default background and the highlighted background.
## 2025-05-23 - Contextual visual cues for scrollability
**Learning:** Hiding scroll keybindings when a detail pane has no overflow text avoids confusing the user into thinking there is more hidden content to read.
**Action:** Always dynamically remove "Scroll" bindings from keybinding footers when max scroll is zero.
## 2025-05-24 - Actionable Empty States Call to Action
**Learning:** An empty state that simply states "No logs found" leaves the user at a dead end, even if there's a keybinding footer available. Directly embedding the specific action needed to resolve the empty state (like "Press [Space] to start the timer") directly inside the empty state message itself provides a much stronger, more discoverable affordance.
**Action:** When creating empty states, always consider if there is an immediate action the user can take to populate the view, and include a clear, actionable instruction (like mentioning the specific keybinding) in the empty state text.
