## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2024-06-19 - Cache Regex compilations in inner loops
**Learning:** Instantiating `Regex::new()` repeatedly inside frequently called functions (such as redaction logic running per-log or extraction logic processing documents) creates significant CPU bottleneck due to compilation overhead. While `once_cell` or `lazy_static` were common previously, Rust 1.80's standard library now provides `std::sync::LazyLock` which handles this optimally and natively.
**Action:** When working on performance optimizations in Rust >= 1.80, always cache compiled regex patterns as `static` variables with `std::sync::LazyLock` when the patterns are fixed and used inside hot loops.
