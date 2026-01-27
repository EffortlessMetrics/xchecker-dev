## 2026-01-27 - Redundant Hashing in Packet Builder
**Learning:** Packet construction was hashing file content twice: once during selection (for verification/metadata) and again during cache processing (for cache keys). Reusing the pre-calculated hash saves significant CPU time (approx 6-8% in benchmarks).
**Action:** When implementing multi-stage data processing pipelines, always check if expensive properties (like hashes) computed in early stages can be passed down to later stages to avoid re-computation.
