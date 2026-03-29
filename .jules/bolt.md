## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2024-05-23 - Static Regex Compilation using LazyLock
**Learning:** Instantiating regular expressions with `Regex::new(...)` inside frequently called extraction functions like `summarize_requirements` results in significant overhead due to repeated compilation of the same pattern.
**Action:** Always wrap `Regex::new` in `std::sync::LazyLock` when defining constant regexes inside functions to ensure the pattern is compiled exactly once, minimizing CPU usage on repeated calls.
