## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.
## 2025-05-23 - Static Regex Caching via LazyLock
**Learning:** Compiling complex regexes dynamically on every function call in high-traffic paths like metadata extraction creates a CPU bottleneck.
**Action:** Use `std::sync::LazyLock` inside function scopes to statically cache compiled regexes. This eliminates redundant parsing overhead without polluting the module namespace.
