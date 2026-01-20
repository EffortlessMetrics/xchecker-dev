//! Common helper functions used across CLI commands
//!
//! This module contains shared functionality for orchestrator configuration,
//! lockfile handling, and other utilities.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;

use crate::orchestrator::OrchestratorConfig;
use crate::redaction::SecretRedactor;
use crate::{CliArgs, Config, OrchestratorHandle};

/// Create default configuration from Config struct and CLI args
pub fn create_default_config(
    verbose: bool,
    config: &Config,
    cli_args: &CliArgs,
) -> HashMap<String, String> {
    let mut config_map = HashMap::new();

    if verbose {
        config_map.insert("verbose".to_string(), "true".to_string());
    }

    // Use values from the configuration system
    if let Some(packet_max_bytes) = config.defaults.packet_max_bytes {
        config_map.insert("packet_max_bytes".to_string(), packet_max_bytes.to_string());
    }

    if let Some(packet_max_lines) = config.defaults.packet_max_lines {
        config_map.insert("packet_max_lines".to_string(), packet_max_lines.to_string());
    }

    if let Some(max_turns) = config.defaults.max_turns {
        config_map.insert("max_turns".to_string(), max_turns.to_string());
    }

    if let Some(model) = &config.defaults.model {
        config_map.insert("model".to_string(), model.clone());
    }

    if let Some(output_format) = &config.defaults.output_format {
        config_map.insert("output_format".to_string(), output_format.clone());
    }

    // Add new CLI arguments (R7.2, R7.4, R9.2)
    if !cli_args.allow.is_empty() {
        config_map.insert("allowed_tools".to_string(), cli_args.allow.join(","));
    }

    if !cli_args.deny.is_empty() {
        config_map.insert("disallowed_tools".to_string(), cli_args.deny.join(","));
    }

    if cli_args.dangerously_skip_permissions {
        config_map.insert(
            "dangerously_skip_permissions".to_string(),
            "true".to_string(),
        );
    }

    if !cli_args.ignore_secret_pattern.is_empty() {
        config_map.insert(
            "ignore_secret_patterns".to_string(),
            cli_args.ignore_secret_pattern.join("|"),
        );
    }

    if !cli_args.extra_secret_pattern.is_empty() {
        config_map.insert(
            "extra_secret_patterns".to_string(),
            cli_args.extra_secret_pattern.join("|"),
        );
    }

    // Add debug_packet flag (FR-PKT-006, FR-PKT-007)
    if cli_args.debug_packet {
        config_map.insert("debug_packet".to_string(), "true".to_string());
    }

    config_map
}

/// Build an OrchestratorConfig from CLI parameters.
///
/// This helper reduces duplication between execute_spec_command and execute_resume_command
/// by combining create_default_config with the common additional parameters.
///
/// # Arguments
/// * `dry_run` - Whether to run in simulation mode
/// * `verbose` - Enable verbose logging
/// * `apply_fixups` - Whether to apply fixups (true) or preview (false)
/// * `config` - The loaded xchecker configuration
/// * `cli_args` - CLI arguments passed by the user
/// * `problem_statement` - Optional problem statement to include in phase prompts
pub fn build_orchestrator_config(
    dry_run: bool,
    verbose: bool,
    apply_fixups: bool,
    config: &Config,
    cli_args: &CliArgs,
    problem_statement: Option<&str>,
    redactor: Arc<SecretRedactor>,
) -> OrchestratorConfig {
    let mut config_map = create_default_config(verbose, config, cli_args);
    config_map.insert("logger_enabled".to_string(), verbose.to_string());
    config_map.insert("apply_fixups".to_string(), apply_fixups.to_string());

    // Include problem statement in config for prompt construction (FR-PKT)
    if let Some(ps) = problem_statement {
        config_map.insert("problem_statement".to_string(), ps.to_string());
    }

    OrchestratorConfig {
        dry_run,
        config: config_map,
        selectors: Some(config.selectors.clone()),
        strict_validation: config.strict_validation(),
        redactor,
        hooks: Some(config.hooks.clone()),
    }
}

/// Detect Claude CLI version by running `claude --version`
pub fn detect_claude_cli_version() -> Result<String> {
    use crate::runner::CommandSpec;

    let output = CommandSpec::new("claude")
        .arg("--version")
        .to_command()
        .output()
        .context("Failed to execute 'claude --version'")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "claude --version exited with non-zero status"
        ));
    }

    let version_str = String::from_utf8(output.stdout)
        .context("Failed to parse claude --version output as UTF-8")?;

    // Parse version from output (format: "claude 0.8.1" or similar)
    let version = version_str
        .split_whitespace()
        .last()
        .ok_or_else(|| anyhow::anyhow!("Failed to parse version from output"))?
        .to_string();

    Ok(version)
}

