
## 2024-04-23 - Fix TOCTOU vulnerability in builder.rs
**Vulnerability:** fs::metadata followed by fs::read_to_string is vulnerable to TOCTOU and file growth attacks
**Learning:** Using File::open first, then file.metadata(), then file.take() avoids TOCTOU problems.
**Prevention:** Always use a file handle and take() when reading files with size limits.
