## 2024-04-18 - TUI Help Footer Alignment and Discoverability
**Learning:** Center-aligning Ratatui Paragraphs causes their Block titles to incorrectly inherit center alignment unless explicitly overridden with left alignment. Additionally, failing to document context-dependent shortcuts (like Esc to quit vs. back) reduces discoverability and user confidence.
**Action:** Always use `Line::from(" Title ").alignment(Alignment::Left)` when center-aligning Ratatui widgets, and ensure all active event loop keys are explicitly exposed in context-aware help footers.
