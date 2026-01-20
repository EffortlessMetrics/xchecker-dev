use super::model::{PriorityRules, SelectedFile};
use crate::config::Selectors;
use crate::types::Priority;
use anyhow::{Context, Result};
use blake3::Hasher;
use camino::{Utf8Path, Utf8PathBuf};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::fs;

/// Content selector that implements priority-based file selection
/// with concrete defaults and LIFO ordering within priority classes
#[derive(Debug, Clone)]
pub struct ContentSelector {
    /// Include patterns for file selection
    include_patterns: GlobSet,
    /// Exclude patterns for file filtering
    exclude_patterns: GlobSet,
    /// Priority rules for content selection
    priority_rules: PriorityRules,
}

impl ContentSelector {
    /// Create a new `ContentSelector` with default patterns
    pub fn new() -> Result<Self> {
        let mut include_builder = GlobSetBuilder::new();
        // Default include patterns based on requirements
        include_builder.add(Glob::new("**/*.md")?);
        include_builder.add(Glob::new("**/*.yaml")?);
        include_builder.add(Glob::new("**/*.yml")?);
        include_builder.add(Glob::new("**/*.toml")?);
        include_builder.add(Glob::new("**/*.txt")?);
        include_builder.add(Glob::new("**/README*")?);
        include_builder.add(Glob::new("**/SPEC*")?);
        include_builder.add(Glob::new("**/ADR*")?);
        include_builder.add(Glob::new("**/REPORT*")?);
        include_builder.add(Glob::new("**/SCHEMA*")?);
        // Spec context directories - problem statements, notes
        include_builder.add(Glob::new("context/**/*.md")?);
        include_builder.add(Glob::new("source/**/*.md")?);

        let mut exclude_builder = GlobSetBuilder::new();
        // Default exclude patterns
        exclude_builder.add(Glob::new("**/target/**")?);
        exclude_builder.add(Glob::new("**/node_modules/**")?);
        exclude_builder.add(Glob::new("**/.git/**")?);
        // Note: .xchecker/** is excluded for repo-level searches,
        // but when building packets from spec_dir, we're already inside .xchecker/specs/<id>

        Ok(Self {
            include_patterns: include_builder.build()?,
            exclude_patterns: exclude_builder.build()?,
            priority_rules: PriorityRules::default(),
        })
    }

    /// Create a `ContentSelector` with custom patterns
    /// Alternative constructor for custom pattern configuration
    #[allow(dead_code)] // Alternative API constructor
    pub fn with_patterns(include_patterns: Vec<&str>, exclude_patterns: Vec<&str>) -> Result<Self> {
        let mut include_builder = GlobSetBuilder::new();
        for pattern in include_patterns {
            include_builder.add(Glob::new(pattern)?);
        }

        let mut exclude_builder = GlobSetBuilder::new();
        for pattern in exclude_patterns {
            exclude_builder.add(Glob::new(pattern)?);
        }

        Ok(Self {
            include_patterns: include_builder.build()?,
            exclude_patterns: exclude_builder.build()?,
            priority_rules: PriorityRules::default(),
        })
    }

    /// Create a `ContentSelector` from optional config selectors.
    ///
    /// # Precedence
    ///
    /// - If `selectors` is `Some`: use those include/exclude patterns.
    /// - If `None`: fall back to built-in defaults.
    ///
    /// # Errors
    ///
    /// Returns an error if any glob pattern is invalid.
    pub fn from_selectors(selectors: Option<&Selectors>) -> Result<Self> {
        match selectors {
            Some(sel) => {
                let mut include_builder = GlobSetBuilder::new();
                for pattern in &sel.include {
                    include_builder.add(Glob::new(pattern)?);
                }

                let mut exclude_builder = GlobSetBuilder::new();
                for pattern in &sel.exclude {
                    exclude_builder.add(Glob::new(pattern)?);
                }

                Ok(Self {
                    include_patterns: include_builder.build()?,
                    exclude_patterns: exclude_builder.build()?,
                    priority_rules: PriorityRules::default(),
                })
            }
            None => Self::new(),
        }
    }

