## 2024-05-24 - Improve TUI Help Footer
**Learning:** TUI help footers need clear visual hierarchy (center alignment, dark gray style) and explicit documentation of all available keyboard shortcuts (like Home/End) to improve discoverability. The Quit action should be correctly documented depending on the context (Esc vs q).
**Action:** Use `Alignment::Center` for `Paragraph` widgets in footers and explicitly list all active shortcuts in the text.
