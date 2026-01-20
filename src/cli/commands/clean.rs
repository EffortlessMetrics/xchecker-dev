//! Clean command implementation
//!
//! Handles `xchecker clean` command.

use anyhow::{Context, Result};
use std::io::Write;

use crate::{Config, OrchestratorHandle};

/// Execute the clean command
pub fn execute_clean_command(spec_id: &str, hard: bool, force: bool, _config: &Config) -> Result<()> {
    use crate::lock::utils;

    // Check if clean operation is allowed (no active locks unless forced)
    if let Err(lock_error) = utils::can_clean(spec_id, force, None) {
        return Err(anyhow::anyhow!(
            "Cannot clean spec '{spec_id}': {lock_error}"
        ));
    }

    // Collect information we need before dropping the handle
    let (base_path, artifacts_path, receipts_path, context_path, artifacts, receipts) = {
        // Create handle to access managers (this will acquire a lock)
        let handle = OrchestratorHandle::with_force(spec_id, force)
            .with_context(|| format!("Failed to create orchestrator for spec: {spec_id}"))?;

        // Check if spec directory exists
        let base_path = handle.artifact_manager().base_path();
        if !base_path.exists() {
            println!("No spec found for ID: {spec_id}");
            println!("Directory: {base_path} (does not exist)");
            return Ok(());
        }

        // Show what will be cleaned
        println!("Clean spec: {spec_id}");
        println!("  Directory: {base_path}");

        // List what will be removed
        let artifacts = handle
            .artifact_manager()
            .list_artifacts()
            .with_context(|| "Failed to list artifacts")?;
        let receipts = handle
            .receipt_manager()
            .list_receipts()
            .with_context(|| "Failed to list receipts")?;

        if artifacts.is_empty() && receipts.is_empty() {
            println!("  Nothing to clean (no artifacts or receipts found)");
            // Still need to remove the directory if --hard is specified
            if !hard {
                return Ok(());
            }
        }

        println!("  Will remove:");
        if !artifacts.is_empty() {
            println!("    Artifacts: {} files", artifacts.len());
            for artifact in &artifacts {
                println!("      - {artifact}");
            }
        }

        if !receipts.is_empty() {
            println!("    Receipts: {} files", receipts.len());
            for receipt in &receipts {
                let receipt_filename = format!(
                    "{}-{}.json",
                    receipt.phase,
                    receipt.emitted_at.format("%Y%m%d_%H%M%S")
                );
                println!("      - {receipt_filename}");
            }
        }

        // Get paths before dropping handle (clone to own the data)
        let artifacts_path = handle.artifact_manager().artifacts_path().to_path_buf();
        let receipts_path = base_path.join("receipts");
        let context_path = base_path.join("context");
        let base_path_owned = base_path.to_path_buf();

        (
            base_path_owned,
            artifacts_path,
            receipts_path,
            context_path,
            artifacts,
            receipts,
        )
        // Handle is dropped here, releasing the lock
    };

    // Confirmation prompt (R8.1)
    if !hard {
        println!("\nThis will permanently delete all artifacts and receipts for spec '{spec_id}'.");
        print!("Are you sure? (y/N): ");
        // Flush stdout, logging a warning if it fails (non-fatal)
        if let Err(e) = std::io::stdout().flush() {
            tracing::warn!("Failed to flush stdout: {}", e);
        }

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if input != "y" && input != "yes" {
            println!("Clean cancelled.");
            return Ok(());
        }
    }

    // Perform cleanup (R8.2) - orchestrator lock is now released

    let mut removed_count = 0;

    // Remove artifacts directory
    if artifacts_path.exists() {
        std::fs::remove_dir_all(&artifacts_path)
            .with_context(|| format!("Failed to remove artifacts directory: {artifacts_path}"))?;
        removed_count += artifacts.len();
        println!("✓ Removed artifacts directory");
    }

    // Remove receipts directory
    if receipts_path.exists() {
        std::fs::remove_dir_all(&receipts_path)
            .with_context(|| format!("Failed to remove receipts directory: {receipts_path}"))?;
        removed_count += receipts.len();
        println!("✓ Removed receipts directory");
    }

    // Remove context directory
    if context_path.exists() {
        std::fs::remove_dir_all(&context_path)
            .with_context(|| format!("Failed to remove context directory: {context_path}"))?;
        println!("✓ Removed context directory");
    }

    // Remove the spec directory
    if base_path.exists() {
        if hard {
            // With --hard, remove the entire spec directory including any remaining files
            std::fs::remove_dir_all(&base_path)
                .with_context(|| format!("Failed to remove spec directory: {base_path}"))?;
            println!("✓ Removed spec directory completely");
        } else {
            // Without --hard, only remove if empty
            match std::fs::remove_dir(&base_path) {
                Ok(()) => {
                    println!("✓ Removed empty spec directory");
                }
                Err(_) => {
                    // Directory not empty, that's fine
                    println!("✓ Spec directory retained (contains other files)");
                }
            }
        }
    }

    println!("\nClean completed successfully.");
    println!("  Removed {removed_count} files total");

    Ok(())
}