    /// Determine the priority of a file based on its path
    #[must_use]
    pub fn get_priority(&self, path: &Utf8Path) -> Priority {
        let path_str = path.as_str();

        // *.core.yaml files are always Upstream priority (non-evictable)
        if path_str.ends_with(".core.yaml") {
            return Priority::Upstream;
        }

        // Check priority patterns
        if self.priority_rules.high.is_match(path_str) {
            Priority::High
        } else if self.priority_rules.medium.is_match(path_str) {
            Priority::Medium
        } else {
            Priority::Low
        }
    }

    /// Check if a file should be included based on include/exclude patterns
    #[must_use]
    pub fn should_include(&self, path: &Utf8Path) -> bool {
        let path_str = path.as_str();

        // First check if excluded
        if self.exclude_patterns.is_match(path_str) {
            return false;
        }

        // Then check if included
        self.include_patterns.is_match(path_str)
    }

    /// Select files from a directory with priority-based ordering
    /// Returns files grouped by priority, with LIFO ordering within each group
    pub fn select_files(&self, base_path: &Utf8Path) -> Result<Vec<SelectedFile>> {
        let mut files = Vec::new();

        // Walk the directory tree
        self.walk_directory(base_path, &mut files)?;

        // Sort by priority (Upstream first, then High, Medium, Low)
        // Within each priority, maintain LIFO order (reverse chronological)
        files.sort_by(|a, b| {
            match a.priority.cmp(&b.priority) {
                std::cmp::Ordering::Equal => {
                    // Within same priority, use LIFO (reverse order)
                    b.path.cmp(&a.path)
                }
                other => other,
            }
        });

        Ok(files)
    }

    /// Recursively walk directory and collect matching files
    fn walk_directory(&self, dir: &Utf8Path, files: &mut Vec<SelectedFile>) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = Utf8PathBuf::try_from(entry.path()).context("Invalid UTF-8 path")?;

