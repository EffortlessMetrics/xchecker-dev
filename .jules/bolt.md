## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2024-05-23 - Static Compilation of Regexes using LazyLock
**Learning:** Dynamic instantiation of regular expressions within functions (e.g., `Regex::new(r"...").unwrap()`) incurs significant compilation overhead, particularly in functions that are called frequently like parsing or metadata extraction routines.
**Action:** When a regex pattern is statically known and used inside a frequently invoked function, use `std::sync::LazyLock` to compile the regex exactly once lazily. This provides a substantial performance boost by eliminating repetitive runtime regex compilation.
