## 2024-05-22 - [Context-Aware TUI Help]
**Learning:** In terminal UIs, static help text creates visual noise. Dynamically hiding shortcuts for impossible actions (like navigation in an empty list) significantly reduces cognitive load.
**Action:** When designing TUI footers or help panels, always implement state checks to filter displayed shortcuts. Use distinct colors (e.g., Cyan for keys, DarkGray for descriptions) to make them skimmable.
