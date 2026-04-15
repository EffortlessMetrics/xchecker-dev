## 2026-04-15 - [Prevent DoS via Regex Compilation]
**Vulnerability:** Dynamically compiling regular expressions inside high-traffic logging and redaction functions creates a CPU exhaustion bottleneck (CWE-400).
**Learning:** Frequent `Regex::new().unwrap()` calls in hot paths (like error redaction) introduce significant performance overhead and can be exploited for DoS if error generation is user-controllable.
**Prevention:** Statically cache compiled regexes using `std::sync::LazyLock` in the local function scope to avoid overhead and maintain encapsulation.
