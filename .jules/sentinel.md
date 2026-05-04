## 2025-03-05 - Prevent CPU Exhaustion (DoS) in Error Redaction via Regex Compilation Caching
**Vulnerability:** Dynamically compiling regular expressions on every call in the high-traffic `xchecker-error-redaction` crate created a CPU exhaustion bottleneck (DoS vulnerability / CWE-400).
**Learning:** Even utility functions intended for security (like error redaction) can become attack vectors if expensive operations like regex compilation occur on every invocation.
**Prevention:** Always statically cache compiled regular expressions using `std::sync::LazyLock` in frequently executed paths to eliminate redundant compilation overhead.
