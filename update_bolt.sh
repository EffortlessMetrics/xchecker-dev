cat << 'INNER_EOF' >> .jules/bolt.md
## 2024-05-23 - PacketBuilder SelectedFile Instantiation and SecretRedactor string redaction
**Learning:**
1. In `process_candidate_file` within `crates/xchecker-packet/src/builder.rs`, `SelectedFile` was instantiated early, which required cloning the file's raw content (`content.clone()`) since `content` was also passed into the redact string routine later on. Moving instantiation of `SelectedFile` to the end of the cache/redaction logic allows it to take ownership of the final strings without requiring an intermediate clone for strings that could be several KBs.
2. `SecretRedactor::redact_string` replaced substrings and forcibly called `.to_string()` on every iteration even if no substitutions occurred, triggering unneeded allocations. Replacing with `std::borrow::Cow` allows passing slices unmolested unless matches actually occur.
**Action:**
1. Ensure intermediate structs representing transient state in data processing pipelines take ownership of terminal string values at the *end* of the pipeline rather than at the beginning, minimizing cloning overhead.
2. In high-frequency text processing blocks (like `redact_string`), use `Cow<'a, str>` to avert reallocation until mutation physically happens.
INNER_EOF
