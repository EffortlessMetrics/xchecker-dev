//! CLI argument definitions and parsing structures
//!
//! This module defines the command-line interface structure using clap,
//! including the main `Cli` struct and all subcommand enums.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// xchecker - Claude orchestration tool for spec generation
#[derive(Parser)]
#[command(name = "xchecker")]
#[command(about = "A CLI tool for orchestrating spec generation workflows using LLM providers")]
#[command(long_about = r#"
xchecker is a deterministic, token-efficient pipeline that transforms rough ideas
into detailed implementation plans through a structured phase-based approach.

EXAMPLES:
  # Generate a spec from stdin input
  echo "Create a REST API for user management" | xchecker spec user-api

  # Generate a spec from a GitHub issue
  xchecker spec issue-123 --source gh --gh owner/repo

  # Generate a spec from local filesystem context
  xchecker spec my-feature --source fs --repo /path/to/project

  # Run in dry-run mode to see what would be executed
  xchecker spec test-spec --dry-run --verbose

  # Check status of a spec
  xchecker status user-api

  # Resume from a specific phase
  xchecker resume user-api --phase design

  # Clean up spec artifacts
  xchecker clean user-api

  # Run performance benchmarks
  xchecker benchmark --file-count 50 --iterations 3

  # Run integration tests
  xchecker test --components --smoke

CONFIGURATION:
  Configuration is loaded with precedence: CLI flags > config file > defaults
  Config file is discovered by searching upward from CWD for .xchecker/config.toml
  Use --config to specify an explicit config file path

PHASES:
  Requirements → Design → Tasks → Review → Fixup → Final
  Each phase produces artifacts and receipts for auditability
  Use --dry-run to see planned execution without making LLM calls

For more information, see: https://github.com/EffortlessMetrics/xchecker
"#)]
#[command(version)]
pub struct Cli {
    /// Path to configuration file (overrides discovery)
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    /// Model to use for LLM provider calls
    #[arg(long, global = true)]
    pub model: Option<String>,

    /// Maximum number of turns for Claude interactions
    #[arg(long, global = true)]
    pub max_turns: Option<u32>,

    /// Maximum bytes for packet content
    #[arg(long, global = true)]
    pub packet_max_bytes: Option<usize>,

    /// Maximum lines for packet content
    #[arg(long, global = true)]
    pub packet_max_lines: Option<usize>,

    /// Output format for Claude CLI (stream-json or text)
    #[arg(long, global = true)]
    pub output_format: Option<String>,

    /// Runner mode: native (direct), wsl (Windows only), or auto (detect best option)
    #[arg(long, global = true)]
    pub runner_mode: Option<String>,

    /// WSL distribution to use (when `runner_mode` is wsl)
    #[arg(long, global = true)]
    pub runner_distro: Option<String>,

    /// Path to Claude CLI binary in WSL
    #[arg(long, global = true)]
    pub claude_path: Option<String>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Tool patterns to allow (passed to Claude as allowedTools)
    #[arg(long, global = true)]
    pub allow: Vec<String>,

    /// Tool patterns to deny (passed to Claude as disallowedTools)
    #[arg(long, global = true)]
    pub deny: Vec<String>,

    /// Skip permission checks (passed to Claude)
    #[arg(long, global = true)]
    pub dangerously_skip_permissions: bool,

    /// Ignore specific secret patterns by pattern ID (e.g. github_pat, aws_access_key)
    #[arg(long, global = true)]
    pub ignore_secret_pattern: Vec<String>,

    /// Add extra secret patterns to detect (regex)
    #[arg(long, global = true)]
    pub extra_secret_pattern: Vec<String>,

    /// Phase timeout in seconds (default: 600, min: 5)
    #[arg(long, global = true)]
    pub phase_timeout: Option<u64>,

    /// Maximum bytes for stdout ring buffer (default: 2097152 = 2 MiB)
    #[arg(long, global = true)]
    pub stdout_cap_bytes: Option<usize>,

    /// Maximum bytes for stderr ring buffer (default: 262144 = 256 KiB)
    #[arg(long, global = true)]
    pub stderr_cap_bytes: Option<usize>,

    /// Lock TTL in seconds (default: 900 = 15 minutes)
    #[arg(long, global = true)]
    pub lock_ttl_seconds: Option<u64>,

    /// Write full packet to `context/<phase>-packet.txt` after secret scan passes
    #[arg(long, global = true)]
    pub debug_packet: bool,

    /// Allow symlinks and hardlinks in fixup targets
    #[arg(long, global = true)]
    pub allow_links: bool,

    /// Enable strict validation (validation failures fail phases)
    /// Use --no-strict-validation to disable strict mode
    #[arg(long, global = true, overrides_with = "no_strict_validation")]
    pub strict_validation: bool,

