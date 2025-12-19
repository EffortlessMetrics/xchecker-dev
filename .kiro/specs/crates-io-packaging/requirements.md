# Requirements Document

## Introduction

This spec covers the work to turn xchecker into a crates.io-native CLI that can be installed with `cargo install xchecker` while also exposing a stable library API (kernel) for embedding. The goal is to formalize the boundary between the library kernel and CLI layer, ensure proper packaging metadata, and maintain all existing invariants (security, receipts, atomic writes, JSON contracts).

**Current State:**
- ✅ Core runtime implementation complete
- ✅ CLI functional with all commands
- ✅ JSON contracts (receipts, status, doctor) stable
- ✅ CI and test infrastructure in place
- ⏳ Library API boundary not formalized
- ⏳ Cargo.toml metadata incomplete for crates.io
- ⏳ No embedding examples or documentation

**Goals:**
- Installable via `cargo install xchecker`
- Stable library API via `OrchestratorHandle` facade
- Thin CLI wrapper with all logic in library
- Proper crates.io metadata and documentation
- Preserved security invariants and JSON contracts

## Glossary

- **xchecker**: The Rust CLI tool for orchestrating spec generation workflows
- **Kernel**: The library-grade core containing domain logic, orchestration, and all non-CLI concerns
- **OrchestratorHandle**: The stable public facade for embedding xchecker functionality
- **PhaseId**: Enum identifying workflow phases (Requirements, Design, Tasks, Review, Fixup, Final)
- **EffectiveConfig**: Configuration with source attribution (cli/config/default/programmatic)
- **XcError**: The library-level error type with rich context
- **ExitCode**: Process-level exit code for CLI boundary
- **JCS**: JSON Canonicalization Scheme (RFC 8785) for deterministic JSON output
- **crates.io**: The Rust community's package registry
- **MSRV**: Minimum Supported Rust Version
- **Stable API**: Symbols re-exported at the `xchecker::` root level, covered by semver guarantees
- **Internal API**: Module paths accessible but marked `#[doc(hidden)]`, not covered by stability guarantees

## Requirements

### Requirement 1 (FR-LIB)

**User Story:** As a Rust developer, I want to embed xchecker's spec pipeline in my application, so that I can programmatically create specs and run phases without invoking the CLI.

#### Acceptance Criteria

1. WHEN a developer adds `xchecker` as a dependency THEN the system SHALL expose `OrchestratorHandle` as the primary public API for spec operations
2. WHEN a developer imports xchecker THEN the system SHALL provide access to `PhaseId`, `Config`, `XcError`, and `ExitCode` types at the crate root
3. WHEN `OrchestratorHandle::new(spec_id)` is called THEN the system SHALL create a handle for the specified spec using environment-based config discovery (XCHECKER_HOME, upward search)
4. WHEN `OrchestratorHandle::from_config(spec_id, config)` is called THEN the system SHALL create a handle using the explicitly provided configuration without probing global environment or filesystem
5. WHEN `handle.run_phase(PhaseId)` is called THEN the system SHALL execute the specified phase and return `Result<(), XcError>`
6. WHEN `handle.run_all()` is called THEN the system SHALL execute all phases in sequence and return `Result<(), XcError>`
7. WHEN `handle.run_phase` or `handle.run_all` are called THEN their behavior SHALL match the existing CLI semantics for the same operations (phase ordering, failure behavior, artifact promotion)
8. WHEN library code encounters an error THEN the system SHALL return `XcError` with rich context, not call `std::process::exit`
9. WHEN `OrchestratorHandle` is used THEN it SHALL use the same state directory rules as the CLI (XCHECKER_HOME, `.xchecker/`) unless constructed via `from_config` with an explicit state directory
10. WHEN library APIs log THEN they SHALL use the existing `tracing` infrastructure and SHALL NOT write directly to stdout/stderr; the CLI is responsible for configuring logging output

### Requirement 2 (FR-CLI)

**User Story:** As a CLI user, I want the xchecker binary to remain fully functional, so that existing scripts and workflows continue to work unchanged.

#### Acceptance Criteria

