## 2025-05-24 - [TUI Navigation Patterns]
**Learning:** For TUI lists where viewport height is dynamic or hard to access in the update loop, implementing PageUp/PageDown with a fixed step size (e.g., 10 items) is a highly effective and low-complexity alternative to true page-based scrolling.
**Action:** Always implement fixed-step paging for any scrollable TUI list to support power users and large datasets.
