## 2026-01-23 - Enforced Sensitive File Exclusion
**Vulnerability:** Default packet construction could include sensitive files (like `.env`, `private.pem`) if user configuration overrides defaults or is too broad (`**/*`).
**Learning:** Security controls (exclusions) should be mandatory and non-overridable for high-risk patterns. Relying on default configuration structs is insufficient when users can replace them entirely.
**Prevention:** Implemented `ALWAYS_EXCLUDE_PATTERNS` in `ContentSelector` that are enforced in all constructors, ensuring a secure baseline regardless of user configuration.
