## 2024-05-23 - [CLI Status Symbols]
**Learning:** Standardizing CLI status symbols (✓, ✗, ⚠) using helper functions (`styled_check`, `styled_cross`, `styled_warning`) ensures consistent visual feedback and automatically respects `NO_COLOR` settings across the application. Hardcoded symbols often lead to inconsistencies and lack of color support.
**Action:** Always use the standardized helper functions in `src/cli.rs` for status indicators instead of hardcoded strings.
