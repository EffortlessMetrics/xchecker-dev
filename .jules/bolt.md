## 2024-05-22 - [Optimized File Reading]
**Learning:** In high-throughput file processing (like packet assembly), using `fs::metadata(path)` followed by `fs::read_to_string(path)` introduces a TOCTOU race condition and redundant syscalls (resolving the path twice).
**Action:** Use `fs::File::open(path)` to get a handle, then `file.metadata()` (fstat) and `file.read_to_string()` to read content. This is faster and safer. Also pre-allocate buffers using `String::with_capacity(metadata.len())`.
