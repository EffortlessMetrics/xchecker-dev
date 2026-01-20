//! Tests for project commands (workspace management)
//! Validates: Requirements 4.3.3, 4.3.4, 4.3.5

use super::support::*;
use crate::cli::args::{Cli, Commands, ProjectCommands};
use crate::cli::commands;
use crate::receipt::ReceiptManager;
use crate::types::{
    HistoryEntry, HistoryMetrics, PacketEvidence, PhaseId, WorkspaceHistoryJsonOutput,
    WorkspaceSpecStatus, WorkspaceStatusJsonOutput, WorkspaceStatusSummary,
};
use clap::Parser;
use std::collections::HashMap;

// ===== Project List Tests (Task 28) =====

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

    // Small delay to ensure different timestamps (50ms is sufficient for chrono precision)
    std::thread::sleep(std::time::Duration::from_millis(50));

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

#[test]
fn test_workspace_status_json_output_schema_version() {
    // Test that workspace status JSON output includes schema_version field
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

#[test]
fn test_workspace_history_cli_parsing() {
    // Test that CLI arguments are properly parsed for project history command
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
