//! CLI tests module
//!
//! Tests for CLI command implementations, JSON output formatting,
//! and argument parsing.

use super::*;
use std::env;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};
use tempfile::TempDir;

// Global lock for tests that mutate process-global CLI state (env vars, cwd).
// Any test that uses `TestEnvGuard` or `cli_env_guard()` will be serialized.
static CLI_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn cli_env_guard() -> MutexGuard<'static, ()> {
    CLI_ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

struct TestEnvGuard {
    // Hold the lock for the entire lifetime of the guard
    _lock: MutexGuard<'static, ()>,
    _temp_dir: TempDir,
    original_dir: PathBuf,
    original_xchecker_home: Option<String>,
}

impl Drop for TestEnvGuard {
    fn drop(&mut self) {
        // Restore env and cwd while still holding the lock
        match &self.original_xchecker_home {
            Some(val) => unsafe { env::set_var("XCHECKER_HOME", val) },
            None => unsafe { env::remove_var("XCHECKER_HOME") },
        }
        let _ = env::set_current_dir(&self.original_dir);
        // _lock field drops last, releasing the mutex
    }
}

fn setup_test_environment() -> TestEnvGuard {
    // Take the global CLI lock first
    let lock = cli_env_guard();

    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();
    let original_xchecker_home = env::var("XCHECKER_HOME").ok();

    // From here onwards we're serialized against other CLI tests
    env::set_current_dir(temp_dir.path()).unwrap();

    TestEnvGuard {
        _lock: lock,
        _temp_dir: temp_dir,
        original_dir,
        original_xchecker_home,
    }
}

#[test]
fn test_create_default_config() {
    let cli_args = CliArgs::default();
    let config = Config::discover(&cli_args).unwrap();
    let config_map = commands::create_default_config(true, &config, &cli_args);

    assert_eq!(config_map.get("verbose"), Some(&"true".to_string()));
    assert_eq!(
        config_map.get("packet_max_bytes"),
        Some(&"65536".to_string())
    );
    assert_eq!(
        config_map.get("packet_max_lines"),
        Some(&"1200".to_string())
    );
}

#[test]
fn test_create_default_config_no_verbose() {
    let cli_args = CliArgs::default();
    let config = Config::discover(&cli_args).unwrap();
    let config_map = commands::create_default_config(false, &config, &cli_args);

    assert!(!config_map.contains_key("verbose"));
    assert_eq!(
        config_map.get("packet_max_bytes"),
        Some(&"65536".to_string())
    );
    assert_eq!(
        config_map.get("packet_max_lines"),
        Some(&"1200".to_string())
    );
}

