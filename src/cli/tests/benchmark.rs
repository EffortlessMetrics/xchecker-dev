//! Tests for benchmark command

use crate::benchmark::{BenchmarkConfig, BenchmarkThresholds};
use crate::cli::args::{Cli, Commands};
use crate::cli::commands;
use clap::Parser;

#[test]
fn test_benchmark_command_basic() {
    // Test basic benchmark execution with realistic thresholds for test environments
    // Use more generous thresholds since test environments can be slower
    let result = commands::execute_benchmark_command(
        5,           // file_count
        100,         // file_size
        2,           // iterations
        false,       // json
        Some(10.0),  // max_empty_run_secs - generous for test env
        Some(500.0), // max_packetization_ms - generous for test env (25ms for 5 files)
        None,        // max_rss_mb
        None,        // max_commit_mb
        false,       // verbose
    );

    // Should succeed with realistic test environment thresholds
    assert!(result.is_ok());
}

#[test]
fn test_benchmark_command_with_threshold_overrides() {
    // Test benchmark with custom thresholds (very generous to ensure pass)
    let result = commands::execute_benchmark_command(
        5,             // file_count
        100,           // file_size
        2,             // iterations
        false,         // json
        Some(100.0),   // max_empty_run_secs - very generous
        Some(10000.0), // max_packetization_ms - very generous
        Some(1000.0),  // max_rss_mb - very generous
        Some(2000.0),  // max_commit_mb - very generous
        false,         // verbose
    );

    // Should succeed with generous thresholds
    assert!(result.is_ok());
}

#[test]
fn test_benchmark_command_json_output() {
    // Test that JSON mode runs successfully
    // (We can't easily capture stdout in unit tests, but integration tests verify JSON structure)
    let result = commands::execute_benchmark_command(
        5,             // file_count
        100,           // file_size
        2,             // iterations
        true,          // json - this is what we're testing
        Some(100.0),   // max_empty_run_secs
        Some(10000.0), // max_packetization_ms
        None,          // max_rss_mb
        None,          // max_commit_mb
        false,         // verbose (should be suppressed in JSON mode)
    );

    // Should succeed
    assert!(result.is_ok());
}

#[test]
fn test_benchmark_thresholds_applied() {
    // Test that custom thresholds are properly applied
    let thresholds = BenchmarkThresholds {
        empty_run_max_secs: 3.0,
        packetization_max_ms_per_100_files: 150.0,
        max_rss_mb: Some(500.0),
        max_commit_mb: Some(1000.0),
    };

    let config = BenchmarkConfig {
        file_count: 10,
        file_size_bytes: 100,
        iterations: 2,
        verbose: false,
        thresholds,
    };

    // Verify thresholds are set correctly
    assert_eq!(config.thresholds.empty_run_max_secs, 3.0);
    assert_eq!(config.thresholds.packetization_max_ms_per_100_files, 150.0);
    assert_eq!(config.thresholds.max_rss_mb, Some(500.0));
    assert_eq!(config.thresholds.max_commit_mb, Some(1000.0));
}

#[test]
fn test_benchmark_cli_parsing() {
    // Test that CLI arguments are properly parsed
    // Test basic benchmark command
    let args = vec![
        "xchecker",
        "benchmark",
        "--file-count",
        "50",
        "--iterations",
        "3",
    ];
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_ok());

    if let Ok(cli) = cli {
        match cli.command {
            Commands::Benchmark {
                file_count,
                iterations,
                ..
            } => {
                assert_eq!(file_count, 50);
                assert_eq!(iterations, 3);
            }
            _ => panic!("Expected Benchmark command"),
        }
    }

    // Test benchmark with threshold overrides
    let args_with_thresholds = vec![
        "xchecker",
        "benchmark",
        "--max-empty-run-secs",
        "3.5",
        "--max-packetization-ms",
        "180.0",
        "--json",
    ];
    let cli_thresholds = Cli::try_parse_from(args_with_thresholds);
    assert!(cli_thresholds.is_ok());

    if let Ok(cli) = cli_thresholds {
        match cli.command {
            Commands::Benchmark {
                max_empty_run_secs,
                max_packetization_ms,
                json,
                ..
            } => {
                assert_eq!(max_empty_run_secs, Some(3.5));
                assert_eq!(max_packetization_ms, Some(180.0));
                assert!(json);
            }
            _ => panic!("Expected Benchmark command"),
        }
    }
}

#[test]
fn test_benchmark_default_values() {
    // Test that default values are applied correctly
    let args = vec!["xchecker", "benchmark"];
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_ok());

    if let Ok(cli) = cli {
        match cli.command {
            Commands::Benchmark {
                file_count,
                file_size,
                iterations,
                json,
                max_empty_run_secs,
                max_packetization_ms,
                ..
            } => {
                assert_eq!(file_count, 100); // default
                assert_eq!(file_size, 1024); // default
                assert_eq!(iterations, 5); // default
                assert!(!json); // default false
                assert_eq!(max_empty_run_secs, None); // default None
                assert_eq!(max_packetization_ms, None); // default None
            }
            _ => panic!("Expected Benchmark command"),
        }
    }
}
