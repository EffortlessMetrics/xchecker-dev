## 2026-03-21 - Distinct Visual Hierarchy for TUI Empty States and Footers
**Learning:** In terminal user interfaces, secondary empty states (like details panels without selection) and help footers need clear visual distinction from actionable primary lists to avoid confusing users about where focus and action lie.
**Action:** Use `Alignment::Center` and `Color::DarkGray` to style secondary non-actionable components, pushing them down the visual hierarchy.
