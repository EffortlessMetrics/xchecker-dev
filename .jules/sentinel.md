## 2024-03-27 - [TOCTOU Vulnerability in File Reading]
**Vulnerability:** TOCTOU vulnerability in file reading in `process_candidate_file`
**Learning:** Checking file size before reading using `fs::metadata` and then using `fs::read_to_string` is vulnerable to TOCTOU.
**Prevention:** Open the file first, get metadata, and read with a bounded limit.
