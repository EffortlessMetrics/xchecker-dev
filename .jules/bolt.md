## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2024-10-30 - Cache Regex compilation in extraction module
**Learning:** The `xchecker-extraction` module repeatedly compiles `Regex` patterns inside hot extraction functions (like `summarize_requirements`), causing unnecessary CPU overhead. Rust >= 1.80 provides `std::sync::LazyLock` which is perfect for caching these statically without external dependencies like `once_cell`.
**Action:** Always use `std::sync::LazyLock` to pre-compile and cache regex patterns at the module level when they are used repeatedly.
