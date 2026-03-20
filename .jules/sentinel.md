
## 2025-05-01 - [Fix TOCTOU vulnerability in file reading]
**Vulnerability:** A Time-Of-Check to Time-Of-Use (TOCTOU) vulnerability where `fs::metadata` was checked before opening the file using `fs::read_to_string`, which could lead to memory exhaustion DoS if the file grows between the check and the read.
**Learning:** Directly using `fs::read_to_string` after `fs::metadata` allows race conditions because the file state can change in the interim.
**Prevention:** Always use `fs::File::open` first, then call `file.metadata()`. Additionally, use `std::io::Read::take(limit)` instead of unbounded reading to string to protect against concurrent file size growth.
