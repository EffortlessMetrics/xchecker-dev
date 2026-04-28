## 2024-05-23 - Prevent DoS by statically caching dynamically compiled Regexes
**Vulnerability:** CPU exhaustion / DoS vulnerability (CWE-400) caused by dynamically compiling regular expressions on every call in high-traffic functions (metadata extraction, error logging, diff parsing).
**Learning:** Regex::new().unwrap() was being called repeatedly on high-traffic execution paths, which creates a significant compilation overhead bottleneck.
**Prevention:** Always statically cache compiled regexes using std::sync::LazyLock. Wrap Regex::new().unwrap() in LazyLock and declare the static block inside the function scope to avoid module-level pollution.
