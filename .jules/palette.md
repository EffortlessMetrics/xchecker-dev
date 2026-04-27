## 2024-04-27 - [Center-aligned Text with Left-aligned Titles in Ratatui]
**Learning:** When center-aligning a `Paragraph` widget in Ratatui, the block title incorrectly inherits the center alignment by default. Furthermore, documenting available keyboard shortcuts correctly prevents user confusion.
**Action:** Apply `Line::from(" Title ").alignment(Alignment::Left)` to the block title to explicitly reset its alignment when the main content is centered.
