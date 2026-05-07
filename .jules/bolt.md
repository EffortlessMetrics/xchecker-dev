## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2024-05-24 - Dynamic Regex Compilation in Extraction Module
**Learning:** Dynamically compiling `Regex::new` in high-traffic extraction functions (like `summarize_requirements`, `summarize_design`, and `summarize_tasks`) incurs a large overhead and can create performance bottlenecks during parsing.
**Action:** Always wrap `Regex::new().unwrap()` in `std::sync::LazyLock` to statically cache the compiled regex for frequently executed code paths. Declare the `static` variable inside the function scope to maintain encapsulation and avoid module-level pollution.
