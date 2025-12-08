# Design Document: crates.io Packaging & Library API

## Overview

This design formalizes the boundary between xchecker's library kernel and CLI layer, enabling:
1. Installation via `cargo install xchecker`
2. Embedding via `use xchecker::OrchestratorHandle`
3. Stable public API with semver guarantees for 1.x

The architecture follows a layered approach where the kernel contains all domain logic and the CLI is a thin wrapper that parses arguments, invokes the kernel, and maps errors to exit codes.

## Architecture

### Layer Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                        CLI Layer                             │
│  main.rs (< 50 lines) → cli.rs (clap, output formatting)    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Public API (lib.rs)                       │
│  OrchestratorHandle, PhaseId, Config, XcError, ExitCode     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      Kernel Layer                            │
│  orchestrator/, phases.rs, packet.rs, fixup.rs, runner.rs   │
│  redaction.rs, receipt.rs, status.rs, config.rs, lock.rs    │
│  canonicalization.rs, atomic_write.rs, artifact.rs, etc.    │
└─────────────────────────────────────────────────────────────┘
```

### Module Ownership

| Module | Layer | Stability | Notes |
|--------|-------|-----------|-------|
| `main.rs` | CLI | Internal | Minimal entrypoint |
| `cli.rs` | CLI | Internal | clap definitions, output formatting |
| `lib.rs` | Public API | Stable | Re-exports stable types |
| `orchestrator/` | Kernel | Internal | `OrchestratorHandle` is stable facade |
| `phases.rs` | Kernel | Internal | `PhaseId` enum is stable |
| `config.rs` | Kernel | Internal | `Config` type is stable |
| `error.rs` | Kernel | Internal | `XcError` type is stable |
| `exit_codes.rs` | Kernel | Internal | `ExitCode` type is stable |
| All others | Kernel | Internal | Implementation details |

## Components and Interfaces

### 1. Public API Surface (`src/lib.rs`)

```rust
//! xchecker - Spec pipeline with receipts and gateable JSON contracts
//!
//! # Quick Start (CLI)
//! ```bash
//! cargo install xchecker
//! xchecker spec my-feature --dry-run
//! ```
//!
//! # Quick Start (Library)
//! ```rust,no_run
//! use xchecker::{OrchestratorHandle, PhaseId};
//!
//! fn main() -> Result<(), xchecker::XcError> {
//!     let handle = OrchestratorHandle::new("my-spec")?;
//!     handle.run_phase(PhaseId::Requirements)?;
//!     Ok(())
//! }
//! ```

// Stable public API - covered by semver guarantees
pub use orchestrator::OrchestratorHandle;
pub use phases::PhaseId;
pub use config::Config;
pub use error::XcError;
pub use exit_codes::ExitCode;
pub use status::StatusOutput;

// Internal modules - accessible but not stable
// NOTE: cli module is NOT exported here - it's only used by main.rs via `mod cli;`
#[doc(hidden)]
pub mod orchestrator;
#[doc(hidden)]
pub mod phases;
#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod error;
#[doc(hidden)]
pub mod exit_codes;
#[doc(hidden)]
pub mod status;
// ... other internal modules (but NOT cli)
```

### 2. OrchestratorHandle Facade

```rust
/// The primary public API for embedding xchecker.
///
/// `OrchestratorHandle` provides a stable interface for creating specs
/// and running phases programmatically. It is the canonical way to use
/// xchecker outside of the CLI.
///
/// # Threading
/// `OrchestratorHandle` is NOT guaranteed `Send` or `Sync` in 1.x.
/// Treat as single-threaded; concurrent use is undefined behavior.
/// This may be relaxed in future versions.
///
/// # Mutability
/// Methods take `&mut self` to encode "sequential use only" semantics.
/// This prevents accidental concurrent use at compile time.
///
/// # Example
/// ```rust,no_run
/// use xchecker::{OrchestratorHandle, PhaseId, Config};
///
/// // Using environment-based config discovery
/// let mut handle = OrchestratorHandle::new("my-spec")?;
/// handle.run_phase(PhaseId::Requirements)?;
///
/// // Using explicit configuration
/// let config = Config::builder()
///     .state_dir("/custom/path")
///     .build()?;
/// let mut handle = OrchestratorHandle::from_config("my-spec", config)?;
/// handle.run_all()?;
/// # Ok::<(), xchecker::XcError>(())
/// ```
pub struct OrchestratorHandle {
    inner: PhaseOrchestrator,
}

