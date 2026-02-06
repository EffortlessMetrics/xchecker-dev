## 2024-05-23 - TUI Navigation Standards
**Learning:** In TUI list views, users expect a full suite of navigation keys beyond just arrows/j/k. Missing PageUp/PageDown breaks flow for long lists.
**Action:** For all future TUI list components, implement Home, End, PageUp (step=10), and PageDown (step=10) by default.
