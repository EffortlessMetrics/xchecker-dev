## 2024-05-18 - CWE-400 CPU Exhaustion / DoS in Regex compilation
**Vulnerability:** Dynamically compiling regular expressions inside frequently called extraction functions created a CPU exhaustion bottleneck / DoS vulnerability (CWE-400).
**Learning:** Parsing and allocating regexes is expensive and doing it repeatedly in high-traffic functions provides an attack vector for resource exhaustion.
**Prevention:** Always statically cache compiled regular expressions using `std::sync::LazyLock` in frequently executed code paths.
