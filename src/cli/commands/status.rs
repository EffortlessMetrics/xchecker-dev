//! Status command implementation
//!
//! Handles `xchecker status` and `xchecker status --json` commands.

use anyhow::{Context, Result};
use std::collections::HashMap;

use super::common::count_pending_fixups;
use super::json_emit::emit_status_json;

use crate::{Config, OrchestratorHandle, PhaseId};

/// Execute the status command
pub fn execute_status_command(spec_id: &str, json: bool, config: &Config) -> Result<()> {
    // Create read-only handle to access managers (no lock needed for status)
    let handle = OrchestratorHandle::readonly(spec_id)
        .with_context(|| format!("Failed to create orchestrator for spec: {spec_id}"))?;

    // Check if spec directory exists
    let base_path = handle.artifact_manager().base_path();
    if !base_path.exists() {
        if json {
            // Return empty JSON for non-existent spec
            println!("{{}}");
        } else {
            println!("Status for spec: {spec_id}");
            println!("  Status: No spec found");
            println!("  Directory: {base_path} (does not exist)");
        }
        return Ok(());
    }

    // If JSON output is requested, use status-json.v2 format with full details
    // Includes artifacts with blake3_first8, effective_config, and lock_drift
    if json {
        use crate::lock::{RunContext, XCheckerLock};
        use crate::types::{
            ArtifactInfo, ConfigSource, ConfigValue, PhaseStatusInfo, StatusJsonOutput,
        };
        use std::collections::BTreeMap;

        // Get all phases
        let all_phases = [
            PhaseId::Requirements,
            PhaseId::Design,
            PhaseId::Tasks,
            PhaseId::Review,
            PhaseId::Fixup,
            PhaseId::Final,
        ];

        // Get receipts to determine phase status and receipt IDs
        let receipts = handle.receipt_manager().list_receipts().unwrap_or_default();

        // Build phase status list
        let mut phase_statuses = Vec::new();
        let mut has_errors = false;

        for phase_id in &all_phases {
            // Find the latest receipt for this phase
            let latest_receipt = receipts
                .iter()
                .filter(|r| r.phase == phase_id.as_str())
                .max_by_key(|r| r.emitted_at);

            let (status, receipt_id) = if let Some(receipt) = latest_receipt {
                // Check if the phase succeeded or failed
                if receipt.exit_code == 0 {
                    (
                        "success".to_string(),
                        Some(format!(
                            "{}-{}",
                            receipt.phase,
                            receipt.emitted_at.format("%Y%m%d_%H%M%S")
                        )),
                    )
                } else {
                    has_errors = true;
                    (
                        "failed".to_string(),
                        Some(format!(
                            "{}-{}",
                            receipt.phase,
                            receipt.emitted_at.format("%Y%m%d_%H%M%S")
                        )),
                    )
                }
            } else {
                ("not_started".to_string(), None)
            };

            phase_statuses.push(PhaseStatusInfo {
                phase_id: phase_id.as_str().to_string(),
                status,
                receipt_id,
            });
        }

        // Count pending fixups
        let pending_fixups = count_pending_fixups(&handle);

        // Collect artifacts with blake3_first8 from receipts
        let mut artifact_hashes: BTreeMap<String, String> = BTreeMap::new();
        for receipt in &receipts {
            for output in &receipt.outputs {
                // Extract just the filename from the path for matching
                if let Some(filename) = output.path.split('/').next_back() {
                    let short_hash = if output.blake3_canonicalized.len() >= 8 {
                        &output.blake3_canonicalized[..8]
                    } else {
                        &output.blake3_canonicalized
                    };
                    artifact_hashes.insert(filename.to_string(), short_hash.to_string());
                }
            }
        }

        // Build artifact info list
        let artifact_files = handle
            .artifact_manager()
            .list_artifacts()
            .unwrap_or_default();

        let mut artifacts: Vec<ArtifactInfo> = artifact_files
            .iter()
            .filter_map(|filename| {
                artifact_hashes.get(filename).map(|hash| ArtifactInfo {
                    path: format!("artifacts/{filename}"),
                    blake3_first8: hash.clone(),
                })
            })
            .collect();
        artifacts.sort_by(|a, b| a.path.cmp(&b.path));

        // Build effective_config from config with source attribution
        let mut effective_config: BTreeMap<String, ConfigValue> = BTreeMap::new();

        // Add key configuration values with their sources
        // Provider
        if let Some(ref provider) = config.llm.provider {
            let source = config
                .source_attribution
                .get("provider")
                .cloned()
                .unwrap_or(ConfigSource::Config);
            effective_config.insert(
                "provider".to_string(),
                ConfigValue {
                    value: serde_json::Value::String(provider.clone()),
                    source,
                },
            );
        }

        // Model
        if let Some(ref model) = config.defaults.model {
            let source = config
                .source_attribution
                .get("model")
                .cloned()
                .unwrap_or(ConfigSource::Config);
            effective_config.insert(
                "model".to_string(),
                ConfigValue {
                    value: serde_json::Value::String(model.clone()),
                    source,
                },
            );
        }

        // Max turns
        if let Some(max_turns) = config.defaults.max_turns {
            let source = config
                .source_attribution
                .get("max_turns")
                .cloned()
                .unwrap_or(ConfigSource::Config);
            effective_config.insert(
                "max_turns".to_string(),
                ConfigValue {
                    value: serde_json::Value::Number(max_turns.into()),
                    source,
                },
            );
        }

        // Phase timeout
        if let Some(timeout) = config.defaults.phase_timeout {
            let source = config
                .source_attribution
                .get("phase_timeout")
                .cloned()
                .unwrap_or(ConfigSource::Config);
            effective_config.insert(
                "phase_timeout".to_string(),
                ConfigValue {
                    value: serde_json::Value::Number(timeout.into()),
                    source,
                },
            );
        }

        // Execution strategy
        if let Some(ref strategy) = config.llm.execution_strategy {
            let source = config
                .source_attribution
                .get("execution_strategy")
                .cloned()
                .unwrap_or(ConfigSource::Config);
            effective_config.insert(
                "execution_strategy".to_string(),
                ConfigValue {
                    value: serde_json::Value::String(strategy.clone()),
                    source,
                },
            );
        }

        // Load lockfile and detect drift
        let lock_drift = if let Ok(Some(lock)) = XCheckerLock::load(spec_id) {
            // Get current run context from latest receipt or config
            let model_full_name = receipts
                .last()
                .map(|r| r.model_full_name.clone())
                .unwrap_or_else(|| config.defaults.model.clone().unwrap_or_default());

            let claude_cli_version = receipts
                .last()
                .map(|r| r.claude_cli_version.clone())
                .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());

            let context = RunContext {
                model_full_name,
                claude_cli_version,
                schema_version: "1".to_string(),
            };

            lock.detect_drift(&context)
        } else {
            None
        };

        let output = StatusJsonOutput {
            schema_version: "status-json.v2".to_string(),
            spec_id: spec_id.to_string(),
            phase_statuses,
            pending_fixups,
            has_errors,
            strict_validation: config.strict_validation(),
            artifacts,
            effective_config,
            lock_drift,
        };

        // Emit as canonical JSON using JCS (RFC 8785)
        let json_output =
            emit_status_json(&output).with_context(|| "Failed to emit status JSON")?;

        println!("{json_output}");
        return Ok(());
    }

    // Human-readable output
    println!("Status for spec: {spec_id}");
    println!("  Directory: {base_path}");

    // Get latest completed phase and show phase progression (R2.6)
    let latest_completed = handle.artifact_manager().get_latest_completed_phase();
    match latest_completed {
        Some(phase) => {
            println!("  Latest completed phase: {}", phase.as_str());
        }
        None => {
            println!("  Latest completed phase: None");
        }
    }

    // List artifacts with first-8 BLAKE3 hashes (R2.6, R8.1)
    let artifacts = handle
        .artifact_manager()
        .list_artifacts()
        .with_context(|| "Failed to list artifacts")?;

    if artifacts.is_empty() {
        println!("  Artifacts: None");
    } else {
        println!("  Artifacts: {} found", artifacts.len());

        // Get receipts to extract hashes for artifacts
        let receipts = handle
            .receipt_manager()
            .list_receipts()
            .with_context(|| "Failed to list receipts")?;

        // Create a map of artifact paths to their hashes from receipts
        let mut artifact_hashes: HashMap<String, String> = HashMap::new();
        for receipt in &receipts {
            for output in &receipt.outputs {
                // Extract just the filename from the path for matching
                if let Some(filename) = output.path.split('/').next_back() {
                    let short_hash = if output.blake3_canonicalized.len() >= 8 {
                        &output.blake3_canonicalized[..8]
                    } else {
                        &output.blake3_canonicalized
                    };
                    artifact_hashes.insert(filename.to_string(), short_hash.to_string());
                }
            }
        }

        for artifact in &artifacts {
            if let Some(hash) = artifact_hashes.get(artifact) {
                println!("    - {artifact} -> {hash}");
            } else {
                println!("    - {artifact} -> <no hash>");
            }
        }
    }

    // Display last receipt path and key information (R2.6, R8.2)
    let receipts = handle
        .receipt_manager()
        .list_receipts()
        .with_context(|| "Failed to list receipts")?;

    if receipts.is_empty() {
        println!("  Last receipt: None");
    } else {
        let latest_receipt = receipts.last().unwrap();

        // Show receipt path
        let receipt_filename = format!(
            "{}-{}.json",
            latest_receipt.phase,
            latest_receipt.emitted_at.format("%Y%m%d_%H%M%S")
        );
        let receipt_path = base_path.join("receipts").join(receipt_filename);
        println!("  Last receipt: {receipt_path}");

        // Show key receipt information
        println!("    Phase: {}", latest_receipt.phase);
        println!(
            "    Emitted at: {}",
            latest_receipt.emitted_at.format("%Y-%m-%d %H:%M:%S UTC")
        );
        println!("    Exit code: {}", latest_receipt.exit_code);
        println!("    Model: {}", latest_receipt.model_full_name);
        if let Some(alias) = &latest_receipt.model_alias {
            println!("    Model alias: {alias}");
        }
        println!("    Runner: {}", latest_receipt.runner);
        if let Some(distro) = &latest_receipt.runner_distro {
            println!("    Runner distro: {distro}");
        }
        println!(
            "    Canonicalization: {}",
            latest_receipt.canonicalization_version
        );

        if !latest_receipt.warnings.is_empty() {
            println!("    Warnings: {}", latest_receipt.warnings.len());
            for warning in &latest_receipt.warnings {
                println!("      - {warning}");
            }
        }

        if latest_receipt.fallback_used == Some(true) {
            println!("    Output format fallback: Used (stream-json → text)");
        }
    }

    // Show effective configuration with source attribution (R11.3)
    println!("\n  Effective configuration:");
    let effective_config = config.effective_config();
    for (key, (value, source)) in effective_config {
        println!("    {key} = {value} (from {source})");
    }

    // Check for partial artifacts and resume capabilities
    let phases = [
        PhaseId::Requirements,
        PhaseId::Design,
        PhaseId::Tasks,
        PhaseId::Review,
        PhaseId::Fixup,
        PhaseId::Final,
    ];
    let mut partial_phases = Vec::new();
    let mut completed_phases = Vec::new();

    for phase in phases {
        if handle.artifact_manager().has_partial_artifact(phase) {
            partial_phases.push(phase);
        }
        if handle.artifact_manager().phase_completed(phase) {
            completed_phases.push(phase);
        }
    }

    if !partial_phases.is_empty() {
        println!("\n  Partial artifacts found:");
        for phase in partial_phases {
            println!("    - {} (from failed execution)", phase.as_str());
        }
    }

    if !completed_phases.is_empty() {
        println!("\n  Completed phases:");
        for phase in completed_phases {
            println!("    - {}", phase.as_str());
        }
    }

    // Check for pending fixups and show intended targets (R5.6)
    check_and_display_fixup_targets(&handle, spec_id)?;

    // Show resume suggestions
    match latest_completed {
        Some(PhaseId::Requirements) => {
            println!("\n  Resume options:");
            println!("    - Continue to Design: xchecker resume {spec_id} --phase design");
            println!("    - Re-run Requirements: xchecker resume {spec_id} --phase requirements");
        }
        Some(PhaseId::Design) => {
            println!("\n  Resume options:");
            println!("    - Continue to Tasks: xchecker resume {spec_id} --phase tasks");
            println!("    - Re-run Design: xchecker resume {spec_id} --phase design");
        }
        Some(PhaseId::Tasks) => {
            println!("\n  Resume options:");
            println!("    - Continue to Review: xchecker resume {spec_id} --phase review");
            println!("    - Re-run Tasks: xchecker resume {spec_id} --phase tasks");
        }
        Some(PhaseId::Review) => {
            println!("\n  Resume options:");
            println!("    - Continue to Fixup: xchecker resume {spec_id} --phase fixup");
            println!("    - Re-run Review: xchecker resume {spec_id} --phase review");
        }
        Some(_) => {
            println!("\n  Resume options:");
            println!("    - Re-run any phase: xchecker resume {spec_id} --phase <phase_name>");
        }
        None => {
            println!("\n  Resume options:");
            println!(
                "    - Start from Requirements: xchecker resume {spec_id} --phase requirements"
            );
        }
    }

    Ok(())
}