1. WHEN `xchecker` is invoked THEN the system SHALL support all existing subcommands: spec, resume, status, clean, doctor, init, benchmark, gate
2. WHEN CLI arguments are parsed THEN the system SHALL use clap for argument parsing in `cli.rs`, not in `main.rs`
3. WHEN the CLI encounters an error THEN `cli::run()` SHALL map the underlying `XcError` to the appropriate `ExitCode`, display a user-friendly message via `display_for_user()`, and return `Err(ExitCode)`
4. WHEN `main.rs` executes THEN the system SHALL only invoke `cli::run()`, and on `Err(code)` SHALL call `std::process::exit(code.as_i32())` without printing any additional output
5. WHEN CLI flags are provided THEN the system SHALL support all existing flags including `--verbose`, `--json`, `--force`, `--apply-fixups`, etc.
6. WHEN JSON output is requested THEN the system SHALL emit JCS-canonical JSON matching existing v1 schemas (see FR-CONTRACT)

### Requirement 3 (FR-PKG)

**User Story:** As a Rust developer, I want to install xchecker from crates.io, so that I can easily add it to my toolchain without building from source.

#### Acceptance Criteria

1. WHEN `cargo install xchecker` is executed THEN the system SHALL install the `xchecker` binary
2. WHEN the crate is published THEN Cargo.toml SHALL include: name, version, description, repository, homepage, license, readme, categories, and keywords
3. WHEN the crate is published THEN the system SHALL exclude internal directories (`.xchecker/`, `.kiro/`) from the package
4. WHEN `cargo publish --dry-run` is executed THEN the system SHALL succeed with no blocking errors
5. WHEN the crate is installed THEN the binary SHALL work on Linux, macOS, and Windows given Rust toolchain and Claude CLI configured
6. WHEN the crate is published THEN Cargo.toml SHALL declare `rust-version` (MSRV) and the project SHALL support that version for all 1.x releases
7. WHEN the crate is installed via `cargo install xchecker` THEN only the `xchecker` binary SHALL be installed; `claude-stub` is a dev/test utility and SHALL NOT be installed unless the `dev-tools` feature is explicitly enabled

### Requirement 4 (FR-API)

**User Story:** As a library consumer, I want a stable, documented API, so that I can rely on xchecker without breaking changes in 1.x releases.

#### Acceptance Criteria

1. The set of symbols re-exported at the `xchecker::` root (e.g., `OrchestratorHandle`, `PhaseId`, `Config`, `XcError`, `ExitCode`) SHALL define the stable public API for 1.x
2. Internal modules MAY be publicly accessible via module paths but SHALL be marked `#[doc(hidden)]` and are not covered by 1.x stability guarantees
3. WHEN the API is documented THEN each public type at the crate root SHALL have rustdoc documentation
4. WHEN the API evolves in 1.x THEN the system SHALL only add new functions/types, not remove or change existing signatures
5. WHEN `OrchestratorHandle` is used THEN it SHALL be the canonical facade for all spec operations outside the orchestrator module
6. WHEN `cargo test --doc` runs THEN all documented public API examples SHALL compile, with fragile ones marked `ignore`

### Requirement 5 (FR-ERR)

**User Story:** As a developer, I want consistent error handling between library and CLI, so that errors are informative and exit codes are predictable.

#### Acceptance Criteria

1. WHEN `XcError` is created THEN the system SHALL include error kind, context, and actionable suggestions
2. WHEN `XcError::to_exit_code()` is called THEN the system SHALL return the appropriate `ExitCode` matching existing mappings
3. WHEN `XcError::display_for_user()` is called THEN the system SHALL return a user-friendly error message
4. WHEN the CLI exits THEN the exit code SHALL match the value in any error receipt written for that invocation (when receipts are emitted)
5. WHEN library code returns an error THEN it SHALL be an `XcError`, not a panic or `std::process::exit`
6. WHEN library code encounters an unexpected condition THEN it SHALL return `XcError` (or propagate a typed error) instead of panicking in production paths, except where an internal invariant is explicitly documented

### Requirement 6 (FR-CFG)

