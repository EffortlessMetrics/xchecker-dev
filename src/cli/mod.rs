//! Command-line interface for xchecker
//!
//! This module provides the CLI commands and argument parsing for the
//! xchecker tool, starting with basic spec generation functionality.
//!
//! ## Module Structure
//!
//! - `args`: CLI argument definitions and parsing structures (clap)
//! - `run`: Main entry point and command dispatch
//! - `commands`: Command implementations and helpers
//! - `tests`: Test module (cfg(test) only)

pub mod args;
mod commands;
mod run;

#[cfg(test)]
#[allow(clippy::items_after_test_module)] // Test helper functions defined after tests is intentional
#[allow(clippy::await_holding_lock)] // Test synchronization using mutex guards across awaits is intentional
mod tests;

// Re-export argument types
pub use args::{build_cli, Cli, Commands, ProjectCommands, TemplateCommands};

// Re-export run function
pub use run::run;

// Re-export types needed by other modules (for backwards compatibility if needed)
pub use crate::{CliArgs, Config};
