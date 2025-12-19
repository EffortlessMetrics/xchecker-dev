//! Example: Embedding xchecker to run the Requirements phase
//!
//! This example demonstrates how to use xchecker as a library to programmatically
//! create specs and run phases. It shows both `OrchestratorHandle::new()` for
//! environment-based config discovery and `OrchestratorHandle::from_config()`
//! for explicit configuration.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example embed_requirements
//! ```
//!
//! # Requirements
//!
//! - A valid `.xchecker/` directory in the current workspace or parent directories
//! - For actual LLM execution: Claude CLI configured and available
//!
//! # What This Example Shows
//!
//! 1. Creating an `OrchestratorHandle` using environment-based config discovery
//! 2. Creating an `OrchestratorHandle` using explicit configuration
//! 3. Running a single phase (Requirements)
//! 4. Checking spec status after execution
//! 5. Proper error handling with `XCheckerError`

// Import only from the public API - no internal module paths
use xchecker::{Config, OrchestratorHandle, PhaseId};

/// Demonstrates using `OrchestratorHandle::new()` with environment-based config discovery.
///
/// This approach uses the same configuration discovery as the CLI:
/// - Checks `XCHECKER_HOME` environment variable
/// - Searches upward for `.xchecker/config.toml`
/// - Falls back to built-in defaults
fn demo_environment_based_config() {
    println!("=== Demo 1: Environment-Based Config Discovery ===\n");

    // Create a handle using environment-based config discovery
    // This is the simplest way to use xchecker - it behaves like the CLI
    let spec_id = "embed-demo-env";

    match OrchestratorHandle::new(spec_id) {
        Ok(handle) => {
            println!(
                "✓ Created OrchestratorHandle for spec: {}",
                handle.spec_id()
            );

            // Check what phases can be run
            match handle.legal_next_phases() {
                Ok(phases) => {
                    println!("  Legal next phases: {:?}", phases);
                }
                Err(e) => {
                    println!("  Could not determine legal phases: {}", e);
                }
            }

            // Check current status (read-only, doesn't execute anything)
            match handle.status() {
                Ok(status) => {
                    println!("  Current artifacts: {}", status.artifacts.len());
                    println!("  Schema version: {}", status.schema_version);
                }
                Err(e) => {
                    println!("  Could not get status: {}", e);
                }
            }
        }
        Err(e) => {
            // XCheckerError provides user-friendly error messages
            println!("✗ Failed to create handle: {}", e.display_for_user());
            println!("  Exit code would be: {:?}", e.to_exit_code());
        }
    }

    println!();
}

/// Demonstrates using `OrchestratorHandle::from_config()` with explicit configuration.
///
/// This approach does NOT probe the environment or filesystem for config.
/// Use this when you need deterministic behavior independent of the user's environment.
fn demo_explicit_config() {
    println!("=== Demo 2: Explicit Configuration ===\n");

    let spec_id = "embed-demo-explicit";

    // Build configuration programmatically using the builder pattern
    // This gives you full control over all settings
    let config_result = Config::builder()
        .model("haiku") // Use haiku for fast, cost-effective testing
        .packet_max_bytes(65536) // 64KB packet limit
        .packet_max_lines(1200) // 1200 line limit
        .phase_timeout(std::time::Duration::from_secs(300)) // 5 minute timeout
        .verbose(false) // Quiet mode
        .llm_provider("claude-cli") // Use Claude CLI
        .execution_strategy("controlled") // Controlled execution
        .build();

    match config_result {
        Ok(config) => {
            println!("✓ Built explicit configuration");

            // Create handle with explicit config - no environment probing
            match OrchestratorHandle::from_config(spec_id, config) {
                Ok(handle) => {
                    println!(
                        "✓ Created OrchestratorHandle for spec: {}",
                        handle.spec_id()
                    );

                    // Check status
                    match handle.status() {
                        Ok(status) => {
                            println!("  Artifacts: {}", status.artifacts.len());
                        }
                        Err(e) => {
                            println!("  Could not get status: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Failed to create handle: {}", e.display_for_user());
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to build config: {}", e.display_for_user());
        }
    }

    println!();
}

/// Demonstrates running a phase with dry-run mode.
///
/// Dry-run mode simulates phase execution without calling the LLM.
/// This is useful for testing and validation.
#[tokio::main]
async fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║       xchecker Embedding Example - Requirements Phase       ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Demo 1: Environment-based config discovery
    demo_environment_based_config();

    // Demo 2: Explicit configuration
    demo_explicit_config();

    // Demo 3: Running a phase (dry-run mode)
    println!("=== Demo 3: Running Requirements Phase (Dry-Run) ===\n");

    let spec_id = "embed-demo-run";

    // Create handle and enable dry-run mode
    match OrchestratorHandle::new(spec_id) {
        Ok(mut handle) => {
            // Enable dry-run mode - simulates execution without LLM calls
            handle.set_dry_run(true);
            println!("✓ Created handle with dry-run enabled");

            // Check if we can run the Requirements phase
            match handle.can_run_phase(PhaseId::Requirements) {
                Ok(true) => {
                    println!("✓ Requirements phase can be executed");

                    // Run the Requirements phase
                    println!("  Running Requirements phase...");
                    match handle.run_phase(PhaseId::Requirements).await {
                        Ok(result) => {
                            println!("  Phase completed!");
                            println!("    Success: {}", result.success);
                            if let Some(receipt) = handle.last_receipt_path() {
                                println!("    Receipt: {}", receipt.display());
                            }
                        }
                        Err(e) => {
                            println!("  Phase failed: {}", e);
                        }
                    }
                }
                Ok(false) => {
                    println!("✗ Requirements phase cannot be executed (dependencies not met)");
                }
                Err(e) => {
                    println!("✗ Could not check phase eligibility: {}", e);
                }
            }

            // Check final status
            match handle.status() {
                Ok(status) => {
                    println!("\n  Final status:");
                    println!("    Schema version: {}", status.schema_version);
                    println!("    Artifacts: {}", status.artifacts.len());
                    for artifact in &status.artifacts {
                        println!("      - {} ({})", artifact.path, artifact.blake3_first8);
                    }
                }
                Err(e) => {
                    println!("  Could not get final status: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to create handle: {}", e.display_for_user());
            println!("\n  Note: This example requires a valid .xchecker/ directory.");
            println!("  Run 'xchecker init' in your workspace first.");
        }
    }

    println!("\n=== Example Complete ===");
    println!("\nKey Takeaways:");
    println!("  • Use OrchestratorHandle::new() for CLI-like behavior");
    println!("  • Use OrchestratorHandle::from_config() for deterministic behavior");
    println!("  • Use set_dry_run(true) for testing without LLM calls");
    println!("  • Check status() to inspect artifacts and spec state");
    println!("  • Use can_run_phase() to validate before execution");
}
