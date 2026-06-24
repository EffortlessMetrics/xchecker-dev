## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.
## 2024-05-24 - Cache Regex Compilation in Extraction Module
**Learning:** Compiling regexes inside functions that are called frequently causes a significant CPU bottleneck due to repeated compilation. Using `std::sync::LazyLock` to cache compiled regexes as statics eliminates this overhead.
**Action:** Always cache compiled regexes using `LazyLock` when defining them inside frequently called functions.
