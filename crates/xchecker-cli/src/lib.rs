//! xchecker-cli - CLI interface for xchecker
//!
//! This crate provides the command-line interface for the xchecker tool,
//! including command parsing, argument handling, and CLI-specific logic.

// Re-export engine types for backward compatibility
pub use xchecker_engine::{
    CliArgs, Config, ExitCode, OrchestratorHandle, PhaseId, XCheckerError,
};

// Re-export orchestrator for direct use
pub use xchecker_orchestrator::OrchestratorHandle as OrchestratorHandleNew;

/// Main CLI entry point
///
/// This function parses command-line arguments and executes the appropriate command.
pub async fn run_cli() -> Result<ExitCode, anyhow::Error> {
    // CLI logic will be extracted from src/cli.rs
    // For now, this is a placeholder
    todo!("CLI extraction in progress - see src/cli.rs for current implementation")
}

/// Parse CLI arguments
///
/// This function parses command-line arguments and returns the parsed configuration.
pub fn parse_args() -> Result<CliArgs, anyhow::Error> {
    // CLI argument parsing will be extracted from src/cli.rs
    // For now, this is a placeholder
    todo!("CLI argument parsing extraction in progress - see src/cli.rs for current implementation")
}
