## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2024-05-23 - RegexSet is_match vs matches for Secret Detection
**Learning:** Using `RegexSet::matches(text)` and iterating over the set of matched indices to find actual string matches (using standard regex) involves O(N) operations in `scan_for_secrets`. If you only need a boolean result (like in `has_secrets`), directly using `RegexSet::is_match` avoids vector allocations, iterating over lines line-by-line, and mapping bounds to original string coordinates.
**Action:** Always prefer `RegexSet::is_match` when only testing for the presence of a pattern, rather than calling extraction functions that build contexts and allocate structs, as this reduces CPU usage during the first pass in redaction.
