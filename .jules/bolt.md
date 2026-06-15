## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.
## 2025-06-15 - Cache Regex Compilation with LazyLock
**Learning:** Recompiling regex patterns (like `Regex::new(r"...").unwrap()`) repeatedly inside functions executed frequently (like `summarize_requirements`, `summarize_design`, `summarize_tasks` in extraction pipelines) is a significant and unnecessary CPU overhead.
**Action:** Use `std::sync::LazyLock` combined with `static` variable definitions to compile regex patterns exactly once at runtime, ensuring faster subsequent matches without modifying function signatures.
