## 2024-04-25 - Context-Aware TUI Help Footers
**Learning:** Ratatui Paragraph widgets inherit text alignment to their block titles by default, causing UI shifts. Context-dependent shortcuts like Esc must be explicitly documented in the current view's help footer.
**Action:** Explicitly set block title alignment using Line::from(" Title ").alignment(Alignment::Left) when center-aligning paragraph content, and update footer hints dynamically based on UI state.
