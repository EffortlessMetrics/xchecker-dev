## 2024-05-15 - TOCTOU and DoS in file reading
**Vulnerability:** Unbounded file reading and TOCTOU
**Learning:** fs::read_to_string and fs::metadata+File::read_to_string is vulnerable to size mismatch and TOCTOU.
**Prevention:** Use `File::take` with a static limit.