    /// Disable strict validation (validation failures log warnings only)
    /// Use --strict-validation to enable strict mode
    #[arg(long, global = true, overrides_with = "strict_validation")]
    pub no_strict_validation: bool,

    /// LLM provider to use (claude-cli)
    #[arg(long, global = true)]
    pub llm_provider: Option<String>,

    /// Path to Claude CLI binary (for claude-cli provider)
    #[arg(long, global = true)]
    pub llm_claude_binary: Option<String>,

    /// Execution strategy: controlled (default, LLMs cannot write directly)
    #[arg(long, global = true)]
    pub execution_strategy: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands
#[derive(Subcommand)]
pub enum Commands {
    /// Generate a complete spec flow (Requirements → Design → Tasks)
    ///
    /// This command executes the full spec generation workflow, transforming
    /// a rough idea into detailed requirements, design, and implementation tasks.
    ///
    /// EXAMPLES:
    ///   echo "Build a calculator app" | xchecker spec calc-app
    ///   xchecker spec issue-42 --source gh --gh myorg/myrepo
    ///   xchecker spec new-feature --source fs --repo ./project --dry-run
    Spec {
        /// Unique identifier for the spec
        id: String,

        /// Source type: 'gh' (GitHub issue), 'fs' (filesystem), 'stdin' (standard input)
        #[arg(long, default_value = "stdin", value_parser = ["gh", "fs", "stdin"])]
        source: String,

        /// GitHub repository in format 'owner/repo' (required when --source gh)
        #[arg(long, help = "GitHub repository (e.g., 'myorg/myrepo')")]
        gh: Option<String>,

        /// Local repository path (required when --source fs)
        #[arg(long, help = "Path to local repository directory")]
        repo: Option<String>,

        /// Run in dry-run mode (show what would be executed without making LLM calls)
        #[arg(long)]
        dry_run: bool,

        /// Force override of stale locks
        #[arg(long)]
        force: bool,

        /// Apply fixups to files (default is preview mode)
        #[arg(long)]
        apply_fixups: bool,

        /// Hard fail on lockfile drift (exit with error if model/CLI version differs)
        #[arg(long)]
        strict_lock: bool,

        /// Output spec information as JSON (for Claude Code integration)
        #[arg(long)]
        json: bool,
    },

    /// Show status of a spec
    ///
    /// Displays the current state of a spec including completed phases,
    /// artifacts with BLAKE3 hashes, receipts, and configuration.
    ///
    /// EXAMPLES:
    ///   xchecker status my-spec
    ///   xchecker status my-spec --json
    Status {
        /// Spec ID to check status for
        id: String,

        /// Output status as JSON
        #[arg(long)]
        json: bool,
    },

    /// Resume execution from a specific phase
    ///
    /// Continues spec generation from a specific phase, useful for recovery
    /// from failures or re-running phases with different parameters.
    ///
    /// EXAMPLES:
    ///   xchecker resume my-spec --phase design
    ///   xchecker resume my-spec --phase requirements --dry-run
    ///   xchecker resume my-spec --phase design --json
    Resume {
        /// Spec ID to resume
        id: String,

        /// Phase to resume from: requirements, design, tasks, review, fixup, final
        #[arg(long, value_parser = ["requirements", "design", "tasks", "review", "fixup", "final"])]
        phase: String,

        /// Run in dry-run mode (show what would be executed without making LLM calls)
        #[arg(long)]
        dry_run: bool,

        /// Force override of stale locks
        #[arg(long)]
        force: bool,

        /// Apply fixups to files (default is preview mode)
        #[arg(long)]
        apply_fixups: bool,

        /// Hard fail on lockfile drift (exit with error if model/CLI version differs)
        #[arg(long)]
        strict_lock: bool,

        /// Output resume information as JSON (for Claude Code integration)
        #[arg(long)]
        json: bool,
    },

    /// Clean up spec artifacts and receipts
    ///
    /// Removes all artifacts, receipts, and context files for a spec.
    /// Use --hard to skip confirmation prompts.
    ///
    /// EXAMPLES:
    ///   xchecker clean my-spec
    ///   xchecker clean my-spec --hard --force
    Clean {
        /// Spec ID to clean
        id: String,

        /// Remove artifacts without confirmation
        #[arg(long)]
        hard: bool,

        /// Force removal even if lock is present
        #[arg(long)]
        force: bool,
    },

