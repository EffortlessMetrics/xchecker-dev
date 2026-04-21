## 2024-05-24 - Prevent DoS from Dynamic Regex Compilation
**Vulnerability:** Dynamic compilation of regular expressions on every function call in metadata extraction paths.
**Learning:** Compiling regexes at runtime in high-traffic functions can lead to CPU exhaustion bottlenecks (CWE-400).
**Prevention:** Always statically cache compiled regexes using `std::sync::LazyLock` in frequently executed code paths to prevent DoS risks.
