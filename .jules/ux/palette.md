## 2026-01-21 - [CLI Output Formatting]
**Learning:** Naive title-casing algorithms fail on domain-specific acronyms (CLI, LLM, WSL), reducing perceived professionalism.
**Action:** Use an acronym-aware formatter or an allowlist approach for technical terms in UI text generation.

## 2026-01-22 - [Actionable Error States]
**Learning:** CLI tools often report "ISSUES DETECTED" without immediate guidance, forcing users to search docs.
**Action:** When a command fails health checks, immediately print a colored "Tip:" block suggesting the verbose flag and pointing to the specific troubleshooting documentation.

## 2026-01-23 - [Visual Hierarchy in CLI]

**Learning:** Dense text outputs in CLI tools are hard to scan. Users miss the overall status when it's just another line of text.
**Action:** Use emojis (e.g., 🩺) for immediate context recognition and horizontal separators (e.g., ─────) to visually distinguish the summary/result from the detailed logs.
## 2025-05-24 - [Discoverability of TUI Shortcuts]
**Learning:** Keyboard shortcuts like Home/End and context-dependent Esc behavior must be explicitly advertised in UI help footers to maintain discoverability and accessibility. Furthermore, center-aligning help text improves readability, but Ratatui block titles can incorrectly inherit this alignment unless explicitly left-aligned.
**Action:** Always document all supported key bindings in the TUI footer, and use `Line::from(" Title ").alignment(Alignment::Left)` on block titles when centering paragraph content.
