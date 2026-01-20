use crate::types::Priority;
use camino::Utf8PathBuf;
use globset::{Glob, GlobSet, GlobSetBuilder};

/// Priority rules defining the selection order
/// Order: *.core.yaml (non-evictable) → SPEC/ADR/REPORT → README/SCHEMA → misc
/// LIFO within each priority class
#[derive(Debug, Clone)]
pub struct PriorityRules {
    /// High priority patterns (SPEC/ADR/REPORT)
    pub high: GlobSet,
    /// Medium priority patterns (README/SCHEMA)
    pub medium: GlobSet,
    /// Low priority patterns (misc files)
    #[allow(dead_code)] // Reserved for metadata tracking
    pub low: GlobSet,
}

impl Default for PriorityRules {
    fn default() -> Self {
        let mut high_builder = GlobSetBuilder::new();
        high_builder.add(Glob::new("**/SPEC*").unwrap());
        high_builder.add(Glob::new("**/ADR*").unwrap());
        high_builder.add(Glob::new("**/REPORT*").unwrap());
        high_builder.add(Glob::new("**/*SPEC*").unwrap());
        high_builder.add(Glob::new("**/*ADR*").unwrap());
        high_builder.add(Glob::new("**/*REPORT*").unwrap());
        // Problem statement files get high priority - critical context for LLM
        high_builder.add(Glob::new("**/problem-statement*").unwrap());
        high_builder.add(Glob::new("**/*problem-statement*").unwrap());

        let mut medium_builder = GlobSetBuilder::new();
        medium_builder.add(Glob::new("**/README*").unwrap());
        medium_builder.add(Glob::new("**/SCHEMA*").unwrap());
        medium_builder.add(Glob::new("**/*README*").unwrap());
        medium_builder.add(Glob::new("**/*SCHEMA*").unwrap());

        let mut low_builder = GlobSetBuilder::new();
        low_builder.add(Glob::new("**/*").unwrap()); // Catch-all for misc files

        Self {
            high: high_builder.build().unwrap(),
            medium: medium_builder.build().unwrap(),
            low: low_builder.build().unwrap(),
        }
    }
}

/// Represents a file selected for potential inclusion in a packet
#[derive(Debug, Clone)]
pub struct SelectedFile {
    /// Path to the file
    pub path: Utf8PathBuf,
    /// File content
    pub content: String,
    /// Priority level
    pub priority: Priority,
    /// BLAKE3 hash before redaction
    pub blake3_pre_redaction: String,
    /// Number of lines in the file
    #[allow(dead_code)] // Metadata for budget tracking
    pub line_count: usize,
    /// Number of bytes in the file
    #[allow(dead_code)] // Metadata for budget tracking
    pub byte_count: usize,
}
