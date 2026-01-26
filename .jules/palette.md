## 2026-01-26 - TUI Empty States
**Learning:** Ratatui `List` widgets render as blank space when empty, providing no affordance or guidance to the user.
**Action:** Explicitly check for empty collections and render a `Paragraph` with actionable instructions (e.g., command to run) instead of an empty list widget.
