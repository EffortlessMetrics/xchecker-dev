## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2024-05-24 - Zero-allocation string replacement with Cow
**Learning:** Using `regex::Regex::replace_all` returning `std::borrow::Cow` allows avoiding string cloning when no replacements occur or when chaining multiple regex replacements. By repeatedly rebinding a `Cow` variable, we only incur the allocation cost if a substitution actually happens.
**Action:** When performing multiple regex replacements on the same string in sequence, keep the intermediate result as a `Cow<str>` and use `.into_owned()` only at the very end to minimize heap allocations.
