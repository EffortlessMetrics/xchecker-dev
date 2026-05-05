## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.
## 2025-03-05 - LazyLock for Regex in extraction utilities
**Learning:** Functions that provide utility metadata extraction by running regexes on markdown content shouldn't compile those regexes dynamically on every call, creating unnecessary CPU overhead and garbage allocation.
**Action:** Use `std::sync::LazyLock` placed inside function scope to statically compile utility regexes once upon first execution without polluting module namespaces.
