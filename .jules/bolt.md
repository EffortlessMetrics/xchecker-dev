## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2026-03-13 - Cache Regex Compilation with LazyLock
**Learning:** Recompiling regular expressions inside frequently called functions (like `summarize_requirements`, `summarize_design`, `summarize_tasks` in `xchecker-extraction`) adds significant CPU overhead because `Regex::new` parses the pattern and builds an automaton every time.
**Action:** Use `std::sync::LazyLock` to statically cache compiled regular expressions so they are compiled only once during the application's lifetime, reducing CPU overhead on subsequent function calls.