#[tokio::test]
#[allow(clippy::await_holding_lock)] // Test synchronization using mutex guards across awaits is intentional
async fn test_spec_command_execution() -> anyhow::Result<()> {
    use tempfile::TempDir;
    use std::sync::Arc;
    use crate::redaction::SecretRedactor;

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
        Some("input.txt"),
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

#[test]
fn test_status_command_no_spec() {
    let _temp_dir = setup_test_environment();

    let cli_args = CliArgs::default();
    let config = Config::discover(&cli_args).unwrap();

    // Test status for non-existent spec
    let result = commands::execute_status_command("nonexistent-spec", false, &config);
    assert!(result.is_ok());
}

#[test]
fn test_status_command_with_spec() {
    let _temp_dir = setup_test_environment();

    let cli_args = CliArgs::default();
    let config = Config::discover(&cli_args).unwrap();

    // Note: We can't easily test spec creation with stdin in unit tests
    // This test just verifies status command works with non-existent spec
    let result = commands::execute_status_command("test-status-spec", false, &config);
    assert!(result.is_ok());
}

// ===== Spec JSON Output Tests (Task 21.1) =====
// **Property: JSON output includes schema version**
// **Validates: Requirements 4.1.1**

#[test]
fn test_spec_json_output_schema_version() {
    // Test that spec JSON output includes schema_version field
    use crate::types::{PhaseInfo, SpecConfigSummary, SpecOutput};

    let output = SpecOutput {
        schema_version: "spec-json.v1".to_string(),
        spec_id: "test-spec".to_string(),
        phases: vec![PhaseInfo {
            phase_id: "requirements".to_string(),
            status: "completed".to_string(),
            last_run: Some(chrono::Utc::now()),
        }],
        config_summary: SpecConfigSummary {
            execution_strategy: "controlled".to_string(),
            provider: Some("claude-cli".to_string()),
            spec_path: ".xchecker/specs/test-spec".to_string(),
        },
    };

    // Emit as JSON
    let json_result = commands::emit_spec_json(&output);
    assert!(json_result.is_ok(), "Failed to emit spec JSON");

    let json_str = json_result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Verify schema_version is present and correct
    assert_eq!(parsed["schema_version"], "spec-json.v1");
    assert_eq!(parsed["spec_id"], "test-spec");
}

#[test]
fn test_spec_json_output_excludes_packet_contents() {
    // Test that spec JSON output excludes full packet contents
    // per Requirements 4.1.4
    use crate::types::{PhaseInfo, SpecConfigSummary, SpecOutput};

    let output = SpecOutput {
        schema_version: "spec-json.v1".to_string(),
        spec_id: "test-spec".to_string(),
        phases: vec![
            PhaseInfo {
                phase_id: "requirements".to_string(),
                status: "completed".to_string(),
                last_run: None,
            },
            PhaseInfo {
                phase_id: "design".to_string(),
                status: "not_started".to_string(),
                last_run: None,
            },
        ],
        config_summary: SpecConfigSummary {
            execution_strategy: "controlled".to_string(),
            provider: None,
            spec_path: ".xchecker/specs/test-spec".to_string(),
        },
    };

    let json_result = commands::emit_spec_json(&output);
    assert!(json_result.is_ok());

    let json_str = json_result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Verify no packet contents are present
    assert!(
        parsed.get("packet").is_none(),
        "JSON should not contain packet field"
    );
    assert!(
        parsed.get("artifacts").is_none(),
        "JSON should not contain artifacts field"
    );
    assert!(
        parsed.get("raw_response").is_none(),
        "JSON should not contain raw_response field"
    );

    // Verify only expected fields are present
    assert!(parsed.get("schema_version").is_some());
    assert!(parsed.get("spec_id").is_some());
    assert!(parsed.get("phases").is_some());
    assert!(parsed.get("config_summary").is_some());
}

#[test]
fn test_spec_json_command_no_spec() {
    let _temp_dir = setup_test_environment();

    let cli_args = CliArgs::default();
    let config = Config::discover(&cli_args).unwrap();

    // Test spec --json for non-existent spec
    let result = commands::execute_spec_json_command("nonexistent-spec-json", &config);
    assert!(result.is_ok());
}

#[test]
fn test_spec_json_canonical_format() {
    // Test that spec JSON output is in canonical JCS format (no extra whitespace)
    use crate::types::{PhaseInfo, SpecConfigSummary, SpecOutput};

    let output = SpecOutput {
        schema_version: "spec-json.v1".to_string(),
        spec_id: "test-spec".to_string(),
        phases: vec![PhaseInfo {
            phase_id: "requirements".to_string(),
            status: "completed".to_string(),
            last_run: None,
        }],
        config_summary: SpecConfigSummary {
            execution_strategy: "controlled".to_string(),
            provider: None,
            spec_path: ".xchecker/specs/test-spec".to_string(),
        },
    };

    let json_result = commands::emit_spec_json(&output);
    assert!(json_result.is_ok());

    let json_str = json_result.unwrap();

    // Verify canonical JSON properties (no extra whitespace, no newlines)
    assert!(
        !json_str.contains("  "),
        "Canonical JSON should not have indentation"
    );
    assert!(
        !json_str.contains('\n'),
        "Canonical JSON should not have newlines"
    );
}

#[test]
fn test_spec_json_all_phases_present() {
    // Test that all phases are represented in the output
    use crate::types::{PhaseInfo, SpecConfigSummary, SpecOutput};

    let phases = vec![
        PhaseInfo {
            phase_id: "requirements".to_string(),
            status: "completed".to_string(),
            last_run: None,
        },
        PhaseInfo {
            phase_id: "design".to_string(),
            status: "pending".to_string(),
            last_run: None,
        },
        PhaseInfo {
            phase_id: "tasks".to_string(),
            status: "not_started".to_string(),
            last_run: None,
        },
        PhaseInfo {
            phase_id: "review".to_string(),
            status: "not_started".to_string(),
            last_run: None,
        },
        PhaseInfo {
            phase_id: "fixup".to_string(),
            status: "not_started".to_string(),
            last_run: None,
        },
        PhaseInfo {
            phase_id: "final".to_string(),
            status: "not_started".to_string(),
            last_run: None,
        },
    ];

    let output = SpecOutput {
        schema_version: "spec-json.v1".to_string(),
        spec_id: "test-spec".to_string(),
        phases,
        config_summary: SpecConfigSummary {
            execution_strategy: "controlled".to_string(),
            provider: Some("openrouter".to_string()),
            spec_path: ".xchecker/specs/test-spec".to_string(),
        },
    };

    let json_result = commands::emit_spec_json(&output);
    assert!(json_result.is_ok());

    let json_str = json_result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Verify all 6 phases are present
    let phases_array = parsed["phases"].as_array().unwrap();
    assert_eq!(phases_array.len(), 6);

    // Verify phase IDs
    let phase_ids: Vec<&str> = phases_array
        .iter()
        .map(|p| p["phase_id"].as_str().unwrap())
        .collect();
    assert!(phase_ids.contains(&"requirements"));
    assert!(phase_ids.contains(&"design"));
    assert!(phase_ids.contains(&"tasks"));
    assert!(phase_ids.contains(&"review"));
    assert!(phase_ids.contains(&"fixup"));
    assert!(phase_ids.contains(&"final"));
}

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
    use crate::benchmark::{BenchmarkConfig, BenchmarkThresholds};

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
    use clap::Parser;

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
    use clap::Parser;

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

// ===== Status JSON Output Tests (Task 22) =====
// **Property: JSON output includes schema version**
// **Validates: Requirements 4.1.2**

#[test]
fn test_status_json_output_schema_version() {
    // Test that status JSON output includes schema_version field
    use crate::types::{PhaseStatusInfo, StatusJsonOutput};

    let output = StatusJsonOutput {
        schema_version: "status-json.v2".to_string(),
        spec_id: "test-spec".to_string(),
        phase_statuses: vec![PhaseStatusInfo {
            phase_id: "requirements".to_string(),
            status: "success".to_string(),
            receipt_id: Some("requirements-20241201_100000".to_string()),
        }],
        pending_fixups: 0,
        has_errors: false,
        strict_validation: false,
        artifacts: Vec::new(),
        effective_config: std::collections::BTreeMap::new(),
        lock_drift: None,
    };

    // Emit as JSON
    let json_result = commands::emit_status_json(&output);
    assert!(json_result.is_ok(), "Failed to emit status JSON");

    let json_str = json_result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Verify schema_version is present and correct
    assert_eq!(parsed["schema_version"], "status-json.v2");
    assert_eq!(parsed["spec_id"], "test-spec");
}

#[test]
fn test_status_json_output_has_required_fields() {
    // Test that status JSON output has all required fields per Requirements 4.1.2
    use crate::types::{PhaseStatusInfo, StatusJsonOutput};

    let output = StatusJsonOutput {
        schema_version: "status-json.v2".to_string(),
        spec_id: "test-spec".to_string(),
        phase_statuses: vec![
            PhaseStatusInfo {
                phase_id: "requirements".to_string(),
                status: "success".to_string(),
                receipt_id: Some("requirements-20241201_100000".to_string()),
            },
            PhaseStatusInfo {
                phase_id: "design".to_string(),
                status: "failed".to_string(),
                receipt_id: Some("design-20241201_110000".to_string()),
            },
            PhaseStatusInfo {
                phase_id: "tasks".to_string(),
                status: "not_started".to_string(),
                receipt_id: None,
            },
        ],
        pending_fixups: 3,
        has_errors: true,
        strict_validation: false,
        artifacts: Vec::new(),
        effective_config: std::collections::BTreeMap::new(),
        lock_drift: None,
    };

    let json_result = commands::emit_status_json(&output);
    assert!(json_result.is_ok());

    let json_str = json_result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Verify all required fields are present
    assert!(parsed.get("schema_version").is_some());
    assert!(parsed.get("spec_id").is_some());
    assert!(parsed.get("phase_statuses").is_some());
    assert!(parsed.get("pending_fixups").is_some());
    assert!(parsed.get("has_errors").is_some());
    assert!(
        parsed.get("strict_validation").is_some(),
        "strict_validation field should be present"
    );

    // Verify values
    assert_eq!(parsed["pending_fixups"], 3);
    assert_eq!(parsed["has_errors"], true);
    assert_eq!(parsed["strict_validation"], false);
}

#[test]
fn test_status_json_canonical_format() {
    // Test that status JSON output is in canonical JCS format (no extra whitespace)
    use crate::types::{PhaseStatusInfo, StatusJsonOutput};

    let output = StatusJsonOutput {
        schema_version: "status-json.v2".to_string(),
        spec_id: "test-spec".to_string(),
        phase_statuses: vec![PhaseStatusInfo {
            phase_id: "requirements".to_string(),
            status: "success".to_string(),
            receipt_id: None,
        }],
        pending_fixups: 0,
        has_errors: false,
        strict_validation: false,
        artifacts: Vec::new(),
        effective_config: std::collections::BTreeMap::new(),
        lock_drift: None,
    };

    let json_result = commands::emit_status_json(&output);
    assert!(json_result.is_ok());

    let json_str = json_result.unwrap();

    // Verify canonical JSON properties (no extra whitespace, no newlines)
    assert!(
        !json_str.contains("  "),
        "Canonical JSON should not have indentation"
    );
    assert!(
        !json_str.contains('\n'),
        "Canonical JSON should not have newlines"
    );
}

#[test]
fn test_status_json_excludes_raw_packet_contents() {
    // Test that status JSON output excludes raw packet contents (like raw_response)
    // but does include summarized artifacts and effective_config per v2 schema
    use crate::types::{
        ArtifactInfo, ConfigSource, ConfigValue, PhaseStatusInfo, StatusJsonOutput,
    };

    let mut effective_config = std::collections::BTreeMap::new();
    effective_config.insert(
        "model".to_string(),
        ConfigValue {
            value: serde_json::Value::String("haiku".to_string()),
            source: ConfigSource::Config,
        },
    );

    let output = StatusJsonOutput {
        schema_version: "status-json.v2".to_string(),
        spec_id: "test-spec".to_string(),
        phase_statuses: vec![PhaseStatusInfo {
            phase_id: "requirements".to_string(),
            status: "success".to_string(),
            receipt_id: Some("requirements-20241201_100000".to_string()),
        }],
        pending_fixups: 0,
        has_errors: false,
        strict_validation: false,
        artifacts: vec![ArtifactInfo {
            path: "artifacts/requirements.yaml".to_string(),
            blake3_first8: "abc12345".to_string(),
        }],
        effective_config,
        lock_drift: None,
    };

    let json_result = commands::emit_status_json(&output);
    assert!(json_result.is_ok());

    let json_str = json_result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Verify no raw packet/response contents are present
    assert!(
        parsed.get("packet").is_none(),
        "JSON should not contain packet field"
    );
    assert!(
        parsed.get("raw_response").is_none(),
        "JSON should not contain raw_response field"
    );

    // Verify artifacts and effective_config ARE present in v2
    assert!(
        parsed.get("artifacts").is_some(),
        "JSON should contain artifacts field in v2"
    );
    assert!(
        parsed.get("effective_config").is_some(),
        "JSON should contain effective_config field in v2"
    );

    // Verify artifacts have only summary data (blake3_first8), not full content
    let artifacts = parsed["artifacts"].as_array().unwrap();
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0]["blake3_first8"], "abc12345");
    assert!(
        artifacts[0].get("content").is_none(),
        "Artifacts should not include full content"
    );
}

