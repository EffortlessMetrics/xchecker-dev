//! Spec command implementation
//!
//! Handles `xchecker spec` and `xchecker spec --json` commands.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;

use super::common::{build_orchestrator_config, check_lockfile_drift, detect_claude_cli_version};
use super::json_emit::emit_spec_json;

use crate::atomic_write::write_file_atomic;
use crate::error::{ConfigError, PhaseError};
use crate::error_reporter::ErrorReport;
use crate::logging::Logger;
use crate::redaction::SecretRedactor;
use crate::source::SourceResolver;
use crate::{CliArgs, Config, OrchestratorHandle, PhaseId, XCheckerError};

/// Execute the spec generation command
#[allow(clippy::too_many_arguments)]
pub async fn execute_spec_command(
    spec_id: &str,
    source_type: &str,
    gh_repo: Option<&str>,
    fs_repo: Option<&str>,
    dry_run: bool,
    verbose: bool,
    force: bool,
    apply_fixups: bool,
    strict_lock: bool,
    config: &Config,
    cli_args: &CliArgs,
    redactor: &Arc<SecretRedactor>,
) -> Result<()> {
    // Create logger for verbose output and timing (R7.5, NFR5)
    let mut logger = Logger::new(verbose);
    logger.start_timing("total_execution");

    logger.verbose(&format!("Starting spec generation for ID: {spec_id}"));
    if dry_run {
        logger.verbose("Running in dry-run mode (no Claude calls will be made)");
    }

    // Resolve source input (R6.4)
    logger.start_timing("source_resolution");
    let source_content = match source_type {
        "gh" => {
            let gh_repo = gh_repo.ok_or_else(|| {
                XCheckerError::Config(ConfigError::MissingRequired("--gh owner/repo".to_string()))
            })?;

            let parts: Vec<&str> = gh_repo.split('/').collect();
            if parts.len() != 2 {
                return Err(XCheckerError::Config(ConfigError::InvalidValue {
                    key: "gh".to_string(),
                    value: gh_repo.to_string(),
                })
                .into());
            }

            SourceResolver::resolve_github(parts[0], parts[1], spec_id).map_err(|_e| {
                // Convert SourceError to XCheckerError for consistent reporting (R6.4)
                XCheckerError::Source(crate::error::SourceError::GitHubRepoNotFound {
                    owner: parts[0].to_string(),
                    repo: parts[1].to_string(),
                })
            })?
        }
        "fs" => {
            let fs_repo = fs_repo.ok_or_else(|| {
                XCheckerError::Config(ConfigError::MissingRequired("--repo <path>".to_string()))
            })?;

            let path = PathBuf::from(fs_repo);
            SourceResolver::resolve_filesystem(&path).map_err(|e| {
                // Enhanced error reporting for filesystem source resolution (R6.4)
                if path.exists() {
                    if path.is_dir() {
                        // Check if it's a permission issue or other access problem
                        match std::fs::read_dir(&path) {
                            Err(io_err)
                                if io_err.kind() == std::io::ErrorKind::PermissionDenied =>
                            {
                                XCheckerError::Source(
                                    crate::error::SourceError::FileSystemAccessDenied {
                                        path: fs_repo.to_string(),
                                    },
                                )
                            }
                            Err(_) => {
                                XCheckerError::Source(crate::error::SourceError::InvalidFormat {
                                    reason: format!("Directory exists but cannot be read: {e}"),
                                })
                            }
                            Ok(_) => {
                                // Directory is readable, so it's some other issue
                                XCheckerError::Source(crate::error::SourceError::InvalidFormat {
                                    reason: format!("Failed to resolve filesystem source: {e}"),
                                })
                            }
                        }
                    } else {
                        XCheckerError::Source(crate::error::SourceError::FileSystemNotDirectory {
                            path: fs_repo.to_string(),
                        })
                    }
                } else {
                    XCheckerError::Source(crate::error::SourceError::FileSystemNotFound {
                        path: fs_repo.to_string(),
                    })
                }
            })?
        }
        "stdin" => {
            SourceResolver::resolve_stdin().map_err(|e| {
                // Enhanced error reporting for stdin source resolution (R6.4)
                let error_msg = e.to_string();
                if error_msg.contains("empty") || error_msg.contains("EOF") {
                    XCheckerError::Source(crate::error::SourceError::EmptyInput)
                } else if error_msg.contains("permission") || error_msg.contains("access") {
                    XCheckerError::Source(crate::error::SourceError::StdinReadFailed {
                        reason: "Permission denied or stdin not accessible".to_string(),
                    })
                } else {
                    XCheckerError::Source(crate::error::SourceError::StdinReadFailed {
                        reason: error_msg,
                    })
                }
            })?
        }
        _ => {
            return Err(XCheckerError::Config(ConfigError::InvalidValue {
                key: "source".to_string(),
                value: format!("Unknown source type '{source_type}'. Valid options: 'gh' (GitHub), 'fs' (filesystem), 'stdin' (standard input)"),
            }).into());
        }
    };
    logger.end_timing("source_resolution");

    // Extract problem statement from resolved source
    let problem_statement = source_content.content.clone();
    logger.verbose(&format!("Source resolved successfully from: {source_type}"));

    // Persist problem statement to spec directory (FR-PKT: problem statement in packet)
    // This ensures the problem statement is available for packet building
    let spec_root = crate::paths::spec_root(spec_id);
    let source_dir = spec_root.join("source");
    crate::paths::ensure_dir_all(&source_dir)
        .with_context(|| format!("Failed to create source directory: {}", source_dir))?;

    let problem_path = source_dir.join("00-problem-statement.md");
    write_file_atomic(
        &problem_path,
        &format!("# Problem Statement\n\n{}\n", problem_statement.trim()),
    )
    .with_context(|| format!("Failed to write problem statement: {}", problem_path))?;

    logger.verbose(&format!("Problem statement written to: {}", problem_path));

    // Check for lockfile drift (R10.2, R10.4)
    let model_full_name = config.defaults.model.as_deref().unwrap_or("haiku");
    let claude_cli_version = detect_claude_cli_version().unwrap_or_else(|_| "unknown".to_string());
    let _lock_drift =
        check_lockfile_drift(spec_id, strict_lock, model_full_name, &claude_cli_version)?;

    // Configure execution using shared helper, passing problem statement for prompt construction
    let orchestrator_config = build_orchestrator_config(
        dry_run,
        verbose,
        apply_fixups,
        config,
        cli_args,
        Some(&problem_statement),
        redactor.clone(),
    );

    // Create orchestrator handle (this will acquire the file lock)
    logger.start_timing("orchestrator_setup");
    let mut handle = OrchestratorHandle::with_config_and_force(spec_id, orchestrator_config, force)
        .with_context(|| format!("Failed to create orchestrator for spec: {spec_id}"))?;
    logger.end_timing("orchestrator_setup");

    logger.verbose("Executing Requirements phase...");

    // Execute Requirements phase
    logger.start_timing("requirements_phase");
    let result = handle
        .run_phase(PhaseId::Requirements)
        .await
        .with_context(|| "Failed to execute Requirements phase")?;
    logger.end_timing("requirements_phase");

    // Report results
    logger.end_timing("total_execution");

    if result.success {
        println!("✓ Requirements phase completed successfully");

        logger.verbose(&format!("Phase: {}", result.phase.as_str()));
        logger.verbose(&format!("Exit code: {}", result.exit_code));
        logger.verbose(&format!(
            "Artifacts created: {}",
            result.artifact_paths.len()
        ));

        for (i, path) in result.artifact_paths.iter().enumerate() {
            logger.verbose(&format!("  {}: {}", i + 1, path.display()));
        }

        if let Some(receipt_path) = &result.receipt_path {
            logger.verbose(&format!("Receipt: {}", receipt_path.display()));
        }

        // Print performance summary if verbose (R7.5, NFR5)
        logger.print_performance_summary();

        // Show next steps
        println!("\nNext steps:");
        println!("  - Review the generated requirements in .xchecker/specs/{spec_id}/artifacts/");
        println!("  - Check status with: xchecker status {spec_id}");
        println!("  - Continue to Design phase: xchecker resume {spec_id} --phase design");
    } else {
        // Create structured error for phase failure (R1.3, R4.3)
        let phase_error = PhaseError::ExecutionFailed {
            phase: result.phase.as_str().to_string(),
            code: result.exit_code,
        };
        let xchecker_error = XCheckerError::Phase(phase_error);

        // Report with full context and suggestions
        let report = ErrorReport::new(&xchecker_error);
        eprintln!("{}", report.format_with_redactor(redactor.as_ref()));

        // Enhanced error reporting for phase failures (R1.3, R4.3)
        if let Some(error_msg) = &result.error {
            let redacted_error_msg = redactor.redact_string(error_msg);
            eprintln!("\n  Phase failure details: {redacted_error_msg}");

            // Try to parse the error for enhanced reporting
            if error_msg.contains("ExecutionFailedWithStderr") {
                eprintln!("  ↳ Claude CLI produced error output (see receipt for full stderr)");
            } else if error_msg.contains("PartialOutputSaved") {
                eprintln!("  ↳ Partial output was saved for debugging");
            }
        }

        // Show partial artifacts location (R4.3)
        eprintln!("\n  Debugging information:");
        if !result.artifact_paths.is_empty() {
            eprintln!("    Partial artifacts:");
            for path in &result.artifact_paths {
                eprintln!("      - {}", path.display());
            }
        }
        eprintln!("    Spec directory: .xchecker/specs/{spec_id}/");

        if let Some(receipt_path) = &result.receipt_path {
            eprintln!("    Execution receipt: {}", receipt_path.display());
            eprintln!("      ↳ Contains stderr output, warnings, and execution metadata");
        }

        // Provide recovery suggestions
        eprintln!("\n  Recovery options:");
        eprintln!("    - Review partial outputs and receipt for error details");
        eprintln!("    - Fix any configuration or connectivity issues");
        eprintln!("    - Retry with: xchecker spec {spec_id}");
        eprintln!("    - Test configuration with: xchecker spec {spec_id} --dry-run");

        std::process::exit(result.exit_code);
    }

    Ok(())
}

