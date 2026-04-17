## 2024-05-24 - DoS Vulnerability via Repeated Regex Compilation
**Vulnerability:** Regex compiled on every function invocation in high-traffic functions (error logging, metadata extraction).
**Learning:** Dynamically compiling regular expressions on every call creates a CPU exhaustion bottleneck / DoS vulnerability (CWE-400).
**Prevention:** statically cache compiled regexes using `std::sync::LazyLock`.
