# xchecker

[![Crates.io](https://img.shields.io/crates/v/xchecker.svg)](https://crates.io/crates/xchecker)
[![License](https://img.shields.io/crates/l/xchecker.svg)](https://github.com/EffortlessMetrics/xchecker#license)

A Rust CLI tool for orchestrating spec generation workflows with Claude AI. Transform rough feature ideas into structured requirements, designs, and implementation tasks through an automated multi-phase pipeline.

## Features

- **Multi-Phase Pipeline**: Requirements -> Design -> Tasks -> Review -> Fixup -> Final
- **Versioned JSON Contracts**: Stable schemas for receipts, status, and health checks
- **Security First**: Automatic secret detection and redaction
- **Cross-Platform**: Linux, macOS, Windows with WSL support
- **Reproducibility**: Lockfile system for version pinning and drift detection

## Installation

```bash
# From crates.io
cargo install xchecker

# From source
git clone https://github.com/EffortlessMetrics/xchecker.git
cd xchecker && cargo install --path .
```

**Requirements**: Rust 1.70+, [Claude CLI](https://claude.ai/download) installed and authenticated.

## Quick Start

```bash
# Check your environment
xchecker doctor

# Create a spec from stdin
echo "Build a REST API for user management" | xchecker spec my-feature

# Check status
xchecker status my-feature

# Resume from a specific phase
xchecker resume my-feature --phase design

# Apply code changes
xchecker resume my-feature --phase fixup --apply-fixups
```

## Commands

| Command | Description |
|---------|-------------|
| `xchecker spec <id>` | Generate a new spec through the requirements phase |
| `xchecker resume <id> --phase <phase>` | Resume execution from a specific phase |
| `xchecker status <id>` | Display spec status and configuration |
| `xchecker clean <id>` | Clean up spec artifacts and receipts |
| `xchecker doctor` | Run environment health checks |
| `xchecker init <id>` | Initialize a new spec with optional lockfile |
| `xchecker benchmark` | Run performance benchmarks |

### Common Options

```bash
--dry-run              # Preview without making Claude calls
--json                 # Output as JSON
--force                # Override stale locks
--apply-fixups         # Apply file changes (default is preview)
--verbose              # Enable structured logging
```

## Configuration

xchecker uses a hierarchical configuration system: CLI flags > config file > defaults.

```toml
# .xchecker/config.toml
[defaults]
model = "haiku"
phase_timeout = 600

[selectors]
include = ["src/**/*.rs", "docs/**/*.md"]
exclude = ["target/**", ".git/**"]
```

See [Configuration Guide](docs/CONFIGURATION.md) for all options.

## Exit Codes

| Code | Name | Description |
|------|------|-------------|
| 0 | SUCCESS | Completed successfully |
| 7 | PACKET_OVERFLOW | Packet size exceeded |
| 8 | SECRET_DETECTED | Secret found in content |
| 9 | LOCK_HELD | Lock already held |
| 10 | PHASE_TIMEOUT | Phase timed out |
| 70 | CLAUDE_FAILURE | Claude CLI failed |

## Known Limitations & Guarantees

### Guarantees

| What | Guarantee |
|------|-----------|
| **v1 JSON schemas** | Additive-only changes; no breaking changes without version bump |
| **Canonical emission** | JCS (RFC 8785) for reproducible, diff-friendly JSON |
| **Atomic writes** | All artifact writes via staging directory; no partial files |
| **Secret scanning** | Always runs before LLM invocation; blocks on detection |
| **Failure modes** | Structured exit codes; no silent failures or "best effort" modes |
| **State directory layout** | `artifacts/`, `receipts/`, `problem_statement.txt` are stable API |

### Limitations

| Area | Limitation |
|------|------------|
| **LLM provider** | Claude CLI only; no direct API or other providers |
| **Execution strategy** | Controlled only; LLMs propose diffs, xchecker applies |
| **Fixup engine** | Context-based fuzzy matching works for contiguous context; fails on ambiguous patterns, large shifts, or context split by deletions |
| **Diff complexity** | Best with small, focused changes; large refactors may fail fuzzy matching |
| **Windows** | Requires WSL or native Claude CLI; some path edge cases |

For fixup limitations and workarounds, see the [Debugging Guide](docs/DEBUGGING_GUIDE.md#fuzzy-matching--what-works-what-doesnt).

## Documentation

| Document | Description |
|----------|-------------|
| [Configuration](docs/CONFIGURATION.md) | Full configuration reference |
| [Testing](docs/TESTING.md) | Test lanes and profiles |
| [Orchestrator](docs/ORCHESTRATOR.md) | Core engine architecture |
| [Contracts](docs/CONTRACTS.md) | JSON schema versioning |
| [Doctor](docs/DOCTOR.md) | Health check details |
| [LLM Providers](docs/LLM_PROVIDERS.md) | Provider configuration |
| [Troubleshooting](docs/TROUBLESHOOTING.md) | Common issues and fixes |

### Walkthroughs

- [20-Minute Quickstart](docs/WALKTHROUGH_20_MINUTES.md) - Get running fast
- [Spec to PR](docs/WALKTHROUGH_SPEC_TO_PR.md) - Complete workflow guide

### Operations

- [Debugging Guide](docs/DEBUGGING_GUIDE.md) - Diagnostics and debugging workflows
- [Workspace Guide](docs/WORKSPACE_GUIDE.md) - Workspace setup and TUI usage
- [CI Profiles](docs/CI_PROFILES.md) - CI gate configuration

## Development

```bash
# Run fast tests (~30s)
cargo test --lib --bins

# Run full suite
cargo test

# Check formatting and lints
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

See [Testing Guide](docs/TESTING.md) for test profiles and CI configuration.

## License

MIT OR Apache-2.0

## Contributing

1. All tests pass: `cargo test`
2. Code formatted: `cargo fmt`
3. No clippy warnings: `cargo clippy -- -D warnings`
4. Documentation updated for new features

See [CHANGELOG.md](CHANGELOG.md) for version history.
