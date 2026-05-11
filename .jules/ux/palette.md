## 2026-01-21 - [CLI Output Formatting]
**Learning:** Naive title-casing algorithms fail on domain-specific acronyms (CLI, LLM, WSL), reducing perceived professionalism.
**Action:** Use an acronym-aware formatter or an allowlist approach for technical terms in UI text generation.

## 2026-01-22 - [Actionable Error States]
**Learning:** CLI tools often report "ISSUES DETECTED" without immediate guidance, forcing users to search docs.
**Action:** When a command fails health checks, immediately print a colored "Tip:" block suggesting the verbose flag and pointing to the specific troubleshooting documentation.

## 2026-01-23 - [Visual Hierarchy in CLI]

**Learning:** Dense text outputs in CLI tools are hard to scan. Users miss the overall status when it's just another line of text.
**Action:** Use emojis (e.g., 🩺) for immediate context recognition and horizontal separators (e.g., ─────) to visually distinguish the summary/result from the detailed logs.

## 2024-05-11 - [TUI Keyboard Discoverability]
**Learning:** TUI applications often implement rich keyboard shortcuts (Home/End) and context-dependent behaviors (Esc to exit vs. back) in the event loop without advertising them, causing navigation frustration and accessibility issues. Furthermore, center-aligning Ratatui Paragraphs causes their Block titles to incorrectly inherit the center alignment.
**Action:** Always explicitly document all implemented shortcuts in TUI help footers, clearly distinguish context-dependent key behaviors, and apply `Line::from(" Title ").alignment(Alignment::Left)` to prevent block titles from inheriting paragraph content center alignment.
