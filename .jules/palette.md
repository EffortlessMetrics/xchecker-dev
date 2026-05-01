## 2026-05-01 - Center Aligning Ratatui Titles
**Learning:** Ratatui's `Paragraph` block titles inherit the main widget's alignment. If the `Paragraph` text is center-aligned but the block title should be left-aligned (e.g., in a help footer), `Line::from(" Title ").alignment(Alignment::Left)` must be explicitly applied to the block title.
**Action:** Use `alignment(Alignment::Left)` on block titles explicitly when changing a block's alignment to `Center` to avoid visual regressions in the UI framing.
