## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.
## 2024-05-23 - Static Regex Compilation with LazyLock
**Learning:** In metadata extraction pipelines (`xchecker-extraction`), compiling regular expressions dynamically via `Regex::new` on every function call creates unnecessary CPU overhead, especially when analyzing markdown artifacts recursively.
**Action:** Always wrap static/constant regular expressions in `std::sync::LazyLock` inside high-traffic utility functions. This avoids repeated regex compilation and significantly improves execution speed while preserving thread safety and localized encapsulation.
