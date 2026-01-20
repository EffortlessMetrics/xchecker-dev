//! Tests for spec command execution

use super::support::*;
use crate::cli::commands;
use crate::redaction::SecretRedactor;
use crate::{CliArgs, Config};
use serial_test::serial;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
#[serial]
async fn test_spec_command_execution() -> anyhow::Result<()> {
    // Take the global CLI lock for env/cwd mutations
    let _lock = cli_env_guard();

    // Save original state
    let original_dir = std::env::current_dir()?;
    let original_xchecker_home = std::env::var("XCHECKER_HOME").ok();
    let original_skip_llm = std::env::var("XCHECKER_SKIP_LLM_TESTS").ok();

    // Setup isolated test root
    let temp = TempDir::new()?;
    let root = temp.path();

    // Make it look like a repo root
    std::fs::create_dir_all(root.join(".git"))?;

    // Set process environment
    std::env::set_current_dir(root)?;
    unsafe {
        std::env::set_var("XCHECKER_HOME", root);
        std::env::set_var("XCHECKER_SKIP_LLM_TESTS", "1");
    }

    // Create minimal config
    std::fs::write(
        root.join("xchecker.toml"),
        r#"
[runner]
runner_mode = "native"

[packet]
packet_max_bytes = 1048576
packet_max_lines = 5000
"#,
    )?;

    // Create minimal CLI args for dry-run
    let cli_args = CliArgs::default();

    let config = Config::discover(&cli_args)?;
    let redactor = Arc::new(SecretRedactor::from_config(&config)?);

    // Create a minimal input file for the spec
    std::fs::write(root.join("input.txt"), "Test requirement")?;

    // Test dry-run execution (fast, no real LLMs)
    // This should complete quickly and not hang
    let result = commands::execute_spec_command(
        "test-spec",
        "fs",
        None,
        Some(root.to_str().unwrap()), // repo path
        true,                         // dry_run = true
        false,
        false,
        false,
        false,
        &config,
        &cli_args,
        &redactor,
    )
    .await;

    // Restore original environment before asserting
    let _ = std::env::set_current_dir(&original_dir);
    match original_xchecker_home {
        Some(val) => unsafe { std::env::set_var("XCHECKER_HOME", val) },
        None => unsafe { std::env::remove_var("XCHECKER_HOME") },
    }
    match original_skip_llm {
        Some(val) => unsafe { std::env::set_var("XCHECKER_SKIP_LLM_TESTS", val) },
        None => unsafe { std::env::remove_var("XCHECKER_SKIP_LLM_TESTS") },
    }

    // In dry-run mode with a valid source, this should succeed
    // The important thing is it doesn't hang and completes quickly
    assert!(
        result.is_ok(),
        "Dry-run spec execution should succeed: {:?}",
        result.err()
    );

    Ok(())
}
