## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## $(date +%Y-%m-%d) - Caching Regex Compilation
**Learning:** Compiling regular expressions via `Regex::new()` on every function call introduces unnecessary performance overhead, particularly when parsing and analyzing string contents. By leveraging `std::sync::LazyLock`, regular expressions can be compiled exactly once statically and reused across function invocations. This avoids redundant computational expense.
**Action:** When defining static or frequently used regex patterns in a Rust codebase, cache their compilation using `std::sync::LazyLock` rather than compiling them on-the-fly.
