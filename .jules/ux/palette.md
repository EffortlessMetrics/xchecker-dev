## 2026-01-21 - [CLI Output Formatting]
**Learning:** Naive title-casing algorithms fail on domain-specific acronyms (CLI, LLM, WSL), reducing perceived professionalism.
**Action:** Use an acronym-aware formatter or an allowlist approach for technical terms in UI text generation.

## 2026-01-22 - [Actionable Error States]
**Learning:** CLI tools often report "ISSUES DETECTED" without immediate guidance, forcing users to search docs.
**Action:** When a command fails health checks, immediately print a colored "Tip:" block suggesting the verbose flag and pointing to the specific troubleshooting documentation.

## 2026-01-23 - [Visual Hierarchy in CLI]

**Learning:** Dense text outputs in CLI tools are hard to scan. Users miss the overall status when it's just another line of text.
**Action:** Use emojis (e.g., 🩺) for immediate context recognition and horizontal separators (e.g., ─────) to visually distinguish the summary/result from the detailed logs.
## 2024-06-11 - Make TUI footer help shortcuts explicitly clear and horizontally aligned
**Learning:** TUI applications often implement keyboard shortcuts like `Home` / `End` or `Esc` which are not surfaced in the minimal keyboard footer, making them completely undiscoverable for users unfamiliar with standard shell tooling conventions. Furthermore, unaligned simple text footers can appear poorly presented.
**Action:** When working on Ratatui TUIs, always check `event::read` for handled key codes, add them to footer strings (e.g. `Home/End: Top/Bottom`), and use `Alignment::Center` alongside `title(Line::from("...").alignment(Alignment::Left))` to correctly style standard blocks.
