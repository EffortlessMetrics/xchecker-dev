## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2025-02-18 - Regex compilation bottleneck in extraction module
**Learning:** Dynamically compiling regular expressions via `Regex::new` in frequently executed metadata extraction functions causes significant CPU exhaustion and performance bottlenecks.
**Action:** Always wrap `Regex::new().unwrap()` in `std::sync::LazyLock` within the function scope for static caching, ensuring efficient execution without module-level pollution.
