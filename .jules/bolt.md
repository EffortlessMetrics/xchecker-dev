## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.
## 2024-05-23 - Static Regex Compilation with LazyLock
**Learning:** Recompiling `regex::Regex` inside high-traffic function calls (like error redaction or metadata extraction) creates a massive CPU overhead and memory allocation bottleneck.
**Action:** Statically cache compiled regexes using `std::sync::LazyLock`. Wrap `Regex::new().unwrap()` in `LazyLock` for frequently executed code paths to ensure they are compiled exactly once and reused, avoiding redundant runtime work.
