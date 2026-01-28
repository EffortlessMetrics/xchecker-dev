//! Fixup model types for spec generation workflows
//!
//! This module provides a core types and models used by the fixup system
//! for detecting and applying changes to specification artifacts.

use serde::{Deserialize, Serialize};

/// Mode for fixup operations (preview or apply)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FixupMode {
    /// Preview mode: validate and list targets without making changes
    Preview,
    /// Apply mode: actually applys changes to artifacts
    Apply,
}

/// Represents a unified diff block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedDiff {
    /// Path to the file being modified (relative to spec root)
    pub path: String,
    /// Alias for path (for backward compatibility)
    pub target_file: String,
    /// Full diff content as string
    pub diff_content: String,
    /// List of hunks in the diff
    pub hunks: Vec<DiffHunk>,
}

/// Represents a single diff hunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    /// Starting line number (1-indexed)
    pub start: usize,
    /// Number of lines to remove
    pub remove_count: usize,
    /// Number of lines to add
    pub add_count: usize,
    /// Lines to remove
    pub remove_lines: Vec<String>,
    /// Lines to add
    pub add_lines: Vec<String>,
    /// Old line range (for backward compatibility)
    pub old_range: (usize, usize),
    /// New line range (for backward compatibility)
    pub new_range: (usize, usize),
    /// Full hunk content as string
    pub content: String,
}

/// Preview information for fixup changes
#[derive(Debug, Clone)]
pub struct FixupPreview {
    /// List of target files that would be modified
    pub target_files: Vec<String>,
    /// Whether all targets are valid (within sandbox, etc.)
    pub all_valid: bool,
    /// Any warnings encountered during validation
    pub warnings: Vec<String>,
    /// Summary of changes per file
    pub change_summary: std::collections::HashMap<String, ChangeSummary>,
}

/// Summary of changes for fixup operations
#[derive(Debug, Clone)]
pub struct ChangeSummary {
    /// Number of hunks in this diff
    pub hunk_count: usize,
    /// Lines added by this diff
    pub lines_added: usize,
    /// Lines removed by this diff
    pub lines_removed: usize,
    /// Whether validation passed for this diff
    pub validation_passed: bool,
    /// Validation messages for this diff
    pub validation_messages: Vec<String>,
}

/// Result of applying fixup changes
#[derive(Debug, Clone)]
pub struct FixupResult {
    /// Files that were successfully applied
    pub applied_files: Vec<AppliedFile>,
    /// Files that failed to apply
    pub failed_files: Vec<String>,
    /// Whether three-way merge was used
    pub three_way_used: bool,
    /// Any warnings encountered during application
    pub warnings: Vec<String>,
}

/// Result of applying fixup changes
#[derive(Debug, Clone)]
pub struct FixupApplyResult {
    /// Files that were successfully applied
    pub applied_files: Vec<AppliedFile>,
    /// Files that failed to apply
    pub failed_files: Vec<String>,
    /// Whether three-way merge was used
    pub three_way_used: bool,
    /// Any warnings encountered during application
    pub warnings: Vec<String>,
}

/// Information about a file that was applied
#[derive(Debug, Clone)]
pub struct AppliedFile {
    /// Path to the file (relative to spec root)
    pub path: String,
    /// BLAKE3 hash of the file before modification (first 8 bytes)
    pub blake3_first8: String,
    /// Whether the file was successfully applied
    pub applied: bool,
    /// Warnings encountered during application
    pub warnings: Vec<String>,
}

/// Errors that can occur during fixup operations
#[derive(Debug, thiserror::Error)]
pub enum FixupError {
    /// No fixup markers were found in review output
    NoFixupMarkersFound(String),

    /// Invalid diff format detected
    InvalidDiffFormat(String),

    /// Target file is outside of sandbox
    TargetOutsideSandbox(String),

    /// Failed to apply changes to a file
    ApplyFailed(String),
}

impl std::fmt::Display for FixupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoFixupMarkersFound(msg) => {
                write!(f, "No fixup markers found: {}", msg)?;
            }
            Self::InvalidDiffFormat(msg) => {
                write!(f, "Invalid diff format: {}", msg)?;
            }
            Self::TargetOutsideSandbox(msg) => {
                write!(f, "Target outside sandbox: {}", msg)?;
            }
            Self::ApplyFailed(msg) => {
                write!(f, "Failed to apply: {}", msg)?;
            }
        }
        Ok(())
    }
}
