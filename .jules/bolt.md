## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2024-05-24 - Avoid Line-Based Iteration for Redaction
**Learning:** Redacting secrets by iterating lines and replacing the entire line for each match (O(M*N)) is exponentially slow for large files with many secrets. More importantly, processing matches in reverse order while replacing full lines caused earlier matches on the same line to be overwritten by later matches (using original line content).
**Action:** Use absolute byte offsets for replacements. Pre-calculate line start offsets once (O(N)), then map match positions to absolute ranges (O(1)) and replace only the secret range. This is 25x faster and correctly handles multiple secrets per line.
