//! Tests for status JSON output (Task 22)
//! Validates: Requirements 4.1.2

use crate::cli::commands;
use crate::types::{
    ArtifactInfo, ConfigSource, ConfigValue, PhaseStatusInfo, StatusJsonOutput,
};
use std::collections::BTreeMap;

#[test]
fn test_status_json_output_schema_version() {
    // Test that status JSON output includes schema_version field
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
    let mut effective_config = BTreeMap::new();
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
fn test_status_json_all_phases_present() {
    // Test that all phases can be represented in the output
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
