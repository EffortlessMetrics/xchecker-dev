## 2026-01-21 - [CLI Output Formatting]
**Learning:** Naive title-casing algorithms fail on domain-specific acronyms (CLI, LLM, WSL), reducing perceived professionalism.
**Action:** Use an acronym-aware formatter or an allowlist approach for technical terms in UI text generation.

## 2026-01-22 - [Actionable Error States]
**Learning:** CLI tools often report "ISSUES DETECTED" without immediate guidance, forcing users to search docs.
**Action:** When a command fails health checks, immediately print a colored "Tip:" block suggesting the verbose flag and pointing to the specific troubleshooting documentation.

## 2026-01-23 - [Visual Hierarchy in CLI]

**Learning:** Dense text outputs in CLI tools are hard to scan. Users miss the overall status when it's just another line of text.
**Action:** Use emojis (e.g., 🩺) for immediate context recognition and horizontal separators (e.g., ─────) to visually distinguish the summary/result from the detailed logs.

## 2026-02-12 - [CLI Styling Consistency & Next Steps]
**Learning:** Inconsistent styling across CLI commands (init/clean vs doctor) degrades the "polished product" feel. Also, successfully initializing a project leaves users wondering "what now?".
**Action:** Standardize on Bold Cyan for headers, Green Bold for success/checkmarks, and Yellow Bold for warnings. Always include a "Next steps" section after initialization commands to guide the user to the next logical action (e.g., `xchecker spec`).
