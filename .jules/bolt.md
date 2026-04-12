## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2024-05-24 - Static Caching of Regexes in High-Traffic Extraction Functions
**Learning:** In high-traffic extraction functions (like `summarize_requirements`), dynamically compiling regular expressions via `Regex::new` on every function call creates a significant CPU exhaustion bottleneck.
**Action:** Use `std::sync::LazyLock` to statically cache compiled regexes. Declare them inside the function scope to avoid module-level pollution while preventing the overhead of repeated regex compilation.
