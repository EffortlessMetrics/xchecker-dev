## 2024-03-14 - Fix TOCTOU vulnerability in file reading

**Vulnerability:** A Time-Of-Check to Time-Of-Use (TOCTOU) and memory exhaustion DoS vulnerability existed in `crates/xchecker-packet/src/builder.rs`. The code checked `fs::metadata()` to verify file size, and then separately called `fs::read_to_string()`. If a file grew significantly between the check and the read, it could bypass the size limit and cause memory exhaustion.

**Learning:** The vulnerability existed because the metadata check and the file read were separate operations. Even if the metadata check passed, there was no guarantee the file size would remain the same during the read operation.

**Prevention:** To prevent this, always open the file first with `fs::File::open()`, check the size using `file.metadata().len()`, and enforce a hard limit on the read operation using `std::io::Read::take(limit).read_to_string(&mut content)`.
