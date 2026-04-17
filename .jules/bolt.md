## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.
## 2024-05-23 - Dynamic Regex Compilation in High-Traffic Functions
**Learning:** Compiling regexes dynamically inside high-traffic functions (like metadata extraction) causes a significant CPU exhaustion bottleneck (DoS vulnerability).
**Action:** Always statically cache compiled regexes using `std::sync::LazyLock` to prevent compilation overhead, and declare them inside the function scope to avoid module-level pollution.
