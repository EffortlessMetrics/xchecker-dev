## 2025-04-20 - Prevent CPU exhaustion DoS (CWE-400) in Regex compilation
**Vulnerability:** Core functionality like error redaction repeatedly compiled regular expressions dynamically on every function call (e.g., `regex::Regex::new(...)`).
**Learning:** This introduces a CPU exhaustion bottleneck, exposing the application to Denial of Service (DoS) vulnerabilities if an attacker can trigger repeated error logs or extraction endpoints.
**Prevention:** Always statically cache compiled regular expressions using `std::sync::LazyLock` in high-traffic execution paths to prevent continuous recompilation overhead.
