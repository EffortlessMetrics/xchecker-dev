## 2026-01-28 - Centralized Security Exclusions
**Vulnerability:** Inconsistent enforcement of security exclusions if patterns are defined in multiple places.
**Learning:** `crates/xchecker-config/src/config/selectors.rs` acts as the single source of truth for mandatory security exclusions (`ALWAYS_EXCLUDE_PATTERNS`). These are enforced by default in config AND hard-coded in the engine for defense-in-depth.
**Prevention:** Always add new sensitive file patterns to `ALWAYS_EXCLUDE_PATTERNS` to ensure they are blocked everywhere.