#[test]
fn test_status_json_command_no_spec() {
    let _temp_dir = setup_test_environment();

    let cli_args = CliArgs::default();
    let config = Config::discover(&cli_args).unwrap();

    // Test status --json for non-existent spec
    let result = commands::execute_status_command("nonexistent-spec-json", true, &config);
    assert!(result.is_ok());
}

#[test]
fn test_status_json_all_phases_present() {
    // Test that all phases can be represented in the output
    use crate::types::{PhaseStatusInfo, StatusJsonOutput};

    let phase_statuses = vec![
        PhaseStatusInfo {
            phase_id: "requirements".to_string(),
            status: "success".to_string(),
            receipt_id: Some("requirements-20241201_100000".to_string()),
        },
        PhaseStatusInfo {
            phase_id: "design".to_string(),
            status: "success".to_string(),
            receipt_id: Some("design-20241201_110000".to_string()),
        },
        PhaseStatusInfo {
            phase_id: "tasks".to_string(),
            status: "failed".to_string(),
            receipt_id: Some("tasks-20241201_120000".to_string()),
        },
        PhaseStatusInfo {
            phase_id: "review".to_string(),
            status: "not_started".to_string(),
            receipt_id: None,
        },
        PhaseStatusInfo {
            phase_id: "fixup".to_string(),
            status: "not_started".to_string(),
            receipt_id: None,
        },
        PhaseStatusInfo {
            phase_id: "final".to_string(),
            status: "not_started".to_string(),
            receipt_id: None,
        },
    ];

    let output = StatusJsonOutput {
        schema_version: "status-json.v2".to_string(),
        spec_id: "test-spec".to_string(),
        phase_statuses,
        pending_fixups: 0,
        has_errors: true, // tasks failed
        strict_validation: false,
        artifacts: Vec::new(),
        effective_config: std::collections::BTreeMap::new(),
        lock_drift: None,
    };

    let json_result = commands::emit_status_json(&output);
    assert!(json_result.is_ok());

    let json_str = json_result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Verify all 6 phases are present
    let phases_array = parsed["phase_statuses"].as_array().unwrap();
    assert_eq!(phases_array.len(), 6);

    // Verify phase IDs
    let phase_ids: Vec<&str> = phases_array
        .iter()
        .map(|p| p["phase_id"].as_str().unwrap())
        .collect();
    assert!(phase_ids.contains(&"requirements"));
    assert!(phase_ids.contains(&"design"));
    assert!(phase_ids.contains(&"tasks"));
    assert!(phase_ids.contains(&"review"));
    assert!(phase_ids.contains(&"fixup"));
    assert!(phase_ids.contains(&"final"));
}