/// Check for lockfile drift and warn or fail based on `strict_lock` flag
pub fn check_lockfile_drift(
    spec_id: &str,
    strict_lock: bool,
    model_full_name: &str,
    claude_cli_version: &str,
) -> Result<Option<crate::types::LockDrift>> {
    use crate::lock::{RunContext, XCheckerLock};

    // Try to load lockfile
    let lock = match XCheckerLock::load(spec_id) {
        Ok(Some(lock)) => lock,
        Ok(None) => return Ok(None), // No lockfile, no drift
        Err(e) => {
            eprintln!("⚠ Warning: Failed to load lockfile: {e}");
            return Ok(None);
        }
    };

    // Create current run context
    let context = RunContext {
        model_full_name: model_full_name.to_string(),
        claude_cli_version: claude_cli_version.to_string(),
        schema_version: "1".to_string(),
    };

    // Detect drift
    if let Some(drift) = lock.detect_drift(&context) {
        // Print drift warning
        eprintln!("\n⚠ Lockfile drift detected for spec '{spec_id}':");

        if let Some(ref model_drift) = drift.model_full_name {
            eprintln!("  Model: {} → {}", model_drift.locked, model_drift.current);
        }

        if let Some(ref cli_drift) = drift.claude_cli_version {
            eprintln!("  Claude CLI: {} → {}", cli_drift.locked, cli_drift.current);
        }

        if let Some(ref schema_drift) = drift.schema_version {
            eprintln!(
                "  Schema: {} → {}",
                schema_drift.locked, schema_drift.current
            );
        }

        if strict_lock {
            eprintln!("\n✗ Strict lock mode enabled: failing due to drift");
            eprintln!("  To proceed, either:");
            eprintln!(
                "    - Update the lockfile: rm .xchecker/specs/{spec_id}/lock.json && xchecker init {spec_id} --create-lock"
            );
            eprintln!("    - Remove --strict-lock flag to allow drift with warning");

            return Err(anyhow::anyhow!("Lockfile drift detected in strict mode"));
        }
        eprintln!("\n  Continuing with drift (use --strict-lock to fail on drift)");

        Ok(Some(drift))
    } else {
        Ok(None)
    }
}

/// Generate next steps hint for resume JSON output
pub fn generate_next_steps_hint(
    spec_id: &str,
    phase_id: crate::types::PhaseId,
    current_inputs: &crate::types::CurrentInputs,
    _config: &Config,
) -> String {
    use crate::types::PhaseId;

    if !current_inputs.spec_exists {
        return format!(
            "Spec '{}' does not exist. Run 'xchecker spec {}' to create it first.",
            spec_id, spec_id
        );
    }

    // Check if we have the required inputs for this phase
    let has_requirements = current_inputs
        .available_artifacts
        .iter()
        .any(|a| a.contains("requirements"));
    let has_design = current_inputs
        .available_artifacts
        .iter()
        .any(|a| a.contains("design"));
    let has_tasks = current_inputs
        .available_artifacts
        .iter()
        .any(|a| a.contains("tasks"));
    let has_review = current_inputs
        .available_artifacts
        .iter()
        .any(|a| a.contains("review"));

    match phase_id {
        PhaseId::Requirements => {
            "Run requirements phase to generate initial requirements from the problem statement."
                .to_string()
        }
        PhaseId::Design => {
            if has_requirements {
                "Run design phase to generate architecture and design from requirements."
                    .to_string()
            } else {
                format!(
                    "Requirements phase not completed. Run 'xchecker resume {} --phase requirements' first.",
                    spec_id
                )
            }
        }
        PhaseId::Tasks => {
            if has_design {
                "Run tasks phase to generate implementation tasks from design.".to_string()
            } else {
                format!(
                    "Design phase not completed. Run 'xchecker resume {} --phase design' first.",
                    spec_id
                )
            }
        }
        PhaseId::Review => {
            if has_tasks {
                "Run review phase to review and validate the generated spec.".to_string()
            } else {
                format!(
                    "Tasks phase not completed. Run 'xchecker resume {} --phase tasks' first.",
                    spec_id
                )
            }
        }
        PhaseId::Fixup => {
            if has_review {
                "Run fixup phase to apply any suggested changes from review.".to_string()
            } else {
                format!(
                    "Review phase not completed. Run 'xchecker resume {} --phase review' first.",
                    spec_id
                )
            }
        }
        PhaseId::Final => "Run final phase to complete the spec generation workflow.".to_string(),
    }
}

/// Count pending fixups for a spec
/// Returns the number of target files with pending fixups
pub fn count_pending_fixups(handle: &OrchestratorHandle) -> u32 {
    crate::fixup::pending_fixups_from_handle(handle).targets
}

/// Count pending fixups for a spec by spec_id
pub fn count_pending_fixups_for_spec(spec_id: &str) -> u32 {
    crate::fixup::pending_fixups_for_spec(spec_id).targets
}
