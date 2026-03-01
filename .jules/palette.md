## 2024-05-24 - Secondary TUI Empty States
**Learning:** Secondary TUI empty states (e.g., 'No spec selected' in the details panel) should be visually distinct from actionable primary empty lists to clarify hierarchy and focus.
**Action:** Use `ratatui::widgets::Paragraph` with `Alignment::Center` and `Color::DarkGray` text for secondary empty states.
