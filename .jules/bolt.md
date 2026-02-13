## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2024-05-24 - Efficient Redaction & CRLF Pitfalls
**Learning:** Calculating file offsets by iterating `lines()` and summing `len() + 1` is INCORRECT for CRLF files (as `lines()` strips both \r and \n, but +1 only accounts for \n). This caused content corruption when redacting secrets in Windows-style files.
**Action:** When modifying line-based content, prefer iterating lines once and rebuilding the string (O(N)) over repeated offset calculations and in-place replacements (O(M*N)). This is both safer (handles line endings via normalization) and significantly faster (~70x speedup for 5000 lines).
