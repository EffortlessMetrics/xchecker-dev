## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2025-03-13 - Regex Compilation Bottleneck
**Learning:** Compiling `regex::Regex` inside frequently called functions (like error redaction or path normalization) is a significant CPU bottleneck. The `regex::Regex` type is thread-safe and should be compiled once and reused.
**Action:** Always use `std::sync::LazyLock` to statically cache compiled regular expressions (`Regex::new().unwrap()`) instead of instantiating them on every call to avoid significant CPU overhead.
