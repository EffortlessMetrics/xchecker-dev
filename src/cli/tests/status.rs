//! Tests for status command

use super::support::*;
use crate::cli::commands;
use crate::{CliArgs, Config};

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

#[test]
fn test_status_json_command_no_spec() {
    let _temp_dir = setup_test_environment();

    let cli_args = CliArgs::default();
    let config = Config::discover(&cli_args).unwrap();

    // Test status --json for non-existent spec
    let result = commands::execute_status_command("nonexistent-spec-json", true, &config);
    assert!(result.is_ok());
}