**User Story:** As a library consumer, I want to configure xchecker programmatically, so that I can control behavior without environment variables or config files.

#### Acceptance Criteria

1. WHEN `Config::discover_from_env_and_fs()` is called THEN the system SHALL use the same discovery logic AND precedence rules as the CLI (XCHECKER_HOME, upward search, cli/config/default)
2. WHEN `Config` is constructed programmatically THEN the system SHALL allow setting all configuration options
3. WHEN `OrchestratorHandle::from_config()` is used THEN the system SHALL not probe global environment or filesystem for config
4. WHEN configuration is loaded THEN the system SHALL track source attribution (cli/config/default/programmatic)

### Requirement 7 (FR-SEC)

**User Story:** As a security engineer, I want the library API to maintain all security invariants, so that embedding xchecker doesn't introduce vulnerabilities.

#### Acceptance Criteria

##### Secret Redaction (FR-SEC-1)
1. WHEN the library API invokes LLMs THEN the system SHALL apply secret redaction before any external invocation
2. THE default redaction patterns SHALL cover at minimum: common cloud provider keys (AWS, GCP, Azure), generic API keys and OAuth tokens, database connection strings (Postgres, MySQL, SQL Server), SSH private keys and generic PEM secrets
3. WHEN callers need custom patterns THEN the system SHALL allow adding extra secret patterns and ignore patterns via config or programmatic API
4. THE documented secret pattern categories SHALL be explicitly listed in SECURITY.md

##### Atomic Writes (FR-SEC-2)
5. WHEN the library API writes spec artifacts THEN the system SHALL use atomic writes via the `atomic_write` module (temp → fsync → rename)
6. WHEN the library API writes receipts THEN the system SHALL use atomic writes via the `atomic_write` module
7. WHEN the library API writes status files THEN the system SHALL use atomic writes via the `atomic_write` module
8. WHEN the library API writes lock files THEN the system SHALL use atomic writes via the `atomic_write` module
9. THE system SHALL NOT use direct `fs::write` for any file in the xchecker state directory

##### Path Sandboxing (FR-SEC-3)
10. WHEN the library API handles user-provided paths THEN the system SHALL canonicalize paths before use
11. WHEN the library API handles paths THEN the system SHALL enforce a configured root (workspace or state dir)
12. WHEN a path contains `..` traversal components THEN the system SHALL reject the path with an error
13. WHEN a path is absolute and outside the configured root THEN the system SHALL reject the path with an error
14. WHEN symlinks or hardlinks are encountered THEN the system SHALL reject them unless explicitly enabled via configuration

##### Process Execution (FR-SEC-4)
15. WHEN the library API executes processes THEN the system SHALL use argv-style APIs only (Command::new + .arg/.args)
16. THE system SHALL NOT use shell string evaluation (sh -c, cmd /C) in production paths
17. WHEN the WSL runner constructs command lines THEN it SHALL use argv APIs with no string concatenation of unvalidated input
18. WHEN arguments cross trust boundaries THEN the system SHALL validate or normalize them before execution

##### Output Sanitization (FR-SEC-5)
19. WHEN the library API persists data THEN receipts, status, doctor outputs, and logs SHALL NOT contain raw secrets as defined by the default and configured patterns
20. WHEN debug facilities are used (e.g., `--debug-packet`) THEN they SHALL be opt-in and clearly documented as increasing exposure
21. THE system SHALL NOT include environment variables or raw packet content in receipts, status, or doctor outputs

##### Security Gate (FR-SEC-6)
22. WHEN the crate is published THEN there SHALL be no open, known critical security vulnerabilities in secret detection, path validation, or process execution
23. WHEN medium or low severity security issues remain THEN they SHALL be documented in SECURITY.md with mitigations

### Requirement 8 (FR-DOC)

**User Story:** As a developer, I want clear documentation for both CLI and library usage, so that I can quickly understand how to use xchecker.

#### Acceptance Criteria

