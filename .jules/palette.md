## 2024-05-23 - Center-align TUI help footer and add key hints
**Learning:** Help footers in TUIs can easily go unnoticed if left-aligned or blended with the main content. Furthermore, implicit keyboard shortcuts (like Home/End) are invisible to users unless explicitly documented, reducing the discoverability of power-user features.
**Action:** Always center-align TUI help footers (e.g., using `Alignment::Center`) to establish a clear visual hierarchy. Additionally, verify that all actively supported event loop keys are explicitly listed in the help text to improve discoverability.