// ===== Resume JSON Output Tests (Task 23) =====
// **Property: JSON output includes schema version**
// **Validates: Requirements 4.1.3**

#[test]
fn test_resume_json_output_schema_version() {
    // Test that resume JSON output includes schema_version field
    use crate::types::{CurrentInputs, ResumeJsonOutput};

    let output = ResumeJsonOutput {
        schema_version: "resume-json.v1".to_string(),
        spec_id: "test-spec".to_string(),
        phase: "design".to_string(),
        current_inputs: CurrentInputs {
            available_artifacts: vec!["00-requirements.md".to_string()],
            spec_exists: true,
            latest_completed_phase: Some("requirements".to_string()),
        },
        next_steps: "Run design phase to generate architecture and design from requirements."
            .to_string(),
    };

    // Emit as JSON
    let json_result = commands::emit_resume_json(&output);
    assert!(json_result.is_ok(), "Failed to emit resume JSON");

    let json_str = json_result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Verify schema_version is present and correct
    assert_eq!(parsed["schema_version"], "resume-json.v1");
    assert_eq!(parsed["spec_id"], "test-spec");
    assert_eq!(parsed["phase"], "design");
}

#[test]
fn test_resume_json_output_has_required_fields() {
    // Test that resume JSON output has all required fields per Requirements 4.1.3
    use crate::types::{CurrentInputs, ResumeJsonOutput};

    let output = ResumeJsonOutput {
        schema_version: "resume-json.v1".to_string(),
        spec_id: "test-spec".to_string(),
        phase: "tasks".to_string(),
        current_inputs: CurrentInputs {
            available_artifacts: vec![
                "00-requirements.md".to_string(),
                "10-design.md".to_string(),
            ],
            spec_exists: true,
            latest_completed_phase: Some("design".to_string()),
        },
        next_steps: "Run tasks phase to generate implementation tasks from design.".to_string(),
    };

    let json_result = commands::emit_resume_json(&output);
    assert!(json_result.is_ok());

    let json_str = json_result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Verify all required fields are present
    assert!(parsed.get("schema_version").is_some());
    assert!(parsed.get("spec_id").is_some());
    assert!(parsed.get("phase").is_some());
    assert!(parsed.get("current_inputs").is_some());
    assert!(parsed.get("next_steps").is_some());

    // Verify current_inputs structure
    let current_inputs = &parsed["current_inputs"];
    assert!(current_inputs.get("available_artifacts").is_some());
    assert!(current_inputs.get("spec_exists").is_some());
    assert!(current_inputs.get("latest_completed_phase").is_some());
}

#[test]
fn test_resume_json_canonical_format() {
    // Test that resume JSON output is in canonical JCS format (no extra whitespace)
    use crate::types::{CurrentInputs, ResumeJsonOutput};

    let output = ResumeJsonOutput {
        schema_version: "resume-json.v1".to_string(),
        spec_id: "test-spec".to_string(),
        phase: "requirements".to_string(),
        current_inputs: CurrentInputs {
            available_artifacts: vec![],
            spec_exists: true,
            latest_completed_phase: None,
        },
        next_steps: "Run requirements phase.".to_string(),
    };

    let json_result = commands::emit_resume_json(&output);
    assert!(json_result.is_ok());

    let json_str = json_result.unwrap();

    // Verify canonical JSON properties (no extra whitespace, no newlines)
    assert!(
        !json_str.contains("  "),
        "Canonical JSON should not have indentation"
    );
    assert!(
        !json_str.contains('\n'),
        "Canonical JSON should not have newlines"
    );
}

#[test]
fn test_resume_json_excludes_raw_artifacts() {
    // Test that resume JSON output excludes full packet and raw artifacts
    // per Requirements 4.1.4
    use crate::types::{CurrentInputs, ResumeJsonOutput};

    let output = ResumeJsonOutput {
        schema_version: "resume-json.v1".to_string(),
        spec_id: "test-spec".to_string(),
        phase: "design".to_string(),
        current_inputs: CurrentInputs {
            available_artifacts: vec!["00-requirements.md".to_string()],
            spec_exists: true,
            latest_completed_phase: Some("requirements".to_string()),
        },
        next_steps: "Run design phase.".to_string(),
    };

    let json_result = commands::emit_resume_json(&output);
    assert!(json_result.is_ok());

    let json_str = json_result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Verify no packet contents or raw artifacts are present
    assert!(
        parsed.get("packet").is_none(),
        "JSON should not contain packet field"
    );
    assert!(
        parsed.get("artifacts").is_none(),
        "JSON should not contain artifacts field"
    );
    assert!(
        parsed.get("raw_response").is_none(),
        "JSON should not contain raw_response field"
    );
    assert!(
        parsed.get("artifact_contents").is_none(),
        "JSON should not contain artifact_contents field"
    );

    // Verify only artifact names are present, not contents
    let artifacts = parsed["current_inputs"]["available_artifacts"]
        .as_array()
        .unwrap();
    for artifact in artifacts {
        // Each artifact should be a simple string (name), not an object with contents
        assert!(
            artifact.is_string(),
            "Artifacts should be names only, not objects with contents"
        );
    }
}

#[test]
fn test_resume_json_command_no_spec() {
    let _temp_dir = setup_test_environment();

    let cli_args = CliArgs::default();
    let config = Config::discover(&cli_args).unwrap();

    // Test resume --json for non-existent spec
    let result = commands::execute_resume_json_command("nonexistent-spec-json", "design", &config);
    assert!(result.is_ok());
}

