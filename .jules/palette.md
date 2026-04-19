## 2024-04-19 - TUI Help Footer Consistency
**Learning:** Help footers in TUI applications must accurately reflect all available keyboard interactions (like Home/End) and center alignment applies correctly when block titles have custom alignments.
**Action:** Center-align the TUI footer text, add `Home/End` to the documentation, and style it with `Color::DarkGray`.
## 2024-04-19 - Persona Boundaries vs CI Failures
**Learning:** CI failures in backend components (like `test_unix_process_termination`) should not be fixed by the Palette persona, whose boundaries explicitly forbid changing backend logic or performance code. Automated prompts must be evaluated against these strict boundaries.
**Action:** Revert or ignore CI failures outside the persona scope, and explicitly document when an issue is left untouched due to operational boundaries.
