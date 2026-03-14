## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2025-03-13 - Regex Compilation Bottleneck
**Learning:** Compiling `regex::Regex` inside frequently called functions (like error redaction or path normalization) is a significant CPU bottleneck. The `regex::Regex` type is thread-safe and should be compiled once and reused.
**Action:** Always use `std::sync::LazyLock` to statically cache compiled regular expressions (`Regex::new().unwrap()`) instead of instantiating them on every call to avoid significant CPU overhead.

## 2025-03-14 - CI Flakiness in Process tests
**Learning:** `sleep` can be interrupted by signals, and `kill(pid, 0)` checks can erroneously return true for zombie processes in slow CI environments.
**Action:** When validating signal termination in integration tests, use an infinite loop `trap '' TERM; while true; do sleep 1; done` to ignore signals robustly, and prefer checking termination explicitly on the child object via `child.try_wait().unwrap().is_some()` rather than external OS checks.
