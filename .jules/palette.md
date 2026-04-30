## 2024-04-30 - Center Alignment Inheritance in Ratatui
**Learning:** In Ratatui, when center-aligning the content of a Paragraph widget (like a help footer), the block's title will incorrectly inherit this center alignment by default, causing awkward visual layout.
**Action:** Always explicitly apply `Line::from(" Title ").alignment(Alignment::Left)` to the block title to preserve correct title positioning when centering paragraph content.