    /// Run performance benchmarks (NFR1 validation)
    ///
    /// Validates performance targets: empty run ≤ 5s, packetization ≤ 200ms for 100 files.
    /// Useful for regression testing and performance validation.
    ///
    /// EXAMPLES:
    ///   xchecker benchmark
    ///   xchecker benchmark --file-count 50 --iterations 10 --verbose
    ///   xchecker benchmark --json
    ///   xchecker benchmark --max-empty-run-secs 3.0 --max-packetization-ms 150.0
    Benchmark {
        /// Number of files to create for packetization benchmark
        #[arg(long, default_value = "100")]
        file_count: usize,

        /// Size of each test file in bytes
        #[arg(long, default_value = "1024")]
        file_size: usize,

        /// Number of benchmark iterations
        #[arg(long, default_value = "5")]
        iterations: usize,

        /// Output benchmark results as JSON
        #[arg(long)]
        json: bool,

        /// Maximum allowed empty run time in seconds (default: 5.0)
        #[arg(long)]
        max_empty_run_secs: Option<f64>,

        /// Maximum allowed packetization time in milliseconds per 100 files (default: 200.0)
        #[arg(long)]
        max_packetization_ms: Option<f64>,

        /// Maximum allowed RSS memory in MB (optional)
        #[arg(long)]
        max_rss_mb: Option<f64>,

        /// Maximum allowed commit memory in MB (Windows only, optional)
        #[arg(long)]
        max_commit_mb: Option<f64>,
    },

    /// Run integration smoke tests to validate all components
    ///
    /// Validates that all systems are properly integrated and working.
    /// Useful for development and CI/CD validation.
    ///
    /// EXAMPLES:
    ///   xchecker test
    ///   xchecker test --components --smoke --verbose
    Test {
        /// Run component validation tests
        #[arg(long)]
        components: bool,

        /// Run smoke tests
        #[arg(long)]
        smoke: bool,
    },

    /// Run environment health checks
    ///
    /// Validates that Claude CLI is installed, runner configuration is correct,
    /// write permissions are available, and configuration is valid.
    ///
    /// EXAMPLES:
    ///   xchecker doctor
    ///   xchecker doctor --json
    ///   xchecker doctor --strict-exit  # Treat warnings as failures
    Doctor {
        /// Output doctor results as JSON
        #[arg(long)]
        json: bool,

        /// Treat warnings as failures (exit non-zero on any warn or fail)
        #[arg(long)]
        strict_exit: bool,
    },

    /// Initialize a new spec with optional lockfile creation
    ///
    /// Creates the spec directory structure and optionally pins the model
    /// and Claude CLI version for reproducibility tracking.
    ///
    /// EXAMPLES:
    ///   xchecker init my-spec
    ///   xchecker init my-spec --create-lock
    ///   xchecker init my-spec --create-lock --model haiku
    Init {
        /// Spec ID to initialize
        id: String,

        /// Create a lockfile to pin model and CLI version
        #[arg(long)]
        create_lock: bool,
    },

    /// Manage workspace and multi-spec projects
    ///
    /// Workspace commands allow managing multiple specs within a project.
    /// A workspace is defined by a `workspace.yaml` file that contains
    /// metadata about registered specs.
    ///
    /// EXAMPLES:
    ///   xchecker project init my-project
    ///   xchecker project add-spec feature-auth --tag backend
    ///   xchecker project list
    #[command(subcommand)]
    Project(ProjectCommands),

    /// Gate merges on xchecker receipts/status
    ///
    /// Evaluates a spec against a configurable policy to determine if it
    /// meets requirements for CI/CD gates. Returns exit code 0 on policy
    /// success, 1 on policy violation.
    ///
    /// EXAMPLES:
    ///   xchecker gate my-spec
    ///   xchecker gate my-spec --min-phase design
    ///   xchecker gate my-spec --fail-on-pending-fixups
    ///   xchecker gate my-spec --max-phase-age 7d
    ///   xchecker gate my-spec --json
    ///
    /// Per FR-GATE (Requirements 4.5.1, 4.5.2, 4.5.3, 4.5.4)
    Gate {
        /// Spec ID to evaluate
        id: String,

        /// Minimum phase that must be completed (default: tasks)
        /// Valid phases: requirements, design, tasks, review, fixup, final
        #[arg(long, default_value = "tasks")]
        min_phase: String,

        /// Fail if any pending fixups exist
        #[arg(long)]
        fail_on_pending_fixups: bool,

        /// Maximum age of the latest successful phase run (e.g., "7d", "24h", "30m")
        /// Failed receipts do not count towards age (prevents flapping phases from appearing fresh)
        #[arg(long)]
        max_phase_age: Option<String>,

        /// Output gate results as JSON (gate-json.v1 schema)
        #[arg(long)]
        json: bool,
    },

