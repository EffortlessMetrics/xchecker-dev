## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2024-05-24 - Static Regex Compilation with LazyLock
**Learning:** Dynamically compiling `Regex::new(r"pattern").unwrap()` on every function invocation creates a significant CPU bottleneck and potential DoS vulnerability (CWE-400), especially in frequently called extraction or parsing methods.
**Action:** Always statically cache compiled regular expressions using `std::sync::LazyLock`. Wrap `Regex::new().unwrap()` in a `static` block inside the function scope to maintain encapsulation while eliminating the recompilation overhead on hot paths.
