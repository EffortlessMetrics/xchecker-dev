## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2024-06-25 - Regex Compilation Caching
**Learning:** Recompiling the same regular expressions (using `Regex::new(...)`) on every function invocation is a massive CPU bottleneck in Rust, especially in frequently called parsing/extraction loops. While the `regex` crate is fast at matching, compilation is expensive.
**Action:** Always wrap static, unchanging regular expressions in `std::sync::LazyLock` (available in Rust >= 1.80) to ensure they are compiled exactly once and reused, significantly reducing overhead.
