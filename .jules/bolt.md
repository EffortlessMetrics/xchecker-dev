## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.
## 2024-05-24 - Cache Regex Compilation
**Learning:** Compiling complex regexes dynamically on every function call creates a significant CPU bottleneck. Statically caching them using std::sync::LazyLock provides a massive performance boost, but can break relative timing tests if the baseline 'miss' path becomes dramatically faster.
**Action:** When performing regex extraction inside frequently executed functions, always wrap Regex::new in a local static LazyLock. Be sure to increase TOLERANCE factors in integration tests that measure relative cache performance.
