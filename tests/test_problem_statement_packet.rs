//! Integration tests for problem statement handling in packets and prompts
//!
//! **WHITE-BOX TEST**: This test uses internal module APIs (`packet::{...}`, `phase::{...}`,
//! `phases::RequirementsPhase`, `types::Priority`) and may break with internal refactors.
//! These tests are intentionally white-box to validate internal implementation details.
//! See FR-TEST-4 for white-box test policy.
//!
//! Verifies that:
//! 1. Problem statement files (source/00-problem-statement.md) are included in packets
//! 2. Problem statement content is prioritized in packet selection
//! 3. Problem statement is included in phase prompts when provided via config

use anyhow::Result;
use camino::Utf8PathBuf;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;
use xchecker::packet::{ContentSelector, PacketBuilder};
use xchecker::phase::{Phase, PhaseContext};
use xchecker::phases::RequirementsPhase;
use xchecker::types::Priority;

/// Test that problem statement files are included with high priority
#[test]
fn test_problem_statement_file_priority() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let base_path = Utf8PathBuf::try_from(temp_dir.path().to_path_buf())?;

    // Create source directory with problem statement
    let source_dir = base_path.join("source");
    fs::create_dir_all(&source_dir)?;
    fs::write(
        source_dir.join("00-problem-statement.md"),
        "# Problem Statement\n\nBuild a REST API for user management.",
    )?;

    // Create some other files
    fs::write(base_path.join("README.md"), "# Project README")?;
    fs::write(base_path.join("config.toml"), "# Config file")?;

    let selector = ContentSelector::new()?;
    let files = selector.select_files(&base_path)?;

    // Find the problem statement file
    let problem_file = files
        .iter()
        .find(|f| f.path.to_string().contains("problem-statement"));

    assert!(
        problem_file.is_some(),
        "Problem statement file should be selected"
    );

    // Problem statement should have High priority
    let problem_file = problem_file.unwrap();
    assert_eq!(
        problem_file.priority,
        Priority::High,
        "Problem statement should have High priority"
    );

    Ok(())
}

/// Test that problem statement is included in packet content
#[test]
fn test_problem_statement_in_packet_content() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let base_path = Utf8PathBuf::try_from(temp_dir.path().to_path_buf())?;

    // Create source directory with problem statement
    let source_dir = base_path.join("source");
    let context_dir = base_path.join("context");
    fs::create_dir_all(&source_dir)?;
    fs::create_dir_all(&context_dir)?;

    let problem_text =
        "Build a REST API for user management with authentication and CRUD operations.";
    fs::write(
        source_dir.join("00-problem-statement.md"),
        format!("# Problem Statement\n\n{problem_text}"),
    )?;

    let mut builder = PacketBuilder::new()?;
    let packet = builder.build_packet(&base_path, "requirements", &context_dir, None)?;

    // Packet content should include the problem statement text
    assert!(
        packet.content.contains(problem_text),
        "Packet should contain problem statement text"
    );

    Ok(())
}

/// Test that prompt includes problem statement from config
#[test]
fn test_prompt_includes_problem_statement() {
    let temp_dir = TempDir::new().unwrap();
    let spec_dir = temp_dir.path().to_path_buf();

    let problem_text = "Build a REST API for user management with authentication.";

    // Create PhaseContext with problem statement in config
    let mut config = HashMap::new();
    config.insert("problem_statement".to_string(), problem_text.to_string());

    let ctx = PhaseContext {
        spec_id: "test-spec".to_string(),
        spec_dir,
        config,
        artifacts: vec![],
        selectors: None,
        strict_validation: false,
        redactor: Default::default(),
    };

    let phase = RequirementsPhase::new();
    let prompt = phase.prompt(&ctx);

    // Prompt should include the problem statement
    assert!(
        prompt.contains(problem_text),
        "Prompt should contain problem statement text"
    );

    // Prompt should have the problem statement section header
    assert!(
        prompt.contains("# Problem Statement"),
        "Prompt should have Problem Statement section"
    );
}

/// Test that prompt has fallback when no problem statement provided
#[test]
fn test_prompt_fallback_without_problem_statement() {
    let temp_dir = TempDir::new().unwrap();
    let spec_dir = temp_dir.path().to_path_buf();

    // Create PhaseContext without problem statement
    let ctx = PhaseContext {
        spec_id: "test-spec".to_string(),
        spec_dir,
        config: HashMap::new(),
        artifacts: vec![],
        selectors: None,
        strict_validation: false,
        redactor: Default::default(),
    };

    let phase = RequirementsPhase::new();
    let prompt = phase.prompt(&ctx);

    // Prompt should have fallback text
    assert!(
        prompt.contains("No explicit problem statement was provided"),
        "Prompt should have fallback text when no problem statement"
    );
}

/// Test that problem statement files with various naming patterns are detected
#[test]
fn test_problem_statement_naming_patterns() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let base_path = Utf8PathBuf::try_from(temp_dir.path().to_path_buf())?;

    // Test different naming patterns that should be matched
    let patterns = vec![
        "problem-statement.md",
        "00-problem-statement.md",
        "my-problem-statement.txt",
    ];

    for pattern in patterns {
        // Clean up from previous iteration
        for entry in fs::read_dir(&base_path)? {
            let entry = entry?;
            if entry.path().is_file() {
                fs::remove_file(entry.path())?;
            }
        }

        fs::write(base_path.join(pattern), format!("# Problem: {pattern}"))?;

        let selector = ContentSelector::new()?;
        let files = selector.select_files(&base_path)?;

        let found = files
            .iter()
            .any(|f| f.path.to_string().contains("problem-statement"));

        assert!(
            found,
            "Pattern '{pattern}' should be detected as problem statement"
        );
    }

    Ok(())
}

/// Test that spec_id is included in prompt
#[test]
fn test_prompt_includes_spec_id() {
    let temp_dir = TempDir::new().unwrap();
    let spec_dir = temp_dir.path().to_path_buf();

    let ctx = PhaseContext {
        spec_id: "my-test-spec-123".to_string(),
        spec_dir,
        config: HashMap::new(),
        artifacts: vec![],
        selectors: None,
        strict_validation: false,
        redactor: Default::default(),
    };

    let phase = RequirementsPhase::new();
    let prompt = phase.prompt(&ctx);

    assert!(
        prompt.contains("my-test-spec-123"),
        "Prompt should contain spec ID"
    );
}
