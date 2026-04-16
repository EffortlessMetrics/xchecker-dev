## 2024-04-16 - Prevent DoS and TOCTOU in file reads
**Vulnerability:** Found `std::fs::read_to_string` being used on unvalidated file paths in `xchecker-utils/src/source.rs`. This can lead to DoS if a huge file is provided, as it will be loaded entirely into memory. There's also a potential for TOCTOU issues if the file changes between existence check and reading.
**Learning:** Utilities for reading file sources didn't have size limits in place, which is a common vulnerability when accepting arbitrary input file paths, allowing resource exhaustion.
**Prevention:** Use a wrapper like `secure_read_to_string` that utilizes `.take()` on the file handle to limit the maximum read size (e.g., to 10MB) to prevent OOM errors and consume the file safely.
