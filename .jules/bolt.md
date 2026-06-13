## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2024-05-28 - Regex compilation bottleneck
**Learning:** Creating `regex::Regex` instances within functions called frequently (like metadata extraction) is a major performance bottleneck due to the cost of compiling the regular expressions.
**Action:** Statically cache compiled regexes using `std::sync::LazyLock` for regexes used in frequent operations. This ensures compilation happens only once and significantly speeds up execution.
