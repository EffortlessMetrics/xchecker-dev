## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2024-05-24 - Statically Cache Regexes in Extraction Functions
**Learning:** Compiling regular expressions repeatedly inside high-traffic metadata extraction functions creates unnecessary performance overhead and CPU exhaustion vulnerabilities. Additionally, instantiating a default struct followed by sequential mutations can cause `clippy` warnings.
**Action:** When working with frequent string pattern matching, cache compiled regular expressions using `std::sync::LazyLock` inside the function scope. Initialize summary structs directly with their fields instead of mutating a default instance to improve performance and avoid clippy warnings.