#[test]
fn test_resume_json_all_phases_valid() {
    // Test that all valid phases can be used in resume JSON output
    use crate::types::{CurrentInputs, ResumeJsonOutput};

    let phases = [
        "requirements",
        "design",
        "tasks",
        "review",
        "fixup",
        "final",
    ];

    for phase in &phases {
        let output = ResumeJsonOutput {
            schema_version: "resume-json.v1".to_string(),
            spec_id: "test-spec".to_string(),
            phase: phase.to_string(),
            current_inputs: CurrentInputs {
                available_artifacts: vec![],
                spec_exists: true,
                latest_completed_phase: None,
            },
            next_steps: format!("Run {} phase.", phase),
        };

        let json_result = commands::emit_resume_json(&output);
        assert!(
            json_result.is_ok(),
            "Failed to emit resume JSON for phase: {}",
            phase
        );

        let json_str = json_result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(parsed["phase"], *phase);
    }
}

#[test]
fn test_resume_json_spec_not_exists() {
    // Test resume JSON output when spec doesn't exist
    use crate::types::{CurrentInputs, ResumeJsonOutput};

    let output = ResumeJsonOutput {
        schema_version: "resume-json.v1".to_string(),
        spec_id: "nonexistent-spec".to_string(),
        phase: "requirements".to_string(),
        current_inputs: CurrentInputs {
            available_artifacts: vec![],
            spec_exists: false,
            latest_completed_phase: None,
        },
        next_steps: "Spec 'nonexistent-spec' does not exist. Run 'xchecker spec nonexistent-spec' to create it first.".to_string(),
    };

    let json_result = commands::emit_resume_json(&output);
    assert!(json_result.is_ok());

    let json_str = json_result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Verify spec_exists is false
    assert_eq!(parsed["current_inputs"]["spec_exists"], false);
    // Verify available_artifacts is either empty or not present (due to skip_serializing_if)
    let artifacts = parsed["current_inputs"].get("available_artifacts");
    match artifacts {
        Some(arr) => {
            let arr = arr.as_array().unwrap();
            assert!(arr.is_empty());
        }
        None => {
            // Field is skipped when empty, which is valid
        }
    }
}

// ===== Project List Tests (Task 28) =====
// Tests for derive_spec_status function
// **Validates: Requirements 4.3.3**

#[test]
fn test_derive_spec_status_not_started() {
    // Use isolated home to avoid conflicts with other tests
    let _temp_dir = crate::paths::with_isolated_home();

    // Test status for non-existent spec
    let status = commands::derive_spec_status("nonexistent-spec-status-test");
    assert_eq!(status, "not_started");
}

#[test]
fn test_derive_spec_status_with_receipt() {
    // Use isolated home to avoid conflicts with other tests
    let _temp_dir = crate::paths::with_isolated_home();

    use crate::receipt::ReceiptManager;
    use crate::types::{PacketEvidence, PhaseId};
    use std::collections::HashMap;

    // Create a spec with a receipt
    let spec_id = "test-spec-with-receipt";
    let base_path = crate::paths::spec_root(spec_id);
    crate::paths::ensure_dir_all(&base_path).unwrap();

    let receipt_manager = ReceiptManager::new(&base_path);

    let packet = PacketEvidence {
        files: vec![],
        max_bytes: 65536,
        max_lines: 1200,
    };

    // Create a successful receipt
    let receipt = receipt_manager.create_receipt(
        spec_id,
        PhaseId::Requirements,
        0, // exit_code 0 = success
        vec![],
        "0.1.0",
        "0.8.1",
        "haiku",
        None,
        HashMap::new(),
        packet,
        None,
        None,
        vec![],
        None,
        "native",
        None,
        None,
        None,
        None,
        None,
    );

    receipt_manager.write_receipt(&receipt).unwrap();

    // Test status derivation
    let status = commands::derive_spec_status(spec_id);
    assert!(
        status.contains("success"),
        "Expected 'success' in status, got: {}",
        status
    );
    assert!(
        status.contains("requirements"),
        "Expected 'requirements' in status, got: {}",
        status
    );
}

#[test]
fn test_derive_spec_status_with_failed_receipt() {
    // Use isolated home to avoid conflicts with other tests
    let _temp_dir = crate::paths::with_isolated_home();

    use crate::receipt::ReceiptManager;
    use crate::types::{PacketEvidence, PhaseId};
    use std::collections::HashMap;

    // Create a spec with a failed receipt
    let spec_id = "test-spec-with-failed-receipt";
    let base_path = crate::paths::spec_root(spec_id);
    crate::paths::ensure_dir_all(&base_path).unwrap();

    let receipt_manager = ReceiptManager::new(&base_path);

    let packet = PacketEvidence {
        files: vec![],
        max_bytes: 65536,
        max_lines: 1200,
    };

    // Create a failed receipt
    let receipt = receipt_manager.create_receipt(
        spec_id,
        PhaseId::Design,
        1, // exit_code 1 = failure
        vec![],
        "0.1.0",
        "0.8.1",
        "haiku",
        None,
        HashMap::new(),
        packet,
        None,
        None,
        vec![],
        None,
        "native",
        None,
        None,
        None,
        None,
        None,
    );

    receipt_manager.write_receipt(&receipt).unwrap();

    // Test status derivation
    let status = commands::derive_spec_status(spec_id);
    assert!(
        status.contains("failed"),
        "Expected 'failed' in status, got: {}",
        status
    );
    assert!(
        status.contains("design"),
        "Expected 'design' in status, got: {}",
        status
    );
}

