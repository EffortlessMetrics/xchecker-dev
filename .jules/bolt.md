## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2024-05-23 - Unnecessary Allocations with Cow::to_string()
**Learning:** Calling `.to_string()` on a `std::borrow::Cow` in Rust always allocates a new `String` because it routes through the `Display` trait implementation, completely defeating the purpose of using `Cow` to avoid allocations on the borrowed path.
**Action:** When working with `Cow` for string manipulation (like chaining regex replacements), always use `.into_owned()` at the end of the operation to cleanly consume the `Cow` and avoid unnecessary heap allocations, and conditionally re-assign the `Cow` variable only when the operation actually returns a `Cow::Owned`.
