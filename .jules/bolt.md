## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2024-05-24 - Static caching of Regex compilation
**Learning:** In metadata extraction modules, dynamically compiling regular expressions on every function call introduces a significant CPU bottleneck due to repeated work. This also poses a potential DoS vulnerability (CWE-400) when running extraction repeatedly on multiple documents.
**Action:** Statically cache compiled regexes in high-traffic functions using `std::sync::LazyLock` wrapped around `Regex::new().unwrap()`. Declare the static variables directly inside the function scope to maintain encapsulation and keep refactoring boundaries clean.
