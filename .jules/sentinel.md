## 2024-05-24 - Regex Compilation DoS in High-Traffic Paths
**Vulnerability:** Dynamically compiling regular expressions on every call in high-traffic logging functions creates a CPU exhaustion bottleneck / DoS vulnerability (CWE-400).
**Learning:** Even safe regex patterns cause significant CPU overhead if compiled repeatedly, which can be abused by flooding the application with errors.
**Prevention:** Statically cache compiled regexes using std::sync::LazyLock inside the function scope.
