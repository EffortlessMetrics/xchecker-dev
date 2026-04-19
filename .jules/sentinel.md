## 2024-04-19 - Regex Compilation DoS Vulnerability (CWE-400)
**Vulnerability:** Regular expressions were dynamically compiled on every call in the `xchecker-extraction` metadata extraction functions (`summarize_requirements`, `summarize_design`, `summarize_tasks`).
**Learning:** Compiling regexes in high-traffic functions creates a CPU exhaustion bottleneck. Even in simple metadata extraction functions, repeated `Regex::new` calls significantly impact performance and create DoS risks.
**Prevention:** Always statically cache compiled regexes using `std::sync::LazyLock`. Declare the `static` block inside the function scope to avoid module-level pollution and maintain encapsulated refactoring boundaries.
