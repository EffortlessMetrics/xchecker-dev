## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.
## 2024-05-23 - Static Regex Compilation
**Learning:** Compiling regular expressions via `Regex::new` inside frequently called functions (such as markdown parsing or summary extraction) introduces significant CPU overhead on every invocation.
**Action:** Always statically cache compiled regular expressions using `std::sync::LazyLock` inside the function scope to minimize compilation overhead and localize refactoring.
