## 2025-05-15 - Missing Redaction for Supported Providers
**Vulnerability:** OpenRouter and Resend API keys were not being redacted, despite OpenRouter being a supported LLM provider in the documentation.
**Learning:** Adding support for a new provider (in code or docs) must be accompanied by adding its secret patterns to the redaction system. The system uses an allowlist of regex patterns, so it doesn't automatically detect new key formats.
**Prevention:** When adding new providers or integrating with new services, always check `crates/xchecker-redaction/src/lib.rs` and add relevant patterns. Use `cargo run --features dev-tools --bin regenerate_secret_patterns_docs` to keep docs in sync.
