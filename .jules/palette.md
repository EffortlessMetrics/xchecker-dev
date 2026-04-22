## 2024-04-22 - TUI Context Alignment
**Learning:** In Ratatui, center-aligning a Paragraph widget will cause its block title to inherit the center alignment unless the title is explicitly left-aligned using `Line::from(" Title ").alignment(Alignment::Left)`. Also, help footers must accurately reflect context-dependent keyboard shortcut behaviors (e.g. Esc quitting vs going back).
**Action:** When applying `.alignment()` to a Paragraph, always check if the surrounding block title needs explicit alignment overriding, and always verify actual event loop logic when documenting shortcuts.
