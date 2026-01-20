//! Init command implementation
//!
//! Handles `xchecker init` command for spec initialization.

use anyhow::{Context, Result};
use std::path::PathBuf;

use super::common::detect_claude_cli_version;

use crate::Config;

/// Execute the init command to initialize a spec with optional lockfile
pub fn execute_init_command(spec_id: &str, create_lock: bool, config: &Config) -> Result<()> {
    use crate::lock::XCheckerLock;

    println!("Initializing spec: {spec_id}");

    // Create spec directory structure
    let spec_dir = PathBuf::from(".xchecker").join("specs").join(spec_id);
    let artifacts_dir = spec_dir.join("artifacts");
    let receipts_dir = spec_dir.join("receipts");
    let context_dir = spec_dir.join("context");

    // Check if spec already exists
    if spec_dir.exists() {
        println!("  Spec directory already exists: {}", spec_dir.display());

        // Check if lockfile exists
        let lock_path = spec_dir.join("lock.json");
        if lock_path.exists() {
            println!("  Lockfile already exists: {}", lock_path.display());

            if create_lock {
                println!("  ⚠ Warning: --create-lock specified but lockfile already exists");
                println!("  To update the lockfile, delete it first and run init again");
            }

            return Ok(());
        }
    } else {
        // Create directory structure (ignore benign races)
        crate::paths::ensure_dir_all(&artifacts_dir).with_context(|| {
            format!(
                "Failed to create artifacts directory: {}",
                artifacts_dir.display()
            )
        })?;
        crate::paths::ensure_dir_all(&receipts_dir).with_context(|| {
            format!(
                "Failed to create receipts directory: {}",
                receipts_dir.display()
            )
        })?;
        crate::paths::ensure_dir_all(&context_dir).with_context(|| {
            format!(
                "Failed to create context directory: {}",
                context_dir.display()
            )
        })?;

        println!("  ✓ Created spec directory: {}", spec_dir.display());
        println!("  ✓ Created artifacts directory");
        println!("  ✓ Created receipts directory");
        println!("  ✓ Created context directory");
    }

    // Create lockfile if requested
    if create_lock {
        // Get model from config or use default
        let model = config.defaults.model.as_deref().unwrap_or("haiku");

        // Get Claude CLI version (we'll need to detect this - for now use a placeholder)
        // In a real implementation, this would call `claude --version` and parse the output
        let claude_cli_version =
            detect_claude_cli_version().unwrap_or_else(|_| "unknown".to_string());

        let lock = XCheckerLock::new(model.to_string(), claude_cli_version.clone());

        lock.save(spec_id)
            .with_context(|| "Failed to save lockfile")?;

        println!("  ✓ Created lockfile: lock.json");
        println!("    Model: {model}");
        println!("    Claude CLI version: {claude_cli_version}");
        println!("    Schema version: 1");

        println!("\n  Lockfile will track drift for:");
        println!("    - Model changes (current: {model})");
        println!("    - Claude CLI version changes (current: {claude_cli_version})");
        println!("    - Schema version changes (current: 1)");
        println!("\n  Use --strict-lock flag to hard fail on drift detection");
    } else {
        println!("\n  No lockfile created (use --create-lock to pin model and CLI version)");
    }

    println!("\nSpec '{spec_id}' initialized successfully");
    println!("  Directory: {}", spec_dir.display());

    Ok(())
}
