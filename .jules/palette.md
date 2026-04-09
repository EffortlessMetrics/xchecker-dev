## 2024-04-09 - [TUI Keyboard Shortcut Discoverability]
**Learning:** In terminal user interfaces (TUIs), hidden keyboard shortcuts (like Home/End for navigation) lead to poor discoverability. Additionally, the main view "Esc" shortcut behavior to quit was undocumented. Footer help text needs clear visual hierarchy (like center alignment) to separate it from content.
**Action:** Always explicitly document all implemented keyboard shortcuts in the help footer and use `Alignment::Center` and `Color::DarkGray` to establish clear visual hierarchy for help text.
