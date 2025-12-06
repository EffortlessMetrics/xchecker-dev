# xchecker Project Structure

## Root Layout

```
├── src/                    # Main source code
├── tests/                  # Integration tests
├── docs/                   # Documentation
├── schemas/                # JSON schemas for contracts
├── examples/               # Example projects
├── scripts/                # Utility scripts
├── .xchecker/              # xchecker's own state directory
├── Cargo.toml              # Rust package manifest
├── justfile                # Task runner recipes
├── CLAUDE.md               # AI assistant guidance
└── CHANGELOG.md            # Version history
```

## Source Organization (`src/`)

### Core Modules

| Module | Purpose |
|--------|---------|
| `orchestrator/` | Core execution engine - phase lifecycle, LLM invocation, workflows |
| `phases.rs` | Phase implementations (Requirements, Design, Tasks, Review, Fixup) |
| `packet.rs` | Packet building with budget enforcement and file selection |
| `receipt.rs` | JSON receipt generation with cryptographic hashes |
| `fixup.rs` | Diff parsing and application with preview/apply modes |
| `redaction.rs` | Secret detection and redaction |
| `runner.rs` | Claude CLI execution (native/WSL) |
| `llm/` | LLM backend abstraction |

### Supporting Modules

| Module | Purpose |
|--------|---------|
| `cli.rs` | Command-line interface |
| `config.rs` | Configuration loading and validation |
| `error.rs` | Error types with user-friendly reporting |
| `exit_codes.rs` | Standardized exit codes |
| `lock.rs` | Concurrent execution prevention |
| `validation.rs` | Output validation |
| `doctor.rs` | Environment health checks |
| `artifact.rs` | Artifact file handling |
| `atomic_write.rs` | Safe file writes via staging |

## Artifact Naming Convention

- Requirements: `00-requirements.md`, `00-requirements.core.yaml`
- Design: `10-design.md`, `10-design.core.yaml`
- Tasks: `20-tasks.md`, `20-tasks.core.yaml`
- Review: `30-review.md`, `30-review.core.yaml`
- Fixup: `40-fixup.md`, `40-fixup.core.yaml`

## Integration Rule

**Outside `src/orchestrator/`, always use `OrchestratorHandle`** - the stable facade for CLI and external tools. Do not use `PhaseOrchestrator` directly except in white-box tests.

```rust
// Correct
let handle = OrchestratorHandle::new("my-spec")?;
handle.run_phase(PhaseId::Requirements).await?;

// Avoid (internal API)
let orch = PhaseOrchestrator::new("my-spec")?;
```

## Module Ownership & Forbidden Dependencies

| Module | Owns | Must Not Depend On |
|--------|------|-------------------|
| `packet` | Payload assembly, file selection, budget | LLM invocation, CLI |
| `fixup` | Diff parsing, hunk application | Direct `fs::write` (must use `atomic_write`) |
| `runner` | Process execution, timeout handling | Phase logic, packet building |
| `receipt` | Hash generation, JSON emission | LLM backends, file selection |
| `cli` | Argument parsing, output formatting | Business logic (delegate to orchestrator) |
| `orchestrator` | Phase lifecycle, workflow coordination | Direct secret patterns (delegate to `redaction`) |
| `redaction` | Secret detection, pattern matching | File I/O (operates on strings) |

## State Tree Contract

```
.xchecker/                          # XCHECKER_HOME (stable)
├── config.toml                     # Project configuration (stable)
├── lockfile.toml                   # Version pinning (stable)
└── specs/<spec-id>/                # Per-spec state (stable)
    ├── problem_statement.txt       # Original input (stable, injected into packets)
    ├── artifacts/                  # Phase outputs (stable)
    │   ├── 00-requirements.md      # Requirements markdown
    │   ├── 00-requirements.core.yaml
    │   ├── 10-design.md            # Design markdown
    │   ├── 10-design.core.yaml
    │   ├── 20-tasks.md             # Tasks markdown
    │   ├── 20-tasks.core.yaml
    │   ├── 30-review.md            # Review markdown
    │   ├── 30-review.core.yaml
    │   ├── 40-fixup.md             # Fixup markdown
    │   └── 40-fixup.core.yaml
    ├── receipts/                   # Execution audit trails (stable)
    │   └── <phase>-<timestamp>.json
    ├── context/                    # Packet previews (internal, debugging only)
    │   └── <phase>-packet.txt
    └── .partial/                   # Staging directory (internal, transient)
```

**Stability Guarantees:**
- **Stable**: Safe to rely on in scripts and integrations
- **Internal**: Implementation detail, may change without notice

## Test Organization (`tests/`)

- `test_*.rs` - Integration test files
- `doc_validation/` - Documentation validation tests
- `quarantine/` - Secret scanning test fixtures
- `*.toml` - Test configuration files
