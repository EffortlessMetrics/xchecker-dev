//! CLI entry point and dispatch logic
//!
//! This module owns the `run()` function which:
//! - Parses CLI arguments
//! - Builds CliArgs and discovers Config
//! - Creates the tokio runtime
//! - Dispatches to command handlers
//! - Handles all error output (FR-CLI-3)

use anyhow::Result;
use clap::Parser;
use std::sync::Arc;

use super::args::{Cli, Commands};
use super::commands;

// Stable public API imports from crate root
// _Requirements: FR-CLI-2_
use crate::{CliArgs, Config, ExitCode, XCheckerError};

// Internal module imports (not part of stable public API)
use crate::error::ConfigError;
use crate::error_reporter::utils as error_utils;
use crate::redaction::SecretRedactor;
use crate::spec_id::sanitize_spec_id;

/// Main CLI execution function.
///
/// This function handles ALL output including errors. It returns `Result<(), ExitCode>`:
/// - On success: returns `Ok(())` after printing any output
/// - On error: prints error message via contextual reporting, returns `Err(ExitCode)`
///
/// main.rs only calls `std::process::exit(code.as_i32())` on error - it does NOT print.
///
/// _Requirements: FR-CLI-3, FR-CLI-4_
pub fn run() -> Result<(), ExitCode> {
    let cli = Cli::parse();

    // Build CLI args for configuration system (wired through build_cli)
    let cli_args = CliArgs {
        config_path: cli.config.clone(),
        model: cli.model.clone(),
        max_turns: cli.max_turns,
        packet_max_bytes: cli.packet_max_bytes,
        packet_max_lines: cli.packet_max_lines,
        output_format: cli.output_format.clone(),
        verbose: Some(cli.verbose),
        runner_mode: cli.runner_mode.clone(),
        runner_distro: cli.runner_distro.clone(),
        claude_path: cli.claude_path.clone(),
        allow: cli.allow.clone(),
        deny: cli.deny.clone(),
        dangerously_skip_permissions: cli.dangerously_skip_permissions,
        ignore_secret_pattern: cli.ignore_secret_pattern.clone(),
        extra_secret_pattern: cli.extra_secret_pattern.clone(),
        phase_timeout: cli.phase_timeout,
        stdout_cap_bytes: cli.stdout_cap_bytes,
        stderr_cap_bytes: cli.stderr_cap_bytes,
        lock_ttl_seconds: cli.lock_ttl_seconds,
        debug_packet: cli.debug_packet,
        allow_links: cli.allow_links,
        strict_validation: if cli.strict_validation {
            Some(true)
        } else if cli.no_strict_validation {
            Some(false)
        } else {
            None
        },
        llm_provider: cli.llm_provider.clone(),
        llm_claude_binary: cli.llm_claude_binary.clone(),
        llm_gemini_binary: None, // TODO: Add CLI flag for Gemini binary in future
        execution_strategy: cli.execution_strategy.clone(),
    };

    // Discover and load configuration
    let config = match Config::discover(&cli_args) {
        Ok(config) => config,
        Err(err) => {
            let contextual_report = error_utils::create_contextual_report(&err, "config");
            eprintln!("{contextual_report}");
            return Err(err.to_exit_code());
        }
    };

    // Build a configured redactor once from the effective config so all output surfaces
    // respect extra/ignore patterns (FR-SEC-19).
    let redactor = match SecretRedactor::from_config(&config) {
        Ok(redactor) => Arc::new(redactor),
        Err(e) => {
            let err = XCheckerError::Config(ConfigError::InvalidValue {
                key: "security".to_string(),
                value: e.to_string(),
            });
            let contextual_report = error_utils::create_contextual_report(&err, "config");
            eprintln!("{contextual_report}");
            return Err(err.to_exit_code());
        }
    };

    // Create tokio runtime for async operations
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("✗ Failed to create async runtime: {e}");
            return Err(ExitCode::INTERNAL);
        }
    };

    // Determine operation context for better error reporting before moving cli.command
    let operation = match &cli.command {
        Commands::Spec { .. } => "spec",
        Commands::Status { .. } => "status",
        Commands::Resume { .. } => "resume",
        Commands::Clean { .. } => "clean",
        Commands::Benchmark { .. } => "benchmark",
        Commands::Test { .. } => "test",
        Commands::Doctor { .. } => "doctor",
        Commands::Init { .. } => "init",
        Commands::Project(_) => "project",
        Commands::Gate { .. } => "gate",
        Commands::Template(_) => "template",
    };

    let result = rt.block_on(async {
        match cli.command {
            Commands::Spec {
                id,
                source,
                gh,
                repo,
                dry_run,
                force,
                apply_fixups,
                strict_lock,
                json,
            } => {
                // Sanitize spec ID (R5.7)
                let sanitized_id = sanitize_spec_id(&id).map_err(|e| {
                    XCheckerError::Config(ConfigError::InvalidValue {
                        key: "spec_id".to_string(),
                        value: format!("{e}"),
                    })
                })?;

                // If --json flag is set, output spec info as JSON and return
                if json {
                    return commands::execute_spec_json_command(&sanitized_id, &config);
                }

                commands::execute_spec_command(
                    &sanitized_id,
                    &source,
                    gh.as_deref(),
                    repo.as_deref(),
                    dry_run,
                    cli.verbose,
                    force,
                    apply_fixups,
                    strict_lock,
                    &config,
                    &cli_args,
                    &redactor,
                )
                .await
            }
            Commands::Status { id, json } => {
                // Sanitize spec ID (R5.7)
                let sanitized_id = sanitize_spec_id(&id).map_err(|e| {
                    XCheckerError::Config(ConfigError::InvalidValue {
                        key: "spec_id".to_string(),
                        value: format!("{e}"),
                    })
                })?;
                commands::execute_status_command(&sanitized_id, json, &config)
            }
            Commands::Resume {
                id,
                phase,
                dry_run,
                force,
                apply_fixups,
                strict_lock,
                json,
            } => {
                // Sanitize spec ID (R5.7)
                let sanitized_id = sanitize_spec_id(&id).map_err(|e| {
                    XCheckerError::Config(ConfigError::InvalidValue {
                        key: "spec_id".to_string(),
                        value: format!("{e}"),
                    })
                })?;

                // If --json flag is set, output resume info as JSON and return
                if json {
                    return commands::execute_resume_json_command(&sanitized_id, &phase, &config);
                }

                commands::execute_resume_command(
                    &sanitized_id,
                    &phase,
                    dry_run,
                    cli.verbose,
                    force,
                    apply_fixups,
                    strict_lock,
                    &config,
                    &cli_args,
                    &redactor,
                )
                .await
            }
            Commands::Clean { id, hard, force } => {
                // Sanitize spec ID (R5.7)
                let sanitized_id = sanitize_spec_id(&id).map_err(|e| {
                    XCheckerError::Config(ConfigError::InvalidValue {
                        key: "spec_id".to_string(),
                        value: format!("{e}"),
                    })
                })?;
                commands::execute_clean_command(&sanitized_id, hard, force, &config)
            }
            Commands::Benchmark {
                file_count,
                file_size,
                iterations,
                json,
                max_empty_run_secs,
                max_packetization_ms,
                max_rss_mb,
                max_commit_mb,
            } => commands::execute_benchmark_command(
                file_count,
                file_size,
                iterations,
                json,
                max_empty_run_secs,
                max_packetization_ms,
                max_rss_mb,
                max_commit_mb,
                cli.verbose,
            ),
            Commands::Test { components, smoke } => {
                commands::execute_test_command(components, smoke, cli.verbose)
            }
            Commands::Doctor { json, strict_exit } => {
                commands::execute_doctor_command(json, strict_exit, &config)
            }
            Commands::Init { id, create_lock } => {
                // Sanitize spec ID (R5.7)
                let sanitized_id = sanitize_spec_id(&id).map_err(|e| {
                    XCheckerError::Config(ConfigError::InvalidValue {
                        key: "spec_id".to_string(),
                        value: format!("{e}"),
                    })
                })?;
                commands::execute_init_command(&sanitized_id, create_lock, &config)
            }
            Commands::Project(project_cmd) => commands::execute_project_command(project_cmd),
            Commands::Gate {
                id,
                min_phase,
                fail_on_pending_fixups,
                max_phase_age,
                json,
            } => {
                // Sanitize spec ID (R5.7)
                let sanitized_id = sanitize_spec_id(&id).map_err(|e| {
                    XCheckerError::Config(ConfigError::InvalidValue {
                        key: "spec_id".to_string(),
                        value: format!("{e}"),
                    })
                })?;
                commands::execute_gate_command(
                    &sanitized_id,
                    &min_phase,
                    fail_on_pending_fixups,
                    max_phase_age.as_deref(),
                    json,
                )
            }
            Commands::Template(template_cmd) => commands::execute_template_command(template_cmd),
        }
    });

    // Handle errors with structured reporting (R1.3, R6.4, R6.8, R6.9)
    // cli::run() handles ALL output including errors (FR-CLI-3, FR-CLI-6)
    // Error messages are displayed via contextual reporting which extends display_for_user()
    // with operation-specific context for better user experience
    if let Err(error) = result {
        // Try to downcast to XCheckerError for better reporting
        if let Some(xchecker_error) = error.downcast_ref::<XCheckerError>() {
            // Use contextual error reporting which builds on display_for_user() (FR-CLI-3)
            // This provides user_message(), context(), suggestions() plus operation-specific help
            let contextual_report = error_utils::create_contextual_report_with_redactor(
                xchecker_error,
                operation,
                redactor.as_ref(),
            );
            eprintln!("{contextual_report}");

            // Return the appropriate exit code - main.rs will call std::process::exit()
            return Err(xchecker_error.to_exit_code());
        } else {
            // Fallback for other error types with enhanced context
            let redacted_error = redactor.redact_string(&error.to_string());
            eprintln!("✗ Unexpected error: {redacted_error}");

            // Provide enhanced context and suggestions for common anyhow errors
            if let Some(suggestions) = enhance_error_context(&error) {
                eprintln!("\n  Suggestions:");
                for (i, suggestion) in suggestions.iter().enumerate() {
                    eprintln!("    {}. {}", i + 1, suggestion);
                }
            }

            // Provide general troubleshooting steps
            eprintln!("\n  General troubleshooting:");
            eprintln!("    - Run with --verbose for more detailed output");
            eprintln!("    - Check the xchecker documentation for common issues");
            eprintln!("    - Ensure all dependencies are properly installed");

            return Err(ExitCode::INTERNAL);
        }
    }

    Ok(())
}

/// Enhance error reporting for common failure scenarios
fn enhance_error_context(error: &anyhow::Error) -> Option<Vec<String>> {
    let error_str = error.to_string();

    if error_str.contains("Failed to create orchestrator") {
        Some(vec![
            "Check that the current directory is writable".to_string(),
            "Ensure sufficient disk space is available".to_string(),
            "Verify directory permissions".to_string(),
            "Try running from a different directory".to_string(),
        ])
    } else if error_str.contains("Failed to execute") {
        Some(vec![
            "Check the spec ID is valid and doesn't contain special characters".to_string(),
            "Verify Claude CLI is installed and accessible".to_string(),
            "Try running with --dry-run to test configuration".to_string(),
            "Check your internet connection if using Claude API".to_string(),
        ])
    } else if error_str.contains("Permission denied") {
        Some(vec![
            "Check file and directory permissions".to_string(),
            "Ensure you have write access to the current directory".to_string(),
            "Try running from your home directory or a writable location".to_string(),
        ])
    } else if error_str.contains("No such file or directory") {
        Some(vec![
            "Verify the specified paths exist".to_string(),
            "Check that you're running from the correct directory".to_string(),
            "Ensure all required files are present".to_string(),
        ])
    } else {
        None
    }
}
