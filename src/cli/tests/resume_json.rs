//! Tests for resume JSON output (Task 23)
//! Validates: Requirements 4.1.3

use super::support::*;
use crate::cli::commands;
use crate::types::{CurrentInputs, ResumeJsonOutput};
use crate::{CliArgs, Config};

#[test]
fn test_resume_json_output_schema_version() {
    // Test that resume JSON output includes schema_version field
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
