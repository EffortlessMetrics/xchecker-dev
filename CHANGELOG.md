# Changelog

All notable changes to xchecker will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.0] - 2026-01-18

### Added

- **Multi-Provider Support**: Documentation and CLI updates for Gemini, OpenRouter, and Anthropic support (V14 availability).
- **Packet Construction**: New system with priority-based file selection and configurable size limits.
- **Receipt System**: Implemented JCS (RFC 8785) serialization and BLAKE3 hashing for execution audit trails.
- **Configuration Model**: Comprehensive configuration model with validation, hooks support, and unified `ConfigSource` attribution.
- **Fixup Validation**: Added validation for fixup targets and pending fixups reporting.
- **Unified Phase Execution**: Introduced `execute_phase_core()` for consistent phase execution.
- **Developer Guide**: Added `CLAUDE.md` for AI agent guidance and project overview.
- **Engine Tests**: Added comprehensive engine invariant tests (Suite B3.7-B3.14).
- **Claude CLI Improvements**: Cross-platform runner with automatic detection, output format configuration, and max turns support.
- **NDJSON Support**: Implemented NDJSON parsing functionality for streaming responses.
- **Security Boundaries**: Unified security boundaries and centralized pending fixups handling.

### Changed

- **Documentation**: Overhauled `README.md`, `ORCHESTRATOR.md`, `CONFIGURATION.md`, and `LLM_PROVIDERS.md`.
- **Refactoring**: Streamlined `xchecker-utils` and configuration source handling.
- **Dependencies**: Updated security-critical dependencies and added `libc` for Unix support.
- **WSL Runner**: Improved WSL runner test assertions and validation logic.

### Fixed

- **CI/Tests**: Resolved integration test failures with `claude-stub`.
- **Configuration**: Fixed inconsistency in `ConfigSource` naming (standardized to `Default`).
- **Path Validation**: Enhanced artifact path validation and canonicalization.

## [1.0.0] - 2025-12-05

First stable release of xchecker with complete spec generation workflow support.

### Highlights

- **Spec Pipeline with Receipts**: Complete multi-phase workflow (Requirements → Design → Tasks → Review → Fixup) with cryptographic audit trails
- **Strict Validation Mode**: Configurable validation for low-quality LLM output with `strict_validation` flag
- **Problem Statement Persistence**: Original input preserved and injected into packets and prompts automatically
- **Safe Fixup Engine**: Fuzzy matching with explicit failure modes (`FuzzyMatchFailed` with actionable suggestions)
- **Schema Versioning**: JSON v1 contracts protected by property tests and versioning guards
- **Gateable JSON Output**: CI-ready `--json` output with documented gate patterns (smoke and strict modes)

### Core Features

- **Phase-Based Workflow**: Requirements -> Design -> Tasks -> Review -> Fixup -> Final
  - Structured artifacts (Markdown + YAML) per phase
  - Phase dependencies enforced by orchestrator
  - Resume from any completed phase
  - Atomic artifact promotion via `.partial/` staging

- **Runner System**: Cross-platform process execution
  - Native mode: Linux, macOS, Windows
  - WSL mode: Automatic path translation
  - Auto mode: Native first, WSL fallback
  - Timeout enforcement with graceful termination

- **Packet Builder**: Deterministic payload assembly
  - Priority-based file selection
  - Configurable size limits (default: 64KB, 1200 lines)
  - Exit code 7 on overflow

- **Secret Redaction**: Pre-invocation security
  - GitHub PAT, AWS keys, Slack tokens, Bearer tokens
  - Custom patterns via CLI flags
  - Exit code 8 on detection

- **Fixup Engine**: Safe file modification
  - Path validation and canonicalization
  - Preview mode by default
  - Atomic writes with backup

- **Lock Manager**: Concurrent execution prevention
  - Stale detection via PID/TTL
  - Drift tracking for reproducibility

- **JSON Contracts (v1)**: Versioned schemas
  - Receipt, Status, Doctor schemas with `schema_version` field
  - JCS (RFC 8785) canonical emission with `emitted_at` timestamps
  - BLAKE3 hashing for artifact verification
  - `error_kind` and `error_reason` fields for structured error reporting in receipts

### CLI

All commands support `--json` output and `--verbose` logging.

| Command | Description |
|---------|-------------|
| `spec <id>` | Generate spec through requirements |
| `resume <id> --phase <phase>` | Resume from phase |
| `status <id>` | Display spec status |
| `clean <id>` | Remove artifacts |
| `doctor` | Health checks |
| `init <id>` | Initialize spec |
| `benchmark` | Performance tests |

### Exit Codes

xchecker uses standardized exit codes for different failure modes:

| Code | Name | Description |
|------|------|-------------|
| `0` | SUCCESS | Operation completed successfully |
| `2` | CLI_ARGS | Invalid or missing command-line arguments |
| `7` | PACKET_OVERFLOW | Input packet exceeded size limits (default: 64KB, 1200 lines) |
| `8` | SECRET_DETECTED | Redaction system detected potential secrets in packet |
| `9` | LOCK_HELD | Another process is already working on the same spec |
| `10` | PHASE_TIMEOUT | Phase execution exceeded configured timeout (default: 600s) |
| `70` | CLAUDE_FAILURE | Underlying Claude CLI invocation failed |

### Configuration

Hierarchical system: CLI flags > config file > defaults

```toml
[defaults]
model = "haiku"
phase_timeout = 600

[selectors]
include = ["src/**/*.rs"]
exclude = ["target/**"]

[llm]
provider = "claude-cli"
execution_strategy = "controlled"
```

### Platform Support

| Platform | Status |
|----------|--------|
| Linux | Full support |
| macOS | Full support |
| Windows | Native + WSL |

### Performance

- Empty run: 16ms (target: 5000ms)
- Packetization (100 files): 10ms (target: 200ms)
- JCS emission: <50ms

## Schema Versioning Policy

- **v1 stability**: No breaking changes to v1 schemas
- **Additive only**: New optional fields may be added
- **6-month support**: After v2 release, v1 supported for 6+ months
- **JCS emission**: All JSON uses RFC 8785 canonical format

See [CONTRACTS.md](docs/CONTRACTS.md) for details.
