## 2024-04-01 - TUI Help Footer Alignment and Discoverability
**Learning:** Left-aligned help footers in TUI applications are often overlooked by users scanning the interface, and missing shortcut documentation (like Home/End) leads to poor feature discoverability and reduced keyboard accessibility.
**Action:** Always center-align TUI help footers using `.alignment(Alignment::Center)` to establish clear visual hierarchy, and ensure all active event loop shortcuts are explicitly documented in the help text.
