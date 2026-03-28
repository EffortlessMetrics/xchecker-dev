
## 2025-05-15 - Add missing secret patterns to redaction
**Vulnerability:** Found missing secret patterns for OpenRouter API keys, Discord bot tokens, Square access tokens, and Slack webhooks in the default redaction engine (`crates/xchecker-redaction/src/lib.rs`). This could lead to sensitive API keys and tokens being leaked in logs or error messages.
**Learning:** Hardcoding token recognition regexes often falls behind as new token formats are released or heavily utilized by applications using xchecker. This resulted in inadequate redaction capabilities for newer provider keys.
**Prevention:** Regularly audit the secret patterns defined in `SecretPatternDef` lists against popular token format lists to ensure coverage for modern, commonly-used services.