#[test]
fn test_derive_spec_status_uses_latest_receipt() {
    // Use isolated home to avoid conflicts with other tests
    let _temp_dir = crate::paths::with_isolated_home();

    use crate::receipt::ReceiptManager;
    use crate::types::{PacketEvidence, PhaseId};
    use std::collections::HashMap;

    // Create a spec with multiple receipts
    let spec_id = "test-spec-multiple-receipts";
    let base_path = crate::paths::spec_root(spec_id);
    crate::paths::ensure_dir_all(&base_path).unwrap();

    let receipt_manager = ReceiptManager::new(&base_path);

    let packet = PacketEvidence {
        files: vec![],
        max_bytes: 65536,
        max_lines: 1200,
    };

    // Create first receipt (requirements - success)
    let receipt1 = receipt_manager.create_receipt(
        spec_id,
        PhaseId::Requirements,
        0,
        vec![],
        "0.1.0",
        "0.8.1",
        "haiku",
        None,
        HashMap::new(),
        packet.clone(),
        None,
        None,
        vec![],
        None,
        "native",
        None,
        None,
        None,
        None,
        None,
    );
    receipt_manager.write_receipt(&receipt1).unwrap();

    // Small delay to ensure different timestamps
    std::thread::sleep(std::time::Duration::from_millis(1100));

    // Create second receipt (design - success)
    let receipt2 = receipt_manager.create_receipt(
        spec_id,
        PhaseId::Design,
        0,
        vec![],
        "0.1.0",
        "0.8.1",
        "haiku",
        None,
        HashMap::new(),
        packet,
        None,
        None,
        vec![],
        None,
        "native",
        None,
        None,
        None,
        None,
        None,
    );
    receipt_manager.write_receipt(&receipt2).unwrap();

    // Test status derivation - should show design (latest)
    let status = commands::derive_spec_status(spec_id);
    assert!(
        status.contains("design"),
        "Expected 'design' (latest) in status, got: {}",
        status
    );
    assert!(
        status.contains("success"),
        "Expected 'success' in status, got: {}",
        status
    );
}

// ===== Workspace Status JSON Output Tests (Task 29) =====
// **Property: JSON output includes schema version**
// **Validates: Requirements 4.3.4**

#[test]
fn test_workspace_status_json_output_schema_version() {
    // Test that workspace status JSON output includes schema_version field
    use crate::types::{
        WorkspaceSpecStatus, WorkspaceStatusJsonOutput, WorkspaceStatusSummary,
    };

    let output = WorkspaceStatusJsonOutput {
        schema_version: "workspace-status-json.v1".to_string(),
        workspace_name: "test-workspace".to_string(),
        workspace_path: "/path/to/workspace.yaml".to_string(),
        specs: vec![WorkspaceSpecStatus {
            spec_id: "spec-1".to_string(),
            tags: vec!["backend".to_string()],
            status: "success".to_string(),
            latest_phase: Some("tasks".to_string()),
            last_activity: Some(chrono::Utc::now()),
            pending_fixups: 0,
            has_errors: false,
        }],
        summary: WorkspaceStatusSummary {
            total_specs: 1,
            successful_specs: 1,
            failed_specs: 0,
            pending_specs: 0,
            not_started_specs: 0,
            stale_specs: 0,
        },
    };

    // Emit as JSON
    let json_result = commands::emit_workspace_status_json(&output);
    assert!(json_result.is_ok(), "Failed to emit workspace status JSON");

    let json_str = json_result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Verify schema_version is present and correct
    assert_eq!(parsed["schema_version"], "workspace-status-json.v1");
    assert_eq!(parsed["workspace_name"], "test-workspace");
}

#[test]
fn test_workspace_status_json_output_has_required_fields() {
    // Test that workspace status JSON output has all required fields per Requirements 4.3.4
    use crate::types::{
        WorkspaceSpecStatus, WorkspaceStatusJsonOutput, WorkspaceStatusSummary,
    };

    let output = WorkspaceStatusJsonOutput {
        schema_version: "workspace-status-json.v1".to_string(),
        workspace_name: "test-workspace".to_string(),
        workspace_path: "/path/to/workspace.yaml".to_string(),
        specs: vec![
            WorkspaceSpecStatus {
                spec_id: "spec-1".to_string(),
                tags: vec![],
                status: "success".to_string(),
                latest_phase: Some("design".to_string()),
                last_activity: None,
                pending_fixups: 0,
                has_errors: false,
            },
            WorkspaceSpecStatus {
                spec_id: "spec-2".to_string(),
                tags: vec!["frontend".to_string()],
                status: "failed".to_string(),
                latest_phase: Some("requirements".to_string()),
                last_activity: None,
                pending_fixups: 2,
                has_errors: true,
            },
        ],
        summary: WorkspaceStatusSummary {
            total_specs: 2,
            successful_specs: 1,
            failed_specs: 1,
            pending_specs: 0,
            not_started_specs: 0,
            stale_specs: 0,
        },
    };

    let json_result = commands::emit_workspace_status_json(&output);
    assert!(json_result.is_ok());

    let json_str = json_result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Verify all required fields are present
    assert!(parsed.get("schema_version").is_some());
    assert!(parsed.get("workspace_name").is_some());
    assert!(parsed.get("workspace_path").is_some());
    assert!(parsed.get("specs").is_some());
    assert!(parsed.get("summary").is_some());

    // Verify summary fields
    let summary = &parsed["summary"];
    assert!(summary.get("total_specs").is_some());
    assert!(summary.get("successful_specs").is_some());
    assert!(summary.get("failed_specs").is_some());
    assert!(summary.get("pending_specs").is_some());
    assert!(summary.get("not_started_specs").is_some());
    assert!(summary.get("stale_specs").is_some());

    // Verify values
    assert_eq!(summary["total_specs"], 2);
    assert_eq!(summary["failed_specs"], 1);
}

#[test]
fn test_workspace_status_json_canonical_format() {
    // Test that workspace status JSON output is in canonical JCS format (no extra whitespace)
    use crate::types::{WorkspaceStatusJsonOutput, WorkspaceStatusSummary};

    let output = WorkspaceStatusJsonOutput {
        schema_version: "workspace-status-json.v1".to_string(),
        workspace_name: "test-workspace".to_string(),
        workspace_path: "/path/to/workspace.yaml".to_string(),
        specs: vec![],
        summary: WorkspaceStatusSummary {
            total_specs: 0,
            successful_specs: 0,
            failed_specs: 0,
            pending_specs: 0,
            not_started_specs: 0,
            stale_specs: 0,
        },
    };

    let json_result = commands::emit_workspace_status_json(&output);
    assert!(json_result.is_ok());

    let json_str = json_result.unwrap();

    // Verify canonical JSON properties (no extra whitespace, no newlines)
    assert!(
        !json_str.contains("  "),
        "Canonical JSON should not have indentation"
    );
    assert!(
        !json_str.contains('\n'),
        "Canonical JSON should not have newlines"
    );
}

