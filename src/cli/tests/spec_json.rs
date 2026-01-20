//! Tests for spec JSON output (Task 21.1)
//! Validates: Requirements 4.1.1

use super::support::*;
use crate::cli::commands;
use crate::types::{PhaseInfo, SpecConfigSummary, SpecOutput};
use crate::{CliArgs, Config};

#[test]
fn test_spec_json_output_schema_version() {
    // Test that spec JSON output includes schema_version field
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
