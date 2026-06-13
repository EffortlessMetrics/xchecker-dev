## 2026-01-21 - [CLI Output Formatting]
**Learning:** Naive title-casing algorithms fail on domain-specific acronyms (CLI, LLM, WSL), reducing perceived professionalism.
**Action:** Use an acronym-aware formatter or an allowlist approach for technical terms in UI text generation.

## 2026-01-22 - [Actionable Error States]
**Learning:** CLI tools often report "ISSUES DETECTED" without immediate guidance, forcing users to search docs.
**Action:** When a command fails health checks, immediately print a colored "Tip:" block suggesting the verbose flag and pointing to the specific troubleshooting documentation.

## 2026-01-23 - [Visual Hierarchy in CLI]

**Learning:** Dense text outputs in CLI tools are hard to scan. Users miss the overall status when it's just another line of text.
**Action:** Use emojis (e.g., 🩺) for immediate context recognition and horizontal separators (e.g., ─────) to visually distinguish the summary/result from the detailed logs.
## 2025-02-18 - TUI Help Footer Discoverability and Alignment
**Learning:** TUI help footers must advertise all available keyboard shortcuts (like Home/End and Esc) implemented in the event loop to ensure proper keyboard accessibility and discoverability. Additionally, when center-aligning Ratatui Paragraph text, the block title must explicitly be left-aligned (`Line::from(" Title ").alignment(Alignment::Left)`) to prevent it from inheriting the content's center alignment.
**Action:** Always cross-reference the TUI event loop for implemented shortcuts against the help footer, and explicitly enforce alignment on block titles when applying alignment to parent widgets.
