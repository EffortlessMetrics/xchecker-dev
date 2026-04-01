## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2024-05-23 - Static Regex Compilation inside Frequently Called Extraction Functions
**Learning:** `Regex::new` inside frequently called extraction functions like `summarize_requirements`, `summarize_design`, and `summarize_tasks` compiles multiple regexes synchronously on each call. This results in significant continuous CPU overhead and O(N) regex compilations per processing run.
**Action:** When working with repetitive regex calls, apply `std::sync::LazyLock` inside the function scope to statically cache the compiled `Regex`. This retains modular scope clarity without repeated initialization cost.