#[test]
fn test_workspace_status_json_spec_statuses() {
    // Test that spec statuses are correctly represented
    use crate::types::{
        WorkspaceSpecStatus, WorkspaceStatusJsonOutput, WorkspaceStatusSummary,
    };

    let output = WorkspaceStatusJsonOutput {
        schema_version: "workspace-status-json.v1".to_string(),
        workspace_name: "test-workspace".to_string(),
        workspace_path: "/path/to/workspace.yaml".to_string(),
        specs: vec![
            WorkspaceSpecStatus {
                spec_id: "spec-success".to_string(),
                tags: vec!["tag1".to_string(), "tag2".to_string()],
                status: "success".to_string(),
                latest_phase: Some("final".to_string()),
                last_activity: Some(chrono::Utc::now()),
                pending_fixups: 0,
                has_errors: false,
            },
            WorkspaceSpecStatus {
                spec_id: "spec-failed".to_string(),
                tags: vec![],
                status: "failed".to_string(),
                latest_phase: Some("design".to_string()),
                last_activity: None,
                pending_fixups: 3,
                has_errors: true,
            },
            WorkspaceSpecStatus {
                spec_id: "spec-not-started".to_string(),
                tags: vec![],
                status: "not_started".to_string(),
                latest_phase: None,
                last_activity: None,
                pending_fixups: 0,
                has_errors: false,
            },
        ],
        summary: WorkspaceStatusSummary {
            total_specs: 3,
            successful_specs: 1,
            failed_specs: 1,
            pending_specs: 0,
            not_started_specs: 1,
            stale_specs: 0,
        },
    };

    let json_result = commands::emit_workspace_status_json(&output);
    assert!(json_result.is_ok());

    let json_str = json_result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Verify specs array
    let specs = parsed["specs"].as_array().unwrap();
    assert_eq!(specs.len(), 3);

    // Verify spec IDs
    let spec_ids: Vec<&str> = specs
        .iter()
        .map(|s| s["spec_id"].as_str().unwrap())
        .collect();
    assert!(spec_ids.contains(&"spec-success"));
    assert!(spec_ids.contains(&"spec-failed"));
    assert!(spec_ids.contains(&"spec-not-started"));

    // Verify statuses
    let statuses: Vec<&str> = specs
        .iter()
        .map(|s| s["status"].as_str().unwrap())
        .collect();
    assert!(statuses.contains(&"success"));
    assert!(statuses.contains(&"failed"));
    assert!(statuses.contains(&"not_started"));
}

#[test]
fn test_workspace_status_cli_parsing() {
    // Test that CLI arguments are properly parsed for project status command
    use clap::Parser;

    // Test basic project status command
    let args = vec!["xchecker", "project", "status"];
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_ok());

    if let Ok(cli) = cli {
        match cli.command {
            Commands::Project(ProjectCommands::Status { workspace, json }) => {
                assert!(workspace.is_none());
                assert!(!json);
            }
            _ => panic!("Expected Project Status command"),
        }
    }

    // Test project status with --json flag
    let args_json = vec!["xchecker", "project", "status", "--json"];
    let cli_json = Cli::try_parse_from(args_json);
    assert!(cli_json.is_ok());

    if let Ok(cli) = cli_json {
        match cli.command {
            Commands::Project(ProjectCommands::Status { workspace, json }) => {
                assert!(workspace.is_none());
                assert!(json);
            }
            _ => panic!("Expected Project Status command"),
        }
    }

    // Test project status with --workspace flag
    let args_workspace = vec![
        "xchecker",
        "project",
        "status",
        "--workspace",
        "/path/to/workspace.yaml",
    ];
    let cli_workspace = Cli::try_parse_from(args_workspace);
    assert!(cli_workspace.is_ok());

    if let Ok(cli) = cli_workspace {
        match cli.command {
            Commands::Project(ProjectCommands::Status { workspace, json }) => {
                assert!(workspace.is_some());
                assert_eq!(
                    workspace.unwrap().to_str().unwrap(),
                    "/path/to/workspace.yaml"
                );
                assert!(!json);
            }
            _ => panic!("Expected Project Status command"),
        }
    }
}

// ===== Project History Tests (Task 30) =====
// **Validates: Requirements 4.3.5**

#[test]
fn test_workspace_history_cli_parsing() {
    // Test that CLI arguments are properly parsed for project history command
    use clap::Parser;

    // Test basic project history command
    let args = vec!["xchecker", "project", "history", "my-spec"];
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_ok());

    if let Ok(cli) = cli {
        match cli.command {
            Commands::Project(ProjectCommands::History { spec_id, json }) => {
                assert_eq!(spec_id, "my-spec");
                assert!(!json);
            }
            _ => panic!("Expected Project History command"),
        }
    }

    // Test project history with --json flag
    let args_json = vec!["xchecker", "project", "history", "my-spec", "--json"];
    let cli_json = Cli::try_parse_from(args_json);
    assert!(cli_json.is_ok());

    if let Ok(cli) = cli_json {
        match cli.command {
            Commands::Project(ProjectCommands::History { spec_id, json }) => {
                assert_eq!(spec_id, "my-spec");
                assert!(json);
            }
            _ => panic!("Expected Project History command"),
        }
    }
}

#[test]
fn test_history_json_output_schema_version() {
    // Test that history JSON output includes schema_version field
    use crate::types::{HistoryEntry, HistoryMetrics, WorkspaceHistoryJsonOutput};

    let output = WorkspaceHistoryJsonOutput {
        schema_version: "workspace-history-json.v1".to_string(),
        spec_id: "test-spec".to_string(),
        timeline: vec![HistoryEntry {
            phase: "requirements".to_string(),
            timestamp: chrono::Utc::now(),
            exit_code: 0,
            success: true,
            tokens_input: Some(1000),
            tokens_output: Some(500),
            fixup_count: None,
            model: Some("haiku".to_string()),
            provider: Some("claude-cli".to_string()),
        }],
        metrics: HistoryMetrics {
            total_executions: 1,
            successful_executions: 1,
            failed_executions: 0,
            total_tokens_input: 1000,
            total_tokens_output: 500,
            total_fixups: 0,
            first_execution: Some(chrono::Utc::now()),
            last_execution: Some(chrono::Utc::now()),
        },
    };

    // Emit as JSON
    let json_result = commands::emit_workspace_history_json(&output);
    assert!(json_result.is_ok(), "Failed to emit history JSON");

    let json_str = json_result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Verify schema_version is present and correct
    assert_eq!(parsed["schema_version"], "workspace-history-json.v1");
    assert_eq!(parsed["spec_id"], "test-spec");
}

