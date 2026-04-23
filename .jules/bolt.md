## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.
## 2024-05-24 - Statically cache regular expressions
**Learning:** Dynamically compiling `Regex::new` inside high-traffic functions (like `summarize_requirements`) causes a significant CPU bottleneck as regex parsing overhead occurs on every call.
**Action:** Always wrap `Regex::new().unwrap()` in a `std::sync::LazyLock` for recurring expressions, declaring the static block inside the function scope to cache the compiled automaton without polluting the module namespace.
