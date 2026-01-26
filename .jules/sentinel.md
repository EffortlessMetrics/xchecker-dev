## 2024-05-22 - File Read TOCTOU Protection
**Vulnerability:** Checking file size via `metadata()` before reading with `read_to_string()` is vulnerable to race conditions (TOCTOU). If the file grows after the check, `read_to_string` will read until EOF, potentially exhausting memory.
**Learning:** `metadata().len()` is only a snapshot. `read_to_string` trusts the file stream.
**Prevention:** Always use `.take(limit)` when reading potentially untrusted files into memory, even if a metadata check was performed. `file.take(limit).read_to_string(&mut content)` enforces the limit at the read level.
