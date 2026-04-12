## 2024-04-12 - [Security Context and Learning]
**Vulnerability:** TOCTOU when reading files, and possibility of memory exhaustion (DoS) attacks.
**Learning:** `std::fs::read_to_string` does not guard against these.
**Prevention:** Use `File::open` followed by `file.metadata()`, verify the file is not a directory, that it doesn't exceed 10MB in size, and bound the read using `file.take(MAX_SIZE).read_to_string(&mut content)`.
