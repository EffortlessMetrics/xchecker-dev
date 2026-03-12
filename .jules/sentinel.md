## 2024-05-24 - TOCTOU and Memory Exhaustion Prevention
**Vulnerability:** File sizes were checked using `fs::metadata` followed by `fs::read_to_string`, creating a Time-Of-Check to Time-Of-Use (TOCTOU) vulnerability where a file could be swapped or grown between check and read, leading to potential Out-Of-Memory (OOM) DoS.
**Learning:** Checking file metadata separately from reading opens a race condition window in concurrent or untrusted environments. `fs::read_to_string` loads the entire file into memory regardless of its actual size during reading.
**Prevention:** Open the file first with `fs::File::open`, check `file.metadata().len()`, and use `std::io::Read::take(limit).read_to_string(&mut content)` to enforce an absolute memory limit during the read operation itself.
