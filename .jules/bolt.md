## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2024-06-16 - Inline Regex compilation bottleneck
**Learning:** Repeatedly compiling regexes inside functions that are called frequently causes a massive performance hit (e.g. 10k compilations took 300+ seconds in testing).
**Action:** When a function uses constant Regex patterns, always declare them as static variables. In Rust 1.80+, use `std::sync::LazyLock` to compile them once at runtime.
