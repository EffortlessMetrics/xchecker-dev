## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2025-02-17 - Static Regex Caching with LazyLock
**Learning:** Recompiling complex regular expressions inside frequently called functions introduces significant unnecessary CPU overhead. `Regex::new` is an expensive operation.
**Action:** Always statically cache compiled regular expressions using `std::sync::LazyLock` inside the function scope to avoid repeated compilation overhead. Declaring the `static LazyLock` variables inside the function scope helps keep refactoring localized and minimizes module-level pollution.