/// Check for pending fixups and display intended targets (R5.6)
pub fn check_and_display_fixup_targets(handle: &OrchestratorHandle, spec_id: &str) -> Result<()> {
    use crate::fixup::{FixupMode, FixupParser};

    // Check if Review phase is completed and has fixup markers
    let base_path = handle.artifact_manager().base_path();
    let review_md_path = base_path.join("artifacts").join("30-review.md");

    if !review_md_path.exists() {
        return Ok(()); // No review phase completed yet
    }

    // Read the review content
    let review_content = match std::fs::read_to_string(&review_md_path) {
        Ok(content) => content,
        Err(_) => return Ok(()), // Can't read review file, skip fixup check
    };

    // Create fixup parser in preview mode to check for targets
    let fixup_parser = FixupParser::new(FixupMode::Preview, base_path.clone().into())?;

    // Check if there are fixup markers
    if !fixup_parser.has_fixup_markers(&review_content) {
        return Ok(()); // No fixups needed
    }

    // Parse diffs to get intended targets
    match fixup_parser.parse_diffs(&review_content) {
        Ok(diffs) => {
            if !diffs.is_empty() {
                println!("\n  Pending fixups detected:");
                println!("    Fixup markers found in review phase");
                println!("    Intended targets ({} files):", diffs.len());

                for diff in &diffs {
                    println!("      - {}", diff.target_file);
                }

                // Show preview information
                match fixup_parser.preview_changes(&diffs) {
                    Ok(preview) => {
                        if !preview.all_valid {
                            println!("    ⚠ Warning: Some diffs failed validation");
                        }

                        if !preview.warnings.is_empty() {
                            println!("    Validation warnings:");
                            for warning in &preview.warnings {
                                println!("      - {warning}");
                            }
                        }

                        // Show change summary
                        let mut total_added = 0;
                        let mut total_removed = 0;
                        for (file, summary) in &preview.change_summary {
                            total_added += summary.lines_added;
                            total_removed += summary.lines_removed;
                            if !summary.validation_passed {
                                println!("      ✗ {file}: validation failed");
                            }
                        }

                        if total_added > 0 || total_removed > 0 {
                            println!(
                                "    Estimated changes: +{total_added} lines, -{total_removed} lines"
                            );
                        }
                    }
                    Err(e) => {
                        println!("    ⚠ Warning: Failed to preview changes: {e}");
                    }
                }

                println!("\n    To apply fixups:");
                println!("      xchecker resume {spec_id} --phase fixup --apply-fixups");
                println!("    To preview only (default):");
                println!("      xchecker resume {spec_id} --phase fixup");
            }
        }
        Err(e) => {
            println!("\n  Fixup parsing error: {e}");
            println!("    Review phase contains fixup markers but diffs could not be parsed");
        }
    }

    Ok(())
}
