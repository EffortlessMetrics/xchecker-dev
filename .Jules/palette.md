## 2026-07-02 - Added continuous highlight spacing to TUI List

**Learning:** When using Ratatui's `List` widget with a highlight symbol like `"▶ "`, selecting an item shifts its content right. Moving the selection causes the new item to shift and the old item to shift back, creating a jarring, "jumping" UX during keyboard navigation. Ratatui's `HighlightSpacing::Always` prevents this by preserving space for the symbol on unselected items.

**Action:** Whenever building or modifying a Ratatui `List` that uses a `highlight_symbol`, always configure `.highlight_spacing(HighlightSpacing::Always)` to ensure smooth keyboard navigation without horizontal layout jumping.
