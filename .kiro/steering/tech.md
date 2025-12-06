# xchecker Tech Stack

## Language & Build

- **Language**: Rust (Edition 2024)
- **Build System**: Cargo
- **Task Runner**: just (justfile)
- **Min Rust Version**: 1.91+

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing |
| `tokio` | Async runtime |
| `serde` / `serde_json` | Serialization |
| `blake3` | Cryptographic hashing |
| `anyhow` / `thiserror` | Error handling |
| `tracing` | Structured logging |
| `reqwest` | HTTP client |
| `ratatui` / `crossterm` | TUI interface |
| `globset` / `ignore` | File pattern matching |
| `fd-lock` | File locking |
| `proptest` | Property-based testing (dev) |

## Platform-Specific

- **Unix**: `libc`, `nix` (signals, process management)
- **Windows**: `winapi`, `windows` crate (job objects, process APIs)

## Common Commands

```bash
# Build
cargo build                    # Debug build
cargo build --release          # Release build

# Test (fast feedback ~30s)
cargo test --lib --bins

# Test (full suite)
cargo test --all-features

# Test with just
just test-fast                 # Quick tests
just test-full                 # Complete suite
just test-local                # No external deps
just test-stub                 # With claude-stub mock

# Quality checks
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings

# Run single test
cargo test test_name -- --nocapture

# Doc validation
cargo test --test test_doc_validation -- --test-threads=1
```

## Test Skip Markers

Tests requiring external resources use skip flags:
- `requires_claude_stub` - Needs compiled claude-stub binary
- `requires_real_claude` - Needs real Claude CLI + API key
- `requires_xchecker_binary` - Needs compiled xchecker binary

## LLM Configuration

Only `claude-cli` provider with `controlled` execution strategy is supported:

```toml
# .xchecker/config.toml
[llm]
provider = "claude-cli"
execution_strategy = "controlled"
```

## JSON Output Contracts

All JSON outputs use JCS (RFC 8785) for canonical emission:
- `schemas/receipt.v1.json` - Execution receipts
- `schemas/status.v1.json` - Spec status
- `schemas/doctor.v1.json` - Health checks

## Hard Invariants

These invariants must hold in production code:

| Invariant | Enforcement |
|-----------|-------------|
| No panics in production paths | Unwraps only in tests or with documented invariants |
| JSON v1 schemas are additive-only | Breaking changes require v2; v1 supported 6+ months |
| CI always enforces quality gates | `cargo fmt --check` + `cargo clippy -D warnings` |
| All file writes go through `atomic_write` | No direct `fs::write` in phase code |
| Secrets blocked before LLM invocation | Exit code 8, never proceed with secrets |
| LLM changes go through fixup pipeline | LLMs propose diffs, xchecker applies them |

## Change Playbooks

When modifying key subsystems, follow these checklists:

### Changing Fixup Engine (`src/fixup.rs`)
- [ ] Run fuzzy matching test suite: `cargo test fuzzy`
- [ ] Verify preview mode never mutates: `cargo test preview`
- [ ] Check atomic write paths preserved
- [ ] Update DEBUGGING_GUIDE.md if failure modes change

### Changing Phase Logic (`src/phases.rs`, `src/orchestrator/`)
- [ ] Run engine invariant tests: `cargo test --test test_engine_invariants`
- [ ] Verify artifact naming conventions preserved
- [ ] Update ORCHESTRATOR.md for behavioral changes
- [ ] Run full stub suite: `cargo test -- --include-ignored --skip requires_real_claude`

### Changing JSON Contracts (`src/receipt.rs`, `src/status.rs`, `src/doctor.rs`)
- [ ] Update schema files in `schemas/`
- [ ] Regenerate examples: `cargo test --test doc_validation -- generate`
- [ ] Run versioning tests: `cargo test schema_version`
- [ ] Update CONTRACTS.md if field semantics change
- [ ] Ensure new fields are optional (additive changes only)

### Changing CLI (`src/cli.rs`)
- [ ] Update README.md command table if commands change
- [ ] Update man pages or help text
- [ ] Verify exit codes remain consistent
- [ ] Run CLI integration tests

### Changing Configuration (`src/config.rs`)
- [ ] Update CONFIGURATION.md
- [ ] Add migration logic if breaking changes
- [ ] Update example configs in docs/
