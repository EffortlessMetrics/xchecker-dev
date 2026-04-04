## 2025-02-12 - Prevent TOCTOU and DoS in file reading
**Vulnerability:** Widespread use of `std::fs::read_to_string` exposed the application to TOCTOU race conditions via symlink replacement, and DoS attacks via memory exhaustion from continuously growing files (like `/dev/zero`).
**Learning:** Standard library `read_to_string` is unsafe for untrusted inputs as it reads to EOF unconditionally and follows symlinks implicitly.
**Prevention:** Always use a secure read abstraction that opens the file descriptor first, checks `metadata` on the opened handle, and enforces a strict size bound using `std::io::Read::take(limit)` before allocating memory.
