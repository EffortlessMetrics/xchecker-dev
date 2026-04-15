## 2024-05-20 - TUI Block Title Alignment
**Learning:** When center-aligning Ratatui `Paragraph` widgets, the block title incorrectly inherits the content's center alignment, which causes awkward visual hierarchy in TUI footers.
**Action:** Always apply `Line::from(" Title ").alignment(Alignment::Left)` to block titles when center-aligning TUI component content.
