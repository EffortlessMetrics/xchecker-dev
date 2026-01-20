//! Resume command implementation
//!
//! Handles `xchecker resume` and `xchecker resume --json` commands.

use anyhow::{Context, Result};
use std::sync::Arc;

use super::common::{
    build_orchestrator_config, check_lockfile_drift, detect_claude_cli_version,
    generate_next_steps_hint,
};
use super::json_emit::emit_resume_json;

use crate::error::{ConfigError, PhaseError};
use crate::error_reporter::ErrorReport;
use crate::logging::Logger;
use crate::redaction::SecretRedactor;
use crate::{CliArgs, Config, OrchestratorHandle, PhaseId, XCheckerError};

/// Execute the resume --json command (FR-Claude Code-CLI: Claude Code CLI Surfaces)
/// Returns JSON with schema_version, spec_id, phase, current_inputs, next_steps
/// Excludes full packet and raw artifacts per Requirements 4.1.3, 4.1.4
pub fn execute_resume_json_command(spec_id: &str, phase_name: &str, config: &Config) -> Result<()> {
    use crate::types::{CurrentInputs, PhaseId, ResumeJsonOutput};

    // Parse phase name
    let phase_id = match phase_name.to_lowercase().as_str() {
        "requirements" => PhaseId::Requirements,
        "design" => PhaseId::Design,
        "tasks" => PhaseId::Tasks,
        "review" => PhaseId::Review,
        "fixup" => PhaseId::Fixup,
        "final" => PhaseId::Final,
        _ => {
            return Err(XCheckerError::Config(ConfigError::InvalidValue {
                key: "phase".to_string(),
                value: format!("Unknown phase '{phase_name}'. Valid phases: requirements, design, tasks, review, fixup, final"),
            }).into());
        }
    };

    // Create read-only handle to access managers (no lock needed for JSON output)
    let handle = OrchestratorHandle::readonly(spec_id)
        .with_context(|| format!("Failed to create orchestrator for spec: {spec_id}"))?;

    // Check if spec directory exists
    let base_path = handle.artifact_manager().base_path();
    let spec_exists = base_path.exists();

    // Get available artifacts (names only, not contents)
    let available_artifacts = if spec_exists {
        handle
            .artifact_manager()
            .list_artifacts()
            .unwrap_or_default()
    } else {
        vec![]
    };

    // Get latest completed phase
    let latest_completed_phase = if spec_exists {
        handle
            .artifact_manager()
            .get_latest_completed_phase()
            .map(|p| p.as_str().to_string())
    } else {
        None
    };

    // Build current inputs (high-level metadata only, no full contents)
    let current_inputs = CurrentInputs {
        available_artifacts,
        spec_exists,
        latest_completed_phase,
    };

    // Generate next steps hint based on phase and current state
    let next_steps = generate_next_steps_hint(spec_id, phase_id, &current_inputs, config);

    let output = ResumeJsonOutput {
        schema_version: "resume-json.v1".to_string(),
        spec_id: spec_id.to_string(),
        phase: phase_id.as_str().to_string(),
        current_inputs,
        next_steps,
    };

    let json_output = emit_resume_json(&output)?;
    println!("{json_output}");

    Ok(())
}

/// Execute the resume command
#[allow(clippy::too_many_arguments)]
pub async fn execute_resume_command(
    spec_id: &str,
    phase_name: &str,
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

    // Parse phase name
    let phase_id = match phase_name.to_lowercase().as_str() {
        "requirements" => PhaseId::Requirements,
        "design" => PhaseId::Design,
        "tasks" => PhaseId::Tasks,
        "review" => PhaseId::Review,
        "fixup" => PhaseId::Fixup,
        "final" => PhaseId::Final,
        _ => {
            return Err(XCheckerError::Config(ConfigError::InvalidValue {
                key: "phase".to_string(),
                value: format!("Unknown phase '{phase_name}'. Valid phases: requirements, design, tasks, review, fixup, final"),
            }).into());
        }
    };

    logger.verbose(&format!(
        "Resuming spec {} from {} phase",
        spec_id,
        phase_id.as_str()
    ));
    if dry_run {
        logger.verbose("Running in dry-run mode (no Claude calls will be made)");
    }

    // Check for lockfile drift (R10.2, R10.4)
    let model_full_name = config.defaults.model.as_deref().unwrap_or("haiku");
    let claude_cli_version = detect_claude_cli_version().unwrap_or_else(|_| "unknown".to_string());
    let _lock_drift =
        check_lockfile_drift(spec_id, strict_lock, model_full_name, &claude_cli_version)?;

    // Configure execution using shared helper
    // Note: Problem statement is not passed for resume - it's already persisted in spec dir
    let orchestrator_config = build_orchestrator_config(
        dry_run,
        verbose,
        apply_fixups,
        config,
        cli_args,
        None,
        redactor.clone(),
    );

    // Create orchestrator handle (this will acquire the file lock)
    logger.start_timing("orchestrator_setup");
    let mut handle = OrchestratorHandle::with_config_and_force(spec_id, orchestrator_config, force)
        .with_context(|| format!("Failed to create orchestrator for spec: {spec_id}"))?;
    logger.end_timing("orchestrator_setup");

    // Check if spec exists
    let base_path = handle.artifact_manager().base_path();
    if !base_path.exists() {
        return Err(XCheckerError::Config(ConfigError::NotFound {
            path: format!("Spec directory: {base_path}"),
        })
        .into());
    }

    logger.verbose(&format!(
        "Checking dependencies for {} phase...",
        phase_id.as_str()
    ));

    // Execute resume
    logger.start_timing(&format!("{}_phase", phase_id.as_str()));
    let result = handle
        .run_phase(phase_id)
        .await
        .with_context(|| format!("Failed to resume {} phase", phase_id.as_str()))?;
    logger.end_timing(&format!("{}_phase", phase_id.as_str()));

    // Report results
    logger.end_timing("total_execution");

    if result.success {
        println!("✓ {} phase completed successfully", phase_id.as_str());

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

        // Show next steps based on completed phase
        println!("\nNext steps:");
        match phase_id {
            PhaseId::Requirements => {
                println!(
                    "  - Review the generated requirements in .xchecker/specs/{spec_id}/artifacts/"
                );
                println!("  - Continue to Design phase: xchecker resume {spec_id} --phase design");
            }
            PhaseId::Design => {
                println!("  - Review the generated design in .xchecker/specs/{spec_id}/artifacts/");
                println!("  - Continue to Tasks phase: xchecker resume {spec_id} --phase tasks");
            }
            PhaseId::Tasks => {
                println!("  - Review the generated tasks in .xchecker/specs/{spec_id}/artifacts/");
                println!("  - Continue to Review phase: xchecker resume {spec_id} --phase review");
            }
            _ => {
                println!("  - Check status with: xchecker status {spec_id}");
            }
        }
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
        }

        // Provide recovery suggestions
        eprintln!("\n  Recovery options:");
        eprintln!("    - Review partial outputs and receipt for error details");
        eprintln!("    - Fix any configuration or connectivity issues");
        eprintln!("    - Retry with: xchecker resume {spec_id} --phase {phase_name}");
        eprintln!(
            "    - Test configuration with: xchecker resume {spec_id} --phase {phase_name} --dry-run"
        );

        std::process::exit(result.exit_code);
    }

    Ok(())
}
