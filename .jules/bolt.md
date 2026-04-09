## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2024-05-23 - Static Regex Compilation and Cow in Text Processing
**Learning:** In functions called frequently on every log or path (like `redact_error_message_for_logging`), defining regexes inline causes them to be recompiled into state machines repeatedly, creating a massive CPU bottleneck. Additionally, unconditionally calling `to_string()` and reassigning `String`s when chaining regex replacements wastes heap allocations when strings aren't mutated.
**Action:** Always statically cache compiled regular expressions using `std::sync::LazyLock` in hot loops/functions. Chain string mutations using `std::borrow::Cow` to eliminate allocations when no match/replacement is actually needed.
