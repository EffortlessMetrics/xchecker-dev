//! Benchmark command implementation
//!
//! Handles `xchecker benchmark` command for NFR1 performance validation.

use anyhow::{Context, Result};

use crate::emit_jcs;

/// Execute the benchmark command (NFR1 validation)
#[allow(clippy::too_many_arguments)]
pub fn execute_benchmark_command(
    file_count: usize,
    file_size: usize,
    iterations: usize,
    json: bool,
    max_empty_run_secs: Option<f64>,
    max_packetization_ms: Option<f64>,
    max_rss_mb: Option<f64>,
    max_commit_mb: Option<f64>,
    verbose: bool,
) -> Result<()> {
    use crate::benchmark::{BenchmarkConfig, BenchmarkRunner, BenchmarkThresholds};

    // Build custom thresholds if any overrides provided
    let mut thresholds = BenchmarkThresholds::default();
    if let Some(max_secs) = max_empty_run_secs {
        thresholds.empty_run_max_secs = max_secs;
    }
    if let Some(max_ms) = max_packetization_ms {
        thresholds.packetization_max_ms_per_100_files = max_ms;
    }
    if let Some(max_rss) = max_rss_mb {
        thresholds.max_rss_mb = Some(max_rss);
    }
    if let Some(max_commit) = max_commit_mb {
        thresholds.max_commit_mb = Some(max_commit);
    }

    // Only print header if not in JSON mode
    if !json {
        println!("=== xchecker Performance Benchmark ===");
        println!("Validating NFR1 performance targets:");
        println!("  - Empty run: ≤ {:.3}s", thresholds.empty_run_max_secs);
        println!(
            "  - Packetization: ≤ {:.1}ms per 100 files",
            thresholds.packetization_max_ms_per_100_files
        );
        if let Some(max_rss) = thresholds.max_rss_mb {
            println!("  - RSS memory: ≤ {max_rss:.1}MB");
        }
        if let Some(max_commit) = thresholds.max_commit_mb {
            println!("  - Commit memory: ≤ {max_commit:.1}MB");
        }
        println!();
    }

    // Create benchmark configuration
    let config = BenchmarkConfig {
        file_count,
        file_size_bytes: file_size,
        iterations,
        verbose: verbose && !json, // Suppress verbose output in JSON mode
        thresholds,
    };

    if verbose && !json {
        println!("Benchmark configuration:");
        println!("  File count: {}", config.file_count);
        println!("  File size: {} bytes", config.file_size_bytes);
        println!("  Iterations: {}", config.iterations);
        println!();
    }

    // Create and run benchmark
    let runner = BenchmarkRunner::new(config);
    let results = runner
        .run_all_benchmarks()
        .context("Failed to run benchmarks")?;

    // Output results
    if json {
        // Emit structured JSON output (FR-BENCH-004)
        // Use JCS canonicalization for consistent JSON output (FR-CLI-6)
        use serde_json::json;

        let json_output = json!({
            "ok": results.ok,
            "timings_ms": results.timings_ms,
            "rss_mb": results.rss_mb,
            "commit_mb": results.commit_mb,
            "violations": results.violations,
            "config": {
                "file_count": file_count,
                "file_size_bytes": file_size,
                "iterations": iterations,
            },
            "thresholds": {
                "empty_run_max_secs": runner.config.thresholds.empty_run_max_secs,
                "packetization_max_ms_per_100_files": runner.config.thresholds.packetization_max_ms_per_100_files,
                "max_rss_mb": runner.config.thresholds.max_rss_mb,
                "max_commit_mb": runner.config.thresholds.max_commit_mb,
            }
        });

        let canonical_json = emit_jcs(&json_output).context("Failed to emit benchmark JSON")?;
        println!("{canonical_json}");
    } else {
        // Print human-readable results
        runner.print_summary(&results);
    }

    // Exit with appropriate code based on results
    if results.ok {
        if !json {
            println!("\n✓ All performance targets met!");
        }
        Ok(())
    } else {
        if !json {
            println!("\n✗ Some performance targets not met.");
        }
        std::process::exit(1);
    }
}
