## 2026-01-21 - [CLI Output Formatting]
**Learning:** Naive title-casing algorithms fail on domain-specific acronyms (CLI, LLM, WSL), reducing perceived professionalism.
**Action:** Use an acronym-aware formatter or an allowlist approach for technical terms in UI text generation.

## 2026-01-22 - [Actionable Error States]
**Learning:** CLI tools often report "ISSUES DETECTED" without immediate guidance, forcing users to search docs.
**Action:** When a command fails health checks, immediately print a colored "Tip:" block suggesting the verbose flag and pointing to the specific troubleshooting documentation.

## 2026-01-23 - [Visual Hierarchy in CLI]

**Learning:** Dense text outputs in CLI tools are hard to scan. Users miss the overall status when it's just another line of text.
**Action:** Use emojis (e.g., 🩺) for immediate context recognition and horizontal separators (e.g., ─────) to visually distinguish the summary/result from the detailed logs.
## 2026-06-23 - [TUI Shortcut Discoverability]
**Learning:** Terminal User Interfaces (TUIs) often implement hidden keyboard shortcuts (like Home/End for fast navigation) that are not discoverable without reading the source code, leading to reduced accessibility and usability for keyboard-centric users.
**Action:** Always ensure that any navigational keyboard shortcuts implemented in the event loop are explicitly advertised in the UI's help footers to maintain discoverability and accessibility.
