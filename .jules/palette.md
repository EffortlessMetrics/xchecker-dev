## 2024-03-17 - Center align empty states in TUIs
**Learning:** TUI help footers and secondary empty states (like details pane) look cleaner and have a better visual hierarchy when center-aligned and styled with DarkGray.
**Action:** Apply `.alignment(Alignment::Center)` and `.style(Style::default().fg(Color::DarkGray))` to `ratatui::widgets::Paragraph` for help texts and empty states.