impl OrchestratorHandle {
    /// Create a handle using environment-based config discovery.
    ///
    /// This uses the same discovery logic as the CLI:
    /// - `XCHECKER_HOME` environment variable
    /// - Upward search for `.xchecker/config.toml`
    /// - Built-in defaults
    pub fn new(spec_id: &str) -> Result<Self, XcError>;

    /// Create a handle using explicit configuration.
    ///
    /// This does NOT probe the global environment or filesystem.
    /// Use this when you need deterministic behavior independent
    /// of the user's environment.
    pub fn from_config(spec_id: &str, config: Config) -> Result<Self, XcError>;

    /// Execute a single phase.
    ///
    /// Behavior matches the CLI `xchecker resume --phase <phase>` command.
    /// Takes `&mut self` to enforce sequential use.
    pub fn run_phase(&mut self, phase: PhaseId) -> Result<(), XcError>;

    /// Execute all phases in sequence.
    ///
    /// Stops on first failure. Behavior matches the CLI `xchecker spec` command.
    /// Takes `&mut self` to enforce sequential use.
    pub fn run_all(&mut self) -> Result<(), XcError>;

    /// Get the current spec status.
    ///
    /// Returns `StatusOutput` which is part of the stable public API.
    pub fn status(&self) -> Result<StatusOutput, XcError>;

    /// Get the path to the most recent receipt.
    pub fn last_receipt_path(&self) -> Option<PathBuf>;

    /// Get the spec ID this handle operates on.
    pub fn spec_id(&self) -> &str;
}
```

### 3. Config Builder

```rust
/// Configuration for xchecker operations.
///
/// Use `Config::discover_from_env_and_fs()` for CLI-like behavior,
/// or `Config::builder()` for programmatic configuration.
pub struct Config {
    // Internal fields with source attribution
}

impl Config {
    /// Discover configuration using CLI semantics.
    ///
    /// Searches for `.xchecker/config.toml` upward from CWD,
    /// respects `XCHECKER_HOME`, and applies defaults.
    pub fn discover_from_env_and_fs() -> Result<Self, XcError>;

    /// Create a builder for programmatic configuration.
    pub fn builder() -> ConfigBuilder;
}

pub struct ConfigBuilder {
    // Builder fields
}

