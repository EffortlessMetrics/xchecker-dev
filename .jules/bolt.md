## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.
## 2025-01-01 - Cache Regex Compilation with LazyLock
**Learning:** Recompiling regular expressions on every invocation in hot paths (like metadata extraction, error redaction, and diff parsing) creates severe CPU bottlenecks and potential DoS vulnerabilities.
**Action:** Always statically cache compiled `regex::Regex` patterns using `std::sync::LazyLock`. Declare the `static` block within the function scope to prevent module-level pollution and maintain encapsulation.
