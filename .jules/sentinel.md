## 2025-04-02 - [HIGH] Fix TOCTOU vulnerability in file reading
**Vulnerability:** File metadata was checked before opening the file, leaving a window for a race condition.
**Learning:** Using `fs::read_to_string` after `fs::metadata` is unsafe as the file could be modified or swapped in between.
**Prevention:** Always open the file first with `fs::File::open` and then use `file.metadata()` to check the size and other properties. Use `take()` to limit read size to prevent memory exhaustion.
