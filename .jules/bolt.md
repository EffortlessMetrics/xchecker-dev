## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2026-03-26 - Regex Replacement Chain Optimization
**Learning:** When chaining multiple regex replacements (e.g., `regex::Regex::replace_all`) in Rust, calling `.to_string()` repeatedly creates significant heap allocation overhead, even when no replacements are made.
**Action:** Minimize heap allocations by maintaining the intermediate result as a `std::borrow::Cow<str>`. Re-assign it only when the replacement returns `Cow::Owned` (e.g., `if let Cow::Owned(s) = regex.replace_all(...)`), and call `.into_owned()` only at the end.
