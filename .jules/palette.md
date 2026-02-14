## 2026-02-17 - TUI Footer Styling
**Learning:** `ratatui` widgets like `Paragraph` accept `Line::from(vec![Span])` for rich styling, which is superior to plain strings for key bindings (e.g., cyan/bold for keys, dark gray for descriptions).
**Action:** Always use structured Spans for TUI help text to improve scannability.
