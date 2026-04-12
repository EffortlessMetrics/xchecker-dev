## 2024-05-15 - [TUI Keyboard Discoverability]
**Learning:** Documenting full context-aware shortcuts (like Home/End) directly in the persistent UI footer is critical for command-line interfaces lacking mouse discoverability.
**Action:** Always verify application source bindings (e.g. event loops in rust ratatui) against user-facing documentation to identify hidden but fully functional hotkeys and elevate them into view using center alignment for visual priority.
