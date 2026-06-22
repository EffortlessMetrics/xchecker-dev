## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2024-06-22 - Regex Compilation Bottleneck in Extraction Module
**Learning:** Functions that parse incoming texts utilizing multiple regular expressions, such as `summarize_requirements`, `summarize_design`, and `summarize_tasks` in the extraction module, incur massive overhead by invoking `Regex::new(r"...").unwrap()` repeatedly on each call. This pattern was identified across multiple locations inside `crates/xchecker-extraction/src/lib.rs`.
**Action:** Always extract static regular expressions using `std::sync::LazyLock`. This causes the pattern to be compiled only once on first access rather than on every single invocation, resulting in significant performance improvements for heavily repeated logic.
