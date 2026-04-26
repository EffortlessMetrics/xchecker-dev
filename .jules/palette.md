## 2024-04-26 - TUI Alignment Inheritance
**Learning:** In Ratatui, applying `Alignment::Center` to a Paragraph widget incorrectly centers its Block's title as well unless explicitly overridden.
**Action:** Always apply `Line::from(" Title ").alignment(Alignment::Left)` to the block title when center-aligning its content paragraph.
