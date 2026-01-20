//! Test command implementation
//!
//! Handles `xchecker test` command for integration validation.

use anyhow::{Context, Result};

/// Execute the test command for integration validation
pub fn execute_test_command(components: bool, smoke: bool, verbose: bool) -> Result<()> {
    use crate::integration_tests;

    if verbose {
        println!("Running integration tests...");
    }

    // If no specific test type is specified, run both
    let run_components = components || !smoke;
    let run_smoke = smoke || !components;

    if run_components {
        integration_tests::validate_component_integration()
            .with_context(|| "Component integration validation failed")?;
    }

    if run_smoke {
        integration_tests::run_smoke_tests().with_context(|| "Smoke tests failed")?;
    }

    println!("✓ All integration tests passed successfully");
    Ok(())
}
