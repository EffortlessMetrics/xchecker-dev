## 2024-05-22 - [Inconsistent Color Handling in Utility Crates]
**Learning:** Utility crates (like `xchecker-utils`) may blindly apply ANSI codes using `crossterm` styles without checking `is_terminal` or `NO_COLOR`, leading to garbage output in logs/pipes. CLI crates often handle this, but shared logic in utils can be overlooked.
**Action:** Always implement a `use_color()` helper (checking `IsTerminal` and `NO_COLOR`) in utility crates that perform user-facing output, or pass a styling context from the CLI crate.