    /// Manage spec templates
    ///
    /// Templates provide predefined configurations and problem statements
    /// for common use cases, allowing you to bootstrap specs quickly.
    ///
    /// EXAMPLES:
    ///   xchecker template list
    ///   xchecker template init fullstack-nextjs my-app
    ///   xchecker template init rust-microservice my-service
    ///
    /// Per FR-TEMPLATES (Requirements 4.7.1, 4.7.2, 4.7.3)
    #[command(subcommand)]
    Template(TemplateCommands),
}

/// Project/workspace management subcommands
#[derive(Subcommand)]
pub enum ProjectCommands {
    /// Initialize a new workspace in the current directory
    ///
    /// Creates a `workspace.yaml` file that marks this directory as a project root.
    /// The workspace can then be used to manage multiple specs.
    ///
    /// EXAMPLES:
    ///   xchecker project init my-project
    Init {
        /// Name for the workspace
        name: String,
    },

    /// Add a spec to the workspace
    ///
    /// Registers a spec in the workspace registry with optional tags.
    ///
    /// EXAMPLES:
    ///   xchecker project add-spec feature-auth
    ///   xchecker project add-spec feature-auth --tag backend --tag security
    AddSpec {
        /// Spec ID to add
        spec_id: String,

        /// Tags for categorization (can be specified multiple times)
        #[arg(long, short)]
        tag: Vec<String>,

        /// Force override if spec already exists
        #[arg(long)]
        force: bool,
    },

    /// List all specs in the workspace
    ///
    /// Shows all registered specs with their tags and status.
    ///
    /// EXAMPLES:
    ///   xchecker project list
    List {
        /// Path to workspace file (overrides discovery)
        #[arg(long)]
        workspace: Option<PathBuf>,
    },

    /// Show aggregated status for all specs in the workspace
    ///
    /// Displays status summary for all registered specs including phase summaries,
    /// counts of failed/pending/stale specs, and overall workspace health.
    ///
    /// EXAMPLES:
    ///   xchecker project status
    ///   xchecker project status --json
    Status {
        /// Path to workspace file (overrides discovery)
        #[arg(long)]
        workspace: Option<PathBuf>,

        /// Output status as JSON (workspace-status-json.v1 schema)
        #[arg(long)]
        json: bool,
    },

    /// Show history timeline for a spec
    ///
    /// Displays timeline of phase progression, timestamps, and selected metrics
    /// including LLM token usage and fixup counts.
    ///
    /// EXAMPLES:
    ///   xchecker project history feature-auth
    ///   xchecker project history feature-auth --json
    ///
    /// Per FR-WORKSPACE (Requirements 4.3.5): Emits timeline of phase progression
    History {
        /// Spec ID to show history for
        spec_id: String,

        /// Output history as JSON (workspace-history-json.v1 schema)
        #[arg(long)]
        json: bool,
    },

    /// Launch interactive terminal UI for workspace overview
    ///
    /// Displays an interactive TUI showing specs list with tags and status,
    /// latest receipt summary per selected spec, pending fixups, error counts,
    /// and stale specs. The TUI is read-only (no destructive operations).
    ///
    /// Navigation:
    ///   - Arrow keys or j/k: Move selection up/down
    ///   - Enter: View details for selected spec
    ///   - Esc: Go back / close details
    ///   - q: Quit
    ///
    /// EXAMPLES:
    ///   xchecker project tui
    ///
    /// Per FR-WORKSPACE-TUI (Requirements 4.4.1, 4.4.2, 4.4.3)
    Tui {
        /// Path to workspace file (overrides discovery)
        #[arg(long)]
        workspace: Option<PathBuf>,
    },
}

/// Template management subcommands
#[derive(Subcommand)]
pub enum TemplateCommands {
    /// List available built-in templates
    ///
    /// Shows all available templates with their descriptions and use cases.
    ///
    /// EXAMPLES:
    ///   xchecker template list
    ///
    /// Per FR-TEMPLATES (Requirements 4.7.1)
    List,

    /// Initialize a spec from a template
    ///
    /// Creates a new spec with predefined problem statement, configuration,
    /// and example partial spec flow based on the selected template.
    ///
    /// Available templates:
    ///   - fullstack-nextjs: Full-stack web applications with Next.js
    ///   - rust-microservice: Rust microservices and CLI tools
    ///   - python-fastapi: Python REST APIs with FastAPI
    ///   - docs-refactor: Documentation improvements and refactoring
    ///
    /// EXAMPLES:
    ///   xchecker template init fullstack-nextjs my-app
    ///   xchecker template init rust-microservice my-service
    ///   xchecker template init python-fastapi my-api
    ///   xchecker template init docs-refactor docs-v2
    ///
    /// Per FR-TEMPLATES (Requirements 4.7.2, 4.7.3)
    Init {
        /// Template to use (fullstack-nextjs, rust-microservice, python-fastapi, docs-refactor)
        template: String,

        /// Spec ID to create
        spec_id: String,
    },
}

/// Build the CLI command structure without parsing arguments
/// This is used for introspection in tests and documentation validation
#[must_use]
pub fn build_cli() -> clap::Command {
    <Cli as clap::CommandFactory>::command()
}