1. WHEN README.md is read THEN the system SHALL include an "Install from crates.io" section with `cargo install xchecker`
2. WHEN README.md is read THEN the system SHALL include an "Embedding xchecker" section showing `[dependencies] xchecker = "1"` and a minimal `OrchestratorHandle` example
3. WHEN rustdoc is generated THEN all public types at the crate root SHALL have documentation
4. WHEN docs/ORCHESTRATOR.md is read THEN it SHALL confirm "Outside orchestrator/, use OrchestratorHandle"

##### Security Documentation Accuracy (FR-DOC-5)
5. WHEN SECURITY.md is read THEN it SHALL describe protections actually implemented (redaction patterns, sandboxing, atomic writes)
6. WHEN SECURITY.md is read THEN it SHALL describe known limitations (what patterns are not covered, edge cases)
7. THE system SHALL NOT make blanket claims such as "no secrets ever" that are not strictly enforceable

##### Publication Documentation (FR-DOC-6)
8. WHEN documentation is read THEN it SHALL include a "Release & Publication" section describing versioning strategy (semver 1.x expectations)
9. WHEN documentation is read THEN it SHALL describe the crates.io publication process (tagging, CI gates)
10. WHEN documentation is read THEN it SHALL provide upgrade guidance for consumers

### Requirement 9 (FR-TEST)

**User Story:** As a maintainer, I want tests that validate the library API boundary, so that I can ensure the public API works correctly.

#### Acceptance Criteria

1. WHEN integration tests run THEN at least one test SHALL use only the public API (`OrchestratorHandle`, `PhaseId`, etc.)
2. Integration tests that use only the public API SHALL NOT depend on internal module paths or symbols
3. WHEN examples are built THEN at least one example SHALL demonstrate embedding via the public API
4. WHEN tests use internal modules THEN they SHALL be clearly marked as white-box tests
5. WHEN `cargo test --examples` runs THEN all examples SHALL compile and pass

##### Security Integration Tests (FR-TEST-6)
6. WHEN integration tests run THEN at least one test SHALL verify path traversal attempts are rejected
7. WHEN integration tests run THEN at least one test SHALL verify command injection attempts are blocked
8. WHEN integration tests run THEN at least one test SHALL verify secret strings in payloads are redacted
9. WHEN security tests run THEN they SHALL use the public CLI and/or OrchestratorHandle to ensure documented invariants hold

### Requirement 10 (FR-CONTRACT)

**User Story:** As an integrator, I want JSON contracts to remain unchanged, so that my CI gates and tooling continue to work.

#### Acceptance Criteria

1. WHEN receipts are emitted THEN they SHALL validate against `schemas/receipt.v1.json`
2. WHEN status is emitted THEN it SHALL validate against `schemas/status.v1.json`
3. WHEN doctor output is emitted THEN it SHALL validate against `schemas/doctor.v1.json`
4. WHEN JSON is emitted THEN it SHALL use JCS (RFC 8785) canonicalization
5. WHEN exit codes are returned THEN they SHALL match the documented exit code table
6. WHEN changes to JSON contracts would be breaking THEN the system SHALL follow the documented schema versioning and deprecation policy in CONTRACTS.md

### Requirement 11 (NFR-THREAD)

**User Story:** As a library consumer, I want to understand the threading semantics of the API, so that I can use it correctly in concurrent applications.

#### Acceptance Criteria

1. WHEN `OrchestratorHandle` is documented THEN the documentation SHALL specify whether it is `Send` and/or `Sync`
2. WHEN concurrent use is not supported THEN the documentation SHALL clearly state "single-threaded, not guaranteed thread-safe"
3. WHEN the library exposes async APIs THEN they SHALL be documented as such; if only sync APIs are exposed, the documentation SHALL note that async is handled internally

### Requirement 12 (NFR-ASYNC)

**User Story:** As a library consumer, I want to understand whether the API is sync or async, so that I can integrate it correctly with my runtime.

#### Acceptance Criteria

1. WHEN the public API returns `Result<(), XcError>` THEN it SHALL be a synchronous API that manages its own async runtime internally
2. WHEN async APIs are exposed in the future THEN they SHALL be clearly marked with `async fn` and documented separately
3. WHEN the library uses Tokio internally THEN this SHALL be an implementation detail not exposed to library consumers in 1.x