/// Execute the spec --json command (FR-Claude Code-CLI: Claude Code CLI Surfaces)
/// Returns JSON with schema_version, spec_id, phases, config_summary
/// Excludes full artifacts and packet contents per Requirements 4.1.1, 4.1.4
pub fn execute_spec_json_command(spec_id: &str, config: &Config) -> Result<()> {
    use crate::types::{PhaseId, PhaseInfo, SpecConfigSummary, SpecOutput};

    // Create read-only handle to access managers (no lock needed for JSON output)
    let handle = OrchestratorHandle::readonly(spec_id)
        .with_context(|| format!("Failed to create orchestrator for spec: {spec_id}"))?;

    // Check if spec directory exists
    let base_path = handle.artifact_manager().base_path();
    if !base_path.exists() {
        // Return minimal JSON for non-existent spec
        let output = SpecOutput {
            schema_version: "spec-json.v1".to_string(),
            spec_id: spec_id.to_string(),
            phases: vec![],
            config_summary: SpecConfigSummary {
                execution_strategy: config
                    .llm
                    .execution_strategy
                    .clone()
                    .unwrap_or_else(|| "controlled".to_string()),
                provider: config.llm.provider.clone(),
                spec_path: base_path.to_string(),
            },
        };
        let json_output = emit_spec_json(&output)?;
        println!("{json_output}");
        return Ok(());
    }

    // Get phase information
    let all_phases = [
        PhaseId::Requirements,
        PhaseId::Design,
        PhaseId::Tasks,
        PhaseId::Review,
        PhaseId::Fixup,
        PhaseId::Final,
    ];

    // Get receipts to determine phase status and last run times
    // Handle case where receipts directory doesn't exist yet
    let receipts = handle.receipt_manager().list_receipts().unwrap_or_default();

    // Build phase info list
    let mut phases = Vec::new();
    for phase_id in &all_phases {
        let phase_completed = handle.artifact_manager().phase_completed(*phase_id);

        // Find the latest receipt for this phase
        let latest_receipt = receipts
            .iter()
            .filter(|r| r.phase == phase_id.as_str())
            .max_by_key(|r| r.emitted_at);

        let status = if phase_completed {
            "completed".to_string()
        } else if latest_receipt.is_some() {
            "pending".to_string()
        } else {
            "not_started".to_string()
        };

        let last_run = latest_receipt.map(|r| r.emitted_at);

        phases.push(PhaseInfo {
            phase_id: phase_id.as_str().to_string(),
            status,
            last_run,
        });
    }

    // Build config summary (excludes full artifacts and packet contents)
    let config_summary = SpecConfigSummary {
        execution_strategy: config
            .llm
            .execution_strategy
            .clone()
            .unwrap_or_else(|| "controlled".to_string()),
        provider: config.llm.provider.clone(),
        spec_path: base_path.to_string(),
    };

    let output = SpecOutput {
        schema_version: "spec-json.v1".to_string(),
        spec_id: spec_id.to_string(),
        phases,
        config_summary,
    };

    let json_output = emit_spec_json(&output)?;
    println!("{json_output}");

    Ok(())
}
