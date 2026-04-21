## 2024-04-21 - TUI Help Footer Consistency and Alignment
**Learning:** Ratatui paragraph alignment can cascade unexpectedly to block titles. Help footers must also accurately reflect all active event loop keybindings (like Esc quitting on the main view) to prevent user confusion.
**Action:** Always verify help footer text against actual event loop logic and use `Line::from(" Title ").alignment(Alignment::Left)` to protect block titles when center-aligning content.
