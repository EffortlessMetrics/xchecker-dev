## 2024-05-24 - DoS Vulnerability via Regex Compilation
**Vulnerability:** Dynamically compiling regular expressions on every call in high-traffic functions like error logging creates a CPU exhaustion bottleneck / DoS vulnerability (CWE-400).
**Learning:** The application was compiling 7 complex regexes every time an error was redacted.
**Prevention:** Always wrap `Regex::new().unwrap()` in `std::sync::LazyLock` for frequently executed code paths. Declare the `static` block inside the function scope to avoid module-level pollution and maintain encapsulated refactoring boundaries.
