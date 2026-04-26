
## 2024-05-24 - DoS Vulnerability via Regex Recompilation
**Vulnerability:** Dynamically compiling regular expressions on every call in high-traffic functions like error redaction creates a CPU exhaustion bottleneck (CWE-400).
**Learning:** Recompiling complex regexes dynamically during every error log operation can allow an attacker to trigger excessive CPU usage by generating many errors.
**Prevention:** Always statically cache compiled regular expressions using `std::sync::LazyLock` inside the function scope for frequently executed code paths to prevent DoS.