#[test]
fn test_history_json_output_has_required_fields() {
    // Test that history JSON output has all required fields per Requirements 4.3.5
    use crate::types::{HistoryEntry, HistoryMetrics, WorkspaceHistoryJsonOutput};

    let output = WorkspaceHistoryJsonOutput {
        schema_version: "workspace-history-json.v1".to_string(),
        spec_id: "test-spec".to_string(),
        timeline: vec![
            HistoryEntry {
                phase: "requirements".to_string(),
                timestamp: chrono::Utc::now(),
                exit_code: 0,
                success: true,
                tokens_input: Some(1000),
                tokens_output: Some(500),
                fixup_count: None,
                model: Some("haiku".to_string()),
                provider: None,
            },
            HistoryEntry {
                phase: "design".to_string(),
                timestamp: chrono::Utc::now(),
                exit_code: 1,
                success: false,
                tokens_input: Some(2000),
                tokens_output: Some(100),
                fixup_count: None,
                model: Some("haiku".to_string()),
                provider: None,
            },
        ],
        metrics: HistoryMetrics {
            total_executions: 2,
            successful_executions: 1,
            failed_executions: 1,
            total_tokens_input: 3000,
            total_tokens_output: 600,
            total_fixups: 0,
            first_execution: Some(chrono::Utc::now()),
            last_execution: Some(chrono::Utc::now()),
        },
    };

    let json_result = commands::emit_workspace_history_json(&output);
    assert!(json_result.is_ok());

    let json_str = json_result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Verify all required fields are present
    assert!(parsed.get("schema_version").is_some());
    assert!(parsed.get("spec_id").is_some());
    assert!(parsed.get("timeline").is_some());
    assert!(parsed.get("metrics").is_some());

    // Verify metrics structure
    let metrics = &parsed["metrics"];
    assert!(metrics.get("total_executions").is_some());
    assert!(metrics.get("successful_executions").is_some());
    assert!(metrics.get("failed_executions").is_some());
    assert!(metrics.get("total_tokens_input").is_some());
    assert!(metrics.get("total_tokens_output").is_some());
    assert!(metrics.get("total_fixups").is_some());

    // Verify values
    assert_eq!(metrics["total_executions"], 2);
    assert_eq!(metrics["successful_executions"], 1);
    assert_eq!(metrics["failed_executions"], 1);
    assert_eq!(metrics["total_tokens_input"], 3000);
    assert_eq!(metrics["total_tokens_output"], 600);
}

#[test]
fn test_history_json_canonical_format() {
    // Test that history JSON output is in canonical JCS format (no extra whitespace)
    use crate::types::{HistoryMetrics, WorkspaceHistoryJsonOutput};

    let output = WorkspaceHistoryJsonOutput {
        schema_version: "workspace-history-json.v1".to_string(),
        spec_id: "test-spec".to_string(),
        timeline: vec![],
        metrics: HistoryMetrics {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            total_tokens_input: 0,
            total_tokens_output: 0,
            total_fixups: 0,
            first_execution: None,
            last_execution: None,
        },
    };

    let json_result = commands::emit_workspace_history_json(&output);
    assert!(json_result.is_ok());

    let json_str = json_result.unwrap();

    // Verify canonical JSON properties (no extra whitespace, no newlines)
    assert!(
        !json_str.contains("  "),
        "Canonical JSON should not have indentation"
    );
    assert!(
        !json_str.contains('\n'),
        "Canonical JSON should not have newlines"
    );
}

#[test]
fn test_history_command_no_spec() {
    let _temp_dir = setup_test_environment();

    // Test history for non-existent spec
    let result = commands::execute_project_history_command("nonexistent-spec-history", false);
    assert!(result.is_ok());
}

#[test]
fn test_history_command_json_no_spec() {
    let _temp_dir = setup_test_environment();

    // Test history --json for non-existent spec
    let result = commands::execute_project_history_command("nonexistent-spec-history-json", true);
    assert!(result.is_ok());
}

#[test]
fn test_history_timeline_entry_structure() {
    // Test that timeline entries have correct structure
    use crate::types::HistoryEntry;

    let entry = HistoryEntry {
        phase: "requirements".to_string(),
        timestamp: chrono::Utc::now(),
        exit_code: 0,
        success: true,
        tokens_input: Some(1000),
        tokens_output: Some(500),
        fixup_count: Some(3),
        model: Some("haiku".to_string()),
        provider: Some("openrouter".to_string()),
    };

    // Serialize and verify
    let json_value = serde_json::to_value(&entry).unwrap();

    assert_eq!(json_value["phase"], "requirements");
    assert_eq!(json_value["exit_code"], 0);
    assert_eq!(json_value["success"], true);
    assert_eq!(json_value["tokens_input"], 1000);
    assert_eq!(json_value["tokens_output"], 500);
    assert_eq!(json_value["fixup_count"], 3);
    assert_eq!(json_value["model"], "haiku");
    assert_eq!(json_value["provider"], "openrouter");
}

#[test]
fn test_history_metrics_aggregation() {
    // Test that metrics are correctly aggregated
    use crate::types::HistoryMetrics;

    let metrics = HistoryMetrics {
        total_executions: 5,
        successful_executions: 3,
        failed_executions: 2,
        total_tokens_input: 10000,
        total_tokens_output: 5000,
        total_fixups: 7,
        first_execution: Some(chrono::Utc::now()),
        last_execution: Some(chrono::Utc::now()),
    };

    // Serialize and verify
    let json_value = serde_json::to_value(&metrics).unwrap();

    assert_eq!(json_value["total_executions"], 5);
    assert_eq!(json_value["successful_executions"], 3);
    assert_eq!(json_value["failed_executions"], 2);
    assert_eq!(json_value["total_tokens_input"], 10000);
    assert_eq!(json_value["total_tokens_output"], 5000);
    assert_eq!(json_value["total_fixups"], 7);
}
