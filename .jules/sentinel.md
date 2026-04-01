## 2024-05-30 - Fix TOCTOU vulnerability in fixup application
**Vulnerability:** A Time-of-Check to Time-of-Use (TOCTOU) vulnerability where `apply_single_diff_atomic` used `fs::read_to_string` to read target files. If the file is replaced with a large file or a pseudofile (e.g., in `/proc`), it could cause memory exhaustion (DoS) or silent truncation.
**Learning:** `fs::read_to_string` has no bounding logic, making it unsuitable for reading files in security-sensitive contexts where file paths might be manipulated concurrently.
**Prevention:** Use `fs::File::open` to get a file handle, validate `metadata.is_dir()`, and bound the read using `std::io::Read::take(MAX_SIZE)` to prevent DoS via growing files.
