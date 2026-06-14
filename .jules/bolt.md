## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.
## 2024-06-14 - Regex Compilation Overhead in Extraction Functions
**Learning:** Recompiling static Regex patterns (e.g. `Regex::new(r"...").unwrap()`) within frequently called extraction functions introduces significant unnecessary overhead. In performance-critical areas, this cost dominates string matching time.
**Action:** Always use `std::sync::LazyLock` to statically compile and cache static Regex patterns globally so they are compiled exactly once, drastically reducing CPU cycles in hot loops.