impl ConfigBuilder {
    pub fn state_dir(self, path: impl Into<PathBuf>) -> Self;
    pub fn packet_max_bytes(self, bytes: usize) -> Self;
    pub fn packet_max_lines(self, lines: usize) -> Self;
    pub fn phase_timeout(self, timeout: Duration) -> Self;
    pub fn runner_mode(self, mode: RunnerMode) -> Self;
    // ... other configuration options
    pub fn build(self) -> Result<Config, XcError>;
}
```

### 4. StatusOutput (Stable Public Type)

```rust
/// Status output for a spec, matching `schemas/status.v1.json`.
///
/// This is a stable public type. Changes must be additive only in 1.x.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusOutput {
    pub schema_version: String,
    pub emitted_at: String,  // RFC3339 UTC
    pub spec_id: String,
    pub artifacts: Vec<ArtifactMeta>,
    pub effective_config: BTreeMap<String, ConfigValue>,
    pub lock_drift: Option<DriftInfo>,
    pub pending_fixups: Option<PendingFixups>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMeta {
    pub path: String,
    pub blake3_first8: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValue {
    pub value: serde_json::Value,
    pub source: String,  // "cli" | "config" | "default" | "programmatic"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftInfo {
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingFixups {
    pub targets: u32,
    pub est_added: u32,
    pub est_removed: u32,
}
```

### 5. Security Subsystems

#### 5.1 Redaction Subsystem (`src/redaction.rs`)

The redaction subsystem provides secret detection and redaction before any external invocation or persistence.

```rust
/// Redaction configuration applied globally.
/// 
/// Built-in patterns cover documented categories; users can extend or exclude.
pub struct RedactionConfig {
    /// Built-in patterns covering: cloud keys, API tokens, DB URLs, SSH keys
    builtins: Vec<CompiledPattern>,
    /// User-provided additional patterns from config
    extra_patterns: Vec<CompiledPattern>,
    /// User-provided patterns to ignore (reduce false positives)
    ignore_patterns: Vec<CompiledPattern>,
}

impl RedactionConfig {
    /// Create default config with all built-in patterns.
    /// 
    /// Built-in categories:
    /// - AWS: access keys, secret keys, session tokens
    /// - GCP: service account keys, API keys
    /// - Azure: storage keys, connection strings
    /// - Generic: API keys, OAuth tokens, bearer tokens
    /// - Database: Postgres, MySQL, SQL Server connection strings
    /// - SSH: private keys, PEM secrets
    pub fn default() -> Self;
    
    /// Create from user configuration.
    pub fn from_config(config: &Config) -> Self;
}

/// Single entry point for all redaction.
/// 
/// All logging, JSON emission, and external invocation surfaces
/// MUST call this before output.
pub fn redact_all(config: &RedactionConfig, input: &str) -> String;
```

**Design Rules:**
- Pattern compilation happens at startup, not per-call
- Single pass over text per sink for performance
- All logging and JSON emission surfaces MUST call `redact_all` before output

#### 5.2 Path Sandbox Subsystem (`src/paths.rs`)

The path sandbox provides validated, constrained path operations.

```rust
/// A validated root directory for sandboxed operations.
/// 
/// All paths derived from this root are guaranteed to stay within it.
pub struct SandboxRoot {
    /// Canonicalized absolute path to the root
    root: PathBuf,
}

/// A path that has been validated to be within a SandboxRoot.
/// 
/// Cannot be constructed directly; must come from SandboxRoot::join().
pub struct SandboxPath {
    root: SandboxRoot,
    /// Relative path from root, validated to contain no ".."
    rel: PathBuf,
}

impl SandboxRoot {
    /// Create a new sandbox root from a path.
    /// 
    /// Canonicalizes the path and verifies it exists.
    pub fn new(root: impl AsRef<Path>) -> Result<Self, XcError>;
    
    /// Join a relative path, validating it stays within the sandbox.
    /// 
    /// Rejects:
    /// - Paths containing ".."
    /// - Absolute paths
    /// - Symlinks (unless explicitly enabled)
    pub fn join(&self, rel: impl AsRef<Path>) -> Result<SandboxPath, XcError>;
}

impl SandboxPath {
    /// Get the full path for I/O operations.
    pub fn as_path(&self) -> &Path;
    
    /// Get the relative portion.
    pub fn relative(&self) -> &Path;
}
```

**Design Rules:**
- All file I/O in fixup, artifact discovery, and cache/state handling uses `SandboxPath`
- Raw `PathBuf` concatenation is forbidden for user-controlled paths
- `SandboxRoot::join` canonicalizes, rejects escapes, and optionally rejects symlinks

#### 5.3 Runner Subsystem (`src/runner.rs`)

The runner provides safe process execution without shell injection.

```rust
/// Specification for a command to execute.
/// 
/// All process execution goes through this type to ensure argv-style invocation.
pub struct CommandSpec {
    /// The program to execute
    pub program: OsString,
    /// Arguments as discrete elements (NOT shell strings)
    pub args: Vec<OsString>,
    /// Optional working directory
    pub cwd: Option<PathBuf>,
    /// Optional environment overrides
    pub env: Option<HashMap<OsString, OsString>>,
}

/// Trait for process execution.
/// 
/// Implementations MUST use argv-style APIs only.
pub trait ProcessRunner {
    fn run(&self, cmd: &CommandSpec, timeout: Duration) -> Result<Output, XcError>;
}

/// Native process runner using std::process::Command.
pub struct NativeRunner;

impl ProcessRunner for NativeRunner {
    fn run(&self, cmd: &CommandSpec, timeout: Duration) -> Result<Output, XcError> {
        // Uses Command::new(&cmd.program).args(&cmd.args) only
        // NO shell string construction
    }
}

/// WSL process runner for Windows.
pub struct WslRunner;

impl ProcessRunner for WslRunner {
    fn run(&self, cmd: &CommandSpec, timeout: Duration) -> Result<Output, XcError> {
        // Builds Command::new("wsl") with argv entries
        // NO string concatenation of user data
    }
}
```

**Design Rules:**
- Runner is the ONLY place that talks to `std::process::Command`
- All orchestrator/CLI paths go through the runner with `CommandSpec`
- No production code shall call `Command::new("sh").arg("-c", ...)` or similar

### 6. Error Handling

```rust
/// Library-level error type with rich context.
///
/// `XcError` provides detailed error information including:
/// - Error kind (for programmatic handling)
/// - Context (what was being attempted)
/// - Actionable suggestions (how to fix)
///
/// Use `to_exit_code()` to map to CLI exit codes.
/// Use `display_for_user()` for human-readable messages.
#[derive(Debug)]
pub struct XcError {
    kind: ErrorKind,
    context: String,
    suggestion: Option<String>,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl XcError {
    /// Map this error to the appropriate CLI exit code.
    pub fn to_exit_code(&self) -> ExitCode;

    /// Get a user-friendly error message.
    pub fn display_for_user(&self) -> String;

    /// Get the error kind for programmatic handling.
    pub fn kind(&self) -> &ErrorKind;
}

/// Exit codes matching the documented exit code table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExitCode(i32);

impl ExitCode {
    pub const SUCCESS: ExitCode = ExitCode(0);
    pub const CLI_ARGS: ExitCode = ExitCode(2);
    pub const PACKET_OVERFLOW: ExitCode = ExitCode(7);
    pub const SECRET_DETECTED: ExitCode = ExitCode(8);
    pub const LOCK_HELD: ExitCode = ExitCode(9);
    pub const PHASE_TIMEOUT: ExitCode = ExitCode(10);
    pub const CLAUDE_FAILURE: ExitCode = ExitCode(70);

    pub fn as_i32(self) -> i32 { self.0 }
}
```

### 5. CLI Layer (`src/main.rs`)

```rust
// NOTE: cli module is NOT part of the public API
// It's declared here in main.rs, not exported from lib.rs
mod cli;

fn main() {
    // cli::run() handles all output (including --json mode)
    // Only errors bubble up for exit code mapping
    if let Err(err) = cli::run() {
        let code = err.to_exit_code();
        eprintln!("{}", err.display_for_user());
        std::process::exit(code.as_i32());
    }
}
```

### 6. CLI Module (`src/cli.rs`)

```rust
use clap::{Parser, Subcommand};
// Import from crate root (public API) - NOT from internal modules
use crate::{OrchestratorHandle, Config, XcError, PhaseId, StatusOutput};

#[derive(Parser)]
#[command(name = "xchecker")]
#[command(about = "Spec pipeline with receipts and gateable JSON contracts")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true)]
    verbose: bool,

    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    Spec { /* ... */ },
    Resume { /* ... */ },
    Status { /* ... */ },
    Clean { /* ... */ },
    Doctor { /* ... */ },
    Init { /* ... */ },
    Benchmark { /* ... */ },
    Gate { /* ... */ },
}

/// Entry point for CLI execution.
///
/// Parses arguments, creates appropriate Config, invokes OrchestratorHandle,
/// handles all output (including --json mode), and returns Result for main.rs.
/// 
/// On success, this function handles all output internally.
/// On error, returns XcError for main.rs to map to exit code.
pub fn run() -> Result<(), XcError> {
    let cli = Cli::parse();
    
    // Set up tracing subscriber based on --verbose
    // This is the CLI's responsibility, not the library's
    setup_logging(cli.verbose);
    
    match cli.command {
        Commands::Spec { spec_id, dry_run, .. } => {
            let config = Config::discover_from_env_and_fs()?;
            let mut handle = OrchestratorHandle::from_config(&spec_id, config)?;
            if dry_run {
                // Handle dry-run (print what would happen)
                println!("Would run all phases for spec: {}", spec_id);
            } else {
                handle.run_all()?;
            }
        }
        Commands::Status { spec_id, .. } => {
            let config = Config::discover_from_env_and_fs()?;
            let handle = OrchestratorHandle::from_config(&spec_id, config)?;
            let status = handle.status()?;
            
            if cli.json {
                // Use same canonicalization as receipts/status/doctor
                let json = crate::canonicalization::emit_jcs(&status)?;
                println!("{}", json);
            } else {
                // Human-readable output
                println!("Spec: {}", spec_id);
                println!("Artifacts: {}", status.artifacts.len());
            }
        }
        // ... other commands
    }
    
    Ok(())
}
```

## Data Models

### Cargo.toml Metadata

```toml
[package]
name = "xchecker"
version = "1.0.0"
edition = "2024"
rust-version = "1.91"
description = "Spec pipeline with receipts and gateable JSON contracts"
repository = "https://github.com/EffortlessMetrics/xchecker"
homepage = "https://github.com/EffortlessMetrics/xchecker"
license = "MIT OR Apache-2.0"
readme = "README.md"
categories = ["command-line-utilities", "development-tools"]
keywords = ["cli", "spec", "requirements", "workflow", "automation"]

[package.metadata.docs.rs]
all-features = true

# Exclude internal directories from crates.io package
exclude = [
    ".xchecker/",
    ".kiro/",
    ".github/",
    ".vscode/",
]

[[bin]]
name = "xchecker"
path = "src/main.rs"

# claude-stub is dev-only, NOT installed via `cargo install xchecker`
# It IS present in the published crate but requires the dev-tools feature to build.
# This satisfies FR-PKG-7: "SHALL NOT be installed via cargo install"
[[bin]]
name = "claude-stub"
path = "src/bin/claude_stub.rs"
required-features = ["dev-tools"]

[features]
default = []
dev-tools = []  # Enables claude-stub binary (for testing only)
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Config discovery matches CLI behavior
*For any* environment configuration (XCHECKER_HOME, config files, CWD), calling `Config::discover_from_env_and_fs()` SHALL produce the same effective configuration as the CLI would use for the same environment.
**Validates: Requirements 1.3, 6.1**

### Property 2: from_config ignores environment
*For any* environment variables or config files present, calling `OrchestratorHandle::from_config()` with an explicit `Config` SHALL NOT be affected by those environment variables or files.
**Validates: Requirements 1.4, 6.3**

### Property 3: Library behavior matches CLI
*For any* spec and phase, calling `handle.run_phase(phase)` SHALL produce the same artifacts, receipts, and state changes as invoking `xchecker resume --phase <phase>` via CLI.
**Validates: Requirements 1.7**

### Property 4: Errors return XcError, not panic/exit
*For any* error condition in library code, the system SHALL return `Result::Err(XcError)` and SHALL NOT call `std::process::exit()` or panic (except for documented internal invariant violations).
**Validates: Requirements 1.8, 5.5, 5.6**

### Property 5: Logging uses tracing
*For any* library operation, all log output SHALL go through the `tracing` infrastructure and SHALL NOT write directly to stdout/stderr.
**Validates: Requirements 1.10**

### Property 6: Error to exit code mapping
*For any* `XcError` with a given `ErrorKind`, calling `to_exit_code()` SHALL return the exit code documented in the exit code table. This is the single source of truth for both CLI exit codes and receipt `exit_code` fields.
**Validates: Requirements 2.3, 5.2, 10.5**

### Property 7: JCS canonicalization
*For any* JSON output (receipts, status, doctor), the emitted JSON SHALL be JCS-canonical (RFC 8785), meaning re-serialization produces byte-identical output.
**Validates: Requirements 2.6, 10.4**

### Property 8: Error context and suggestions
*For any* `XcError`, the error SHALL include a non-empty context string and MAY include an actionable suggestion.
**Validates: Requirements 5.1**

### Property 9: Exit code matches receipt
*For any* CLI invocation that writes an error receipt, the process exit code SHALL match the `exit_code` field in the receipt.
**Validates: Requirements 5.4**

### Property 10: Source attribution
*For any* configuration value in `EffectiveConfig`, the source attribution SHALL be exactly one of: `"cli"`, `"config"`, `"default"`, or `"programmatic"`.
**Validates: Requirements 6.4**

### Property 11: Secret redaction coverage
*For any* content containing patterns matching the documented secret categories (AWS keys, GCP keys, Azure keys, generic API tokens, database URLs, SSH keys), the system SHALL redact those patterns before including the content in receipts, status, doctor outputs, or logs.
**Validates: Requirements FR-SEC-1, FR-SEC-2, FR-SEC-3, FR-SEC-4**

### Property 12: Redaction pipeline completeness
*For any* string that passes through LLM invocation, logging, or JSON emission, the string SHALL have been processed by `redact_all()` with the effective `RedactionConfig`.
**Validates: Requirements FR-SEC-1, FR-SEC-19**

### Property 13: Atomic writes for state files
*For any* write to spec artifacts, receipts, status files, or lock files, the write SHALL go through the `atomic_write` module (temp → fsync → rename) and SHALL NOT use direct `fs::write`.
**Validates: Requirements FR-SEC-5, FR-SEC-6, FR-SEC-7, FR-SEC-8, FR-SEC-9**

### Property 14: Path sandbox enforcement
*For any* user-provided path in fixup or artifact operations, the system SHALL:
1. Canonicalize the path
2. Verify it is within the configured root
3. Reject paths containing `..` traversal
4. Reject absolute paths outside the root
**Validates: Requirements FR-SEC-10, FR-SEC-11, FR-SEC-12, FR-SEC-13**

### Property 15: Symlink rejection
*For any* path that resolves to a symlink or hardlink, the system SHALL reject it unless symlinks are explicitly enabled in configuration.
**Validates: Requirements FR-SEC-14**

### Property 16: Argv-style execution
*For any* process execution in the library, arguments SHALL be passed as discrete argv elements via `CommandSpec`, and the system SHALL NOT use shell string evaluation (`sh -c`, `cmd /C`).
**Validates: Requirements FR-SEC-15, FR-SEC-16**

### Property 17: WSL runner safety
*For any* WSL command execution, the runner SHALL construct the command line via argv APIs with no string concatenation of unvalidated input.
**Validates: Requirements FR-SEC-17**

### Property 18: Argument validation at trust boundaries
*For any* argument that crosses a trust boundary (e.g., user-provided subcommands), the system SHALL validate or normalize the argument before execution.
**Validates: Requirements FR-SEC-18**

### Property 19: JSON schema validation
*For any* emitted JSON (receipts, status, doctor), the output SHALL validate against the corresponding v1 schema.
**Validates: Requirements 10.1, 10.2, 10.3**

### Property 20: StatusOutput is stable
*For any* call to `handle.status()`, the returned `StatusOutput` SHALL be a stable public type that can be serialized to JSON matching `schemas/status.v1.json`.
**Validates: Requirements 10.2**

## Error Handling

### Error Flow

```
Library Error → XcError → CLI catches → to_exit_code() → std::process::exit()
                                     → display_for_user() → eprintln!()
```

### Error Categories

| ErrorKind | Exit Code | Description |
|-----------|-----------|-------------|
| `CliArgs` | 2 | Invalid CLI arguments |
| `PacketOverflow` | 7 | Packet size exceeded |
| `SecretDetected` | 8 | Secret found in content |
| `LockHeld` | 9 | Lock already held |
| `PhaseTimeout` | 10 | Phase timed out |
| `ClaudeFailure` | 70 | Claude CLI failed |
| `Internal` | 1 | Internal error |

### No Panics Invariant

Library code SHALL NOT panic in production paths. The only acceptable panics are:
1. In test code
2. With documented invariants (e.g., `unreachable!()` after exhaustive match)
3. In `debug_assert!()` which is disabled in release builds

## Testing Strategy

### Dual Testing Approach

Both unit tests and property-based tests are required:
- **Unit tests**: Verify specific examples and edge cases
- **Property tests**: Verify universal properties across all inputs

### Property-Based Testing

The project uses `proptest` for property-based testing. Each property test:
- Runs a minimum of 100 iterations
- Is tagged with the property it validates
- References the requirements it covers

Example:
```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// **Feature: crates-io-packaging, Property 2: from_config ignores environment**
    #[test]
    fn from_config_ignores_env(
        env_home in ".*",
        config_path in ".*",
    ) {
        // Set environment variables that would normally affect discovery
        std::env::set_var("XCHECKER_HOME", &env_home);
        
        // Create explicit config with known state dir
        let explicit_dir = tempfile::tempdir().unwrap();
        let config = Config::builder()
            .state_dir(explicit_dir.path())
            .build()
            .unwrap();
        
        // from_config should use explicit config, not env
        let handle = OrchestratorHandle::from_config("test-spec", config).unwrap();
        
        // Verify spec_id is correct (observable output)
        prop_assert_eq!(handle.spec_id(), "test-spec");
        
        // Verify status works and doesn't error due to env mismatch
        let status = handle.status();
        prop_assert!(status.is_ok());
    }
}
```

### Test Organization

| Test Type | Location | Purpose |
|-----------|----------|---------|
| Unit tests | `src/*.rs` | Module-level testing |
| Property tests | `tests/property_based_tests.rs` | Universal properties |
| Integration tests | `tests/test_*.rs` | End-to-end flows |
| Public API tests | `tests/test_public_api.rs` | API boundary validation |
| Examples | `examples/*.rs` | Usage documentation |
| Doctests | `src/lib.rs` | API documentation |

### Public API Test Example

```rust
// tests/test_public_api.rs
// This test uses ONLY the public API - no internal modules

use xchecker::{OrchestratorHandle, PhaseId, Config, XcError, ExitCode};

#[test]
fn test_public_api_types_accessible() {
    // Verify all public types are accessible
    let _: fn(&str) -> Result<OrchestratorHandle, XcError> = OrchestratorHandle::new;
    let _: PhaseId = PhaseId::Requirements;
    let _: fn() -> Result<Config, XcError> = Config::discover_from_env_and_fs;
    let _: ExitCode = ExitCode::SUCCESS;
}

#[test]
fn test_orchestrator_handle_smoke() {
    // Create handle using public API
    let config = Config::builder()
        .state_dir(tempdir().path())
        .build()
        .unwrap();
    
    let handle = OrchestratorHandle::from_config("test-spec", config).unwrap();
    
    // Verify handle is usable
    let status = handle.status().unwrap();
    assert!(status.artifacts.is_empty());
}
```

### Example for Embedding

```rust
// examples/embed_requirements.rs
//! Example: Running the requirements phase programmatically

use xchecker::{OrchestratorHandle, PhaseId, Config};

fn main() -> Result<(), xchecker::XcError> {
    // Option 1: Use environment-based discovery (like CLI)
    let handle = OrchestratorHandle::new("my-feature")?;
    
    // Option 2: Use explicit configuration
    let config = Config::builder()
        .state_dir(".xchecker")
        .phase_timeout(std::time::Duration::from_secs(300))
        .build()?;
    let handle = OrchestratorHandle::from_config("my-feature", config)?;
    
    // Run a single phase
    handle.run_phase(PhaseId::Requirements)?;
    
    // Check status
    let status = handle.status()?;
    println!("Artifacts: {:?}", status.artifacts);
    
    Ok(())
}
```
