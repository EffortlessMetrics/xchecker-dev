## 2025-04-13 - [ReDoS / CPU Exhaustion in Regex Compilation]
**Vulnerability:** Dynamically compiling regular expressions on every function call in `xchecker-extraction`.
**Learning:** `Regex::new().unwrap()` in high-traffic functions causes significant CPU exhaustion, leading to a Denial of Service vulnerability (CWE-400), especially during repeated metadata extraction calls.
**Prevention:** Always statically cache compiled regular expressions using `std::sync::LazyLock`. Wrap `Regex::new().unwrap()` in `LazyLock` for frequently executed code paths within the function scope.
