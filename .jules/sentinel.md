## 2024-04-29 - TOCTOU Vulnerability in File Reading
**Vulnerability:** Time-of-check to time-of-use (TOCTOU) race condition when reading files for packet building. The code checked the file metadata size and then read the entire file via path. If the file grew concurrently between the metadata check and the read operation, it could lead to memory exhaustion (DoS).
**Learning:** `fs::read_to_string` does not enforce read limits based on prior metadata checks, exposing a TOCTOU gap in environments where files are modified dynamically.
**Prevention:** Always open a `fs::File` descriptor first, run `file.metadata()` on the opened descriptor to ensure consistent referencing, and use the `Read::take` adapter on the file descriptor to enforce a hard byte limit during the read operation itself.
