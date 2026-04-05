
## 2024-04-05 - Prevent TOCTOU and DoS with Bounded File Reads
**Vulnerability:** Use of `std::fs::read_to_string` to read files is vulnerable to Time-Of-Check to Time-Of-Use (TOCTOU) race conditions (where a file changes type after checking but before reading) and Denial of Service (DoS) attacks (if a maliciously large file or infinite stream like `/dev/urandom` is read into memory).
**Learning:** Checking file metadata prior to calling `std::fs::read_to_string` does not secure the operation because the file is re-opened by the inner implementation.
**Prevention:** Always open the file descriptor first (`fs::File::open`), check `metadata()` on the opened file descriptor to verify it is not a directory, and enforce an absolute maximum read limit by taking only a specific amount (`file.take(MAX_LIMIT).read_to_string(&mut buf)`).
