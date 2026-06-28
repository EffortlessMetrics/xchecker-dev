## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.
## 2025-01-20 - Regex compilation in loops
**Learning:** Compiling regex patterns (e.g., using `Regex::new(...)`) within functions that are called frequently or inside loops creates a major performance bottleneck due to repeatedly constructing the DFA/NFA.
**Action:** When using regex in Rust >= 1.80, always cache the compiled `Regex` by wrapping it in `std::sync::LazyLock` as a static variable.
