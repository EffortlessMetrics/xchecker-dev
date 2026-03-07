## 2025-03-07 - Center Alignment for Secondary TUI Elements
**Learning:** Secondary TUI empty states (like "No spec selected" in details) and global help footers should be visually distinct from primary, actionable lists. Left-aligning them causes them to blend in with primary content flow.
**Action:** Use `ratatui::layout::Alignment::Center` and `Color::DarkGray` for secondary empty states and help footers to establish clear visual hierarchy and reduce cognitive load.