            if path.is_dir() {
                self.walk_directory(&path, files)?;
            } else if self.should_include(&path) {
                let priority = self.get_priority(&path);
                let content = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read file: {path}"))?;

                // Calculate pre-redaction hash
                let mut hasher = Hasher::new();
                hasher.update(content.as_bytes());
                let blake3_pre_redaction = hasher.finalize().to_hex().to_string();

                let line_count = content.lines().count();
                let byte_count = content.len();

                files.push(SelectedFile {
                    path: path.clone(),
                    content,
                    priority,
                    blake3_pre_redaction,
                    line_count,
                    byte_count,
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_priority_assignment() {
        let selector = ContentSelector::new().unwrap();

        // Test upstream priority for .core.yaml files
        assert_eq!(
            selector.get_priority(Utf8Path::new("test.core.yaml")),
            Priority::Upstream
        );
        assert_eq!(
            selector.get_priority(Utf8Path::new("docs/design.core.yaml")),
            Priority::Upstream
        );

        // Test high priority patterns
        assert_eq!(
            selector.get_priority(Utf8Path::new("SPEC-001.md")),
            Priority::High
        );
        assert_eq!(
            selector.get_priority(Utf8Path::new("docs/ADR-001.md")),
            Priority::High
        );
        assert_eq!(
            selector.get_priority(Utf8Path::new("REPORT-final.md")),
            Priority::High
        );

        // Test medium priority patterns
        assert_eq!(
            selector.get_priority(Utf8Path::new("README.md")),
            Priority::Medium
        );
        assert_eq!(
            selector.get_priority(Utf8Path::new("docs/SCHEMA.yaml")),
            Priority::Medium
        );

        // Test low priority (misc files)
        assert_eq!(
            selector.get_priority(Utf8Path::new("src/main.rs")),
            Priority::Low
        );
        assert_eq!(
            selector.get_priority(Utf8Path::new("config.toml")),
            Priority::Low
        );
    }

    #[test]
    fn test_include_exclude_patterns() {
        let selector = ContentSelector::new().unwrap();

        // Should include
        assert!(selector.should_include(Utf8Path::new("README.md")));
        assert!(selector.should_include(Utf8Path::new("config.yaml")));
        assert!(selector.should_include(Utf8Path::new("docs/spec.md")));

        // Should exclude
        assert!(!selector.should_include(Utf8Path::new("target/debug/main")));
        assert!(!selector.should_include(Utf8Path::new("node_modules/package/index.js")));
        assert!(!selector.should_include(Utf8Path::new(".git/config")));
        assert!(!selector.should_include(Utf8Path::new(".xchecker/specs/test/receipt.json")));
    }

    #[test]
    fn test_file_selection_ordering() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let base_path = Utf8PathBuf::try_from(temp_dir.path().to_path_buf())?;

        // Create test files with different priorities
        fs::write(base_path.join("test.core.yaml"), "upstream: content")?;
        fs::write(base_path.join("SPEC-001.md"), "# High priority spec")?;
        fs::write(base_path.join("README.md"), "# Medium priority readme")?;
        fs::write(base_path.join("config.toml"), "# Low priority config")?;

        let selector = ContentSelector::new()?;
        let files = selector.select_files(&base_path)?;

        // Verify ordering: Upstream -> High -> Medium -> Low
        assert_eq!(files.len(), 4);
        assert_eq!(files[0].priority, Priority::Upstream);
        assert_eq!(files[1].priority, Priority::High);
        assert_eq!(files[2].priority, Priority::Medium);
        assert_eq!(files[3].priority, Priority::Low);

        Ok(())
    }

    #[test]
    fn test_content_selector_from_selectors_uses_defaults_when_none() -> Result<()> {
        let selector = ContentSelector::from_selectors(None)?;

        // Verify that when None is passed, selector uses default patterns
        // We test this by checking it accepts default include patterns and rejects default excludes
        // Default includes: *.md, *.yaml, *.yml, etc.
        assert!(selector.should_include(Utf8Path::new("README.md")));
        assert!(selector.should_include(Utf8Path::new("config.yaml")));

        // Default excludes: .git/*, target/*, node_modules/*, etc.
        assert!(!selector.should_include(Utf8Path::new(".git/config")));
        assert!(!selector.should_include(Utf8Path::new("target/debug/main")));
        assert!(!selector.should_include(Utf8Path::new("node_modules/foo/bar.js")));

        Ok(())
    }

    #[test]
    fn test_content_selector_from_selectors_uses_provided_patterns() -> Result<()> {
        let selectors = Selectors {
            include: vec!["src/**".to_string()],
            exclude: vec!["**/*.log".to_string()],
        };

        let selector = ContentSelector::from_selectors(Some(&selectors))?;

        // Should include files matching custom include patterns
        assert!(selector.should_include(Utf8Path::new("src/main.rs")));
        assert!(selector.should_include(Utf8Path::new("src/lib/utils.rs")));

        // Should NOT include files outside custom include patterns (no default includes)
        assert!(!selector.should_include(Utf8Path::new("README.md")));
        assert!(!selector.should_include(Utf8Path::new("config.yaml")));

        // Should exclude files matching custom exclude patterns
        assert!(!selector.should_include(Utf8Path::new("src/debug.log")));

        Ok(())
    }

    #[test]
    fn test_content_selector_from_selectors_empty_patterns() -> Result<()> {
        let selectors = Selectors {
            include: vec![],
            exclude: vec![],
        };

        let selector = ContentSelector::from_selectors(Some(&selectors))?;

        // With empty patterns, nothing matches include (empty globset never matches)
        // and nothing matches exclude (empty globset never matches)
        assert!(!selector.should_include(Utf8Path::new("README.md")));
        assert!(!selector.should_include(Utf8Path::new("src/main.rs")));

        Ok(())
    }
}
