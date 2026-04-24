## 2024-04-24 - Dynamic Regex Compilation in Redaction Logic
**Vulnerability:** Regex compiled inside high-traffic error redaction functions.
**Learning:** This introduces a DoS vulnerability (CWE-400) via CPU exhaustion when repeatedly logging errors.
**Prevention:** Always statically cache regex instances using `std::sync::LazyLock` for recurring invocations.
