## 2026-05-03 - CPU Exhaustion / DoS via Dynamic Regex Compilation

**Vulnerability:** The functions `summarize_requirements`, `summarize_design`, and `summarize_tasks` dynamically compiled `regex::Regex` patterns on every call. In a high-traffic or bulk-processing scenario, this overhead could be weaponized to cause CPU exhaustion / Denial of Service (CWE-400).

**Learning:** `Regex::new` is an expensive operation in Rust. When complex regular expressions are compiled repeatedly within functions, it creates a significant performance bottleneck that can be exploited.

**Prevention:** Always statically cache compiled regular expressions using `std::sync::LazyLock` in frequently executed paths. Declare the static block inside the function scope to avoid module-level pollution while preventing recompilation.
