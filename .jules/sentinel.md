## 2024-05-20 - Regex Compilation Denial of Service (DoS)

**Vulnerability:** The codebase compiles regular expressions dynamically on every invocation inside high-traffic functions in `crates/xchecker-extraction/src/lib.rs` (e.g., `summarize_requirements`, `summarize_design`, `summarize_tasks`). This dynamic compilation consumes CPU, potentially causing a Denial of Service (DoS) under heavy load.

**Learning:** This is a CWE-400 (Uncontrolled Resource Consumption) vulnerability. The memory explicitly notes that `Regex::new` dynamically compiles regular expressions, which takes significant time. It states: "In high-traffic functions (like error logging or metadata extraction), dynamically compiling regular expressions on every call creates a CPU exhaustion bottleneck / DoS vulnerability (CWE-400). Prevent this by statically caching compiled regexes using `std::sync::LazyLock`. Always wrap `Regex::new().unwrap()` in `LazyLock` for frequently executed code paths, and declare the `static` block inside the function scope to avoid module-level pollution and maintain encapsulated refactoring boundaries."

**Prevention:** Use `std::sync::LazyLock` to statically cache compiled regular expressions. Do not compile regexes dynamically inside functions unless the regex pattern itself must be built dynamically at runtime.
