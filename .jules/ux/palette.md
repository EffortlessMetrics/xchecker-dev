## 2026-01-21 - [CLI Output Formatting]
**Learning:** Naive title-casing algorithms fail on domain-specific acronyms (CLI, LLM, WSL), reducing perceived professionalism.
**Action:** Use an acronym-aware formatter or an allowlist approach for technical terms in UI text generation.

## 2026-01-22 - [Actionable Error States]
**Learning:** CLI tools often report "ISSUES DETECTED" without immediate guidance, forcing users to search docs.
**Action:** When a command fails health checks, immediately print a colored "Tip:" block suggesting the verbose flag and pointing to the specific troubleshooting documentation.

## 2026-01-23 - [Visual Hierarchy in CLI]

**Learning:** Dense text outputs in CLI tools are hard to scan. Users miss the overall status when it's just another line of text.
**Action:** Use emojis (e.g., 🩺) for immediate context recognition and horizontal separators (e.g., ─────) to visually distinguish the summary/result from the detailed logs.

## 2024-05-09 - Add Home/End keyboard shortcut hints to TUI footer
**Learning:** TUI interfaces often lack discoverability for advanced keyboard navigation shortcuts (like Home/End for jumping to the start/end of lists). While the shortcuts are implemented in the event loop, they need to be visibly advertised in the UI's help footer to maintain accessibility and usability for keyboard-reliant users. In context-dependent states (like detailed views), the Esc key behavior changes, which must also be accurately reflected (e.g., `Esc/q: Quit` on the main view versus `Esc: Back` on detailed views).
**Action:** Always verify that any interactive keyboard shortcuts implemented in the TUI event loop are explicitly surfaced in the application's help footers, ensuring the help text contextually matches the active view's behavior. Center-align the footer block title appropriately to maintain visual polish.
