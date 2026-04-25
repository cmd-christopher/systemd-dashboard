## 2025-04-17 - Text bleed-through in Ratatui Overlays
**Learning:** In Ratatui TUI applications, popup overlays (like error dialogues) drawn over existing text will allow the background text to bleed through unless the target area is explicitly cleared first. Long messages also require explicit `.wrap(Wrap { trim: true })` configurations to ensure they remain readable inside constrained block boundaries.
**Action:** When rendering popups or overlay components in Ratatui, always call `f.render_widget(Clear, area);` before rendering the actual overlay widget, and ensure potentially long text has proper wrap configurations. Additionally, error dialogue titles should clearly instruct the user on how to dismiss them (e.g., "Press any key to dismiss").
## 2025-02-12 - Error View Modal Redesign
**Learning:** In terminal UIs with a split-pane layout, rendering error messages as a full-size override block obscures the background content, making the error feel disjointed from the action being taken. Using a centered layout to create a "modal" over the existing UI provides better context for the user.
**Action:** Use ratatui's Layout constraints with `Clear` widget to build centered overlay popups instead of full-screen error components.
## 2023-10-27 - Added Empty State for Timer List
**Learning:** Adding empty states to list views helps users understand how to start using the app.
**Action:** When a TUI list view can be empty, provide actionable instructions on how to populate it.
## 2025-04-25 - Essential affordance for scrollable text via Scrollbar
**Learning:** For long, scrollable text areas like logs or detailed configurations in TUI applications, users often lose context on their scroll position and the total content extent without visual cues. Adding a stateful `Scrollbar` component tied to the same layout constraints significantly improves navigation intuitiveness.
**Action:** When implementing scrollable views using ratatui's `scroll` feature on Paragraph or List widgets, always render an accompanying `Scrollbar` bound to the parent area's `.inner()` margins to avoid border overlap and provide immediate visual feedback.
