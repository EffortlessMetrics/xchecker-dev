## 2024-05-23 - RegexSet Pre-filtering for Secret Detection
**Learning:** Iterating through 40+ complex regex patterns for secret detection on every file is a significant CPU bottleneck. The `regex::RegexSet` type allows compiling multiple patterns into a single automaton that can check for *any* match in a single pass (roughly equivalent to `pattern1|pattern2|...`).
**Action:** When performing multi-pattern matching where the negative case (no match) is common, always use `RegexSet` to pre-filter content before running individual regexes for extraction/replacement. This reduced redaction overhead significantly.

## 2024-05-27 - Double Scanning in Hot Path
**Learning:** `process_candidate_file` was calling `redactor.has_secrets()` (which scans) and then `redactor.redact_content()` (which scans again). Since `has_secrets` enforces a strict gate, the second scan was redundant for clean files. Removing the second scan saves ~50% regex overhead per file.
**Action:** When using multi-step validation/processing pipelines, check if early steps already computed necessary information (like "no secrets found") to avoid re-computation in later steps.

## 2024-05-27 - Regex Replace vs Manual Loop
**Learning:** Attempted to optimize `redact_string` by manually collecting match ranges and reconstructing string once (O(N) allocation) vs `regex.replace_all` (multiple passes/allocations). Benchmark showed `replace_all` was ~10% faster for small/medium strings. `regex` crate's internal optimizations (likely COW usage and efficient scanning) beat naive manual reconstruction in Rust.
**Action:** Trust `regex::Regex::replace_all` for performance unless profiling proves otherwise on specific workloads. Complexity of manual range handling is high and performance gain is not guaranteed.
