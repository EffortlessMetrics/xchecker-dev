## 2026-01-25 - Packet Builder Double Hashing
**Learning:** `PacketBuilder` was hashing file content twice: once in `process_candidate` to populate `SelectedFile` and again in `process_file_with_cache` for cache key generation. Reusing the first hash avoids redundant work.
**Action:** When working with multi-stage processing pipelines, verify if intermediate results (like hashes) are available in the passed data structures before recomputing them.
