use super::model::{PriorityRules, SelectedFile};
use crate::config::Selectors;
use crate::types::Priority;
use anyhow::{Context, Result};
use blake3::Hasher;
use camino::{Utf8Path, Utf8PathBuf};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::fs;
use tracing::warn;

/// Default maximum file size (10MB) to prevent DoS
const DEFAULT_MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

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
    /// Whether to follow symlinks (default: false for security)
    ///
    /// When false (default), symlinks are skipped during directory traversal.
    /// When true, symlinks are only followed if they resolve to paths within
    /// the base directory (sandbox), preventing path traversal attacks.
    allow_symlinks: bool,
    /// Maximum file size in bytes (default: 10MB)
    max_file_size: u64,
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
            allow_symlinks: false,
            max_file_size: DEFAULT_MAX_FILE_SIZE,
        })
    }

    /// Set the maximum file size limit in bytes.
    ///
    /// Files larger than this limit will be skipped during selection to prevent
    /// Denial of Service (DoS) attacks via memory exhaustion.
    ///
    /// Default is 10MB.
    #[must_use]
    pub const fn max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = size;
        self
    }

    /// Enable or disable symlink following during directory traversal.
    ///
    /// When enabled, symlinks are only followed if they resolve to paths
    /// within the base directory being scanned (sandbox validation).
    /// This prevents path traversal attacks while allowing legitimate
    /// intra-directory symlinks.
    ///
    /// Default is `false` (symlinks are skipped).
    #[must_use]
    pub const fn allow_symlinks(mut self, allow: bool) -> Self {
        self.allow_symlinks = allow;
        self
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
            allow_symlinks: false,
            max_file_size: DEFAULT_MAX_FILE_SIZE,
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
                    allow_symlinks: false,
                    max_file_size: DEFAULT_MAX_FILE_SIZE,
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

        // Walk the directory tree, passing root for symlink sandbox validation
        self.walk_directory(base_path, base_path, &mut files)?;

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

    /// Recursively walk directory and collect matching files.
    ///
    /// # Security
    ///
    /// This method implements symlink traversal protection:
    /// - When `allow_symlinks` is false (default), symlinks are skipped entirely
    /// - When `allow_symlinks` is true, symlinks are only followed if their
    ///   canonical path is within the `root` directory (sandbox validation)
    /// - Broken symlinks or canonicalization failures result in skipping (fail-closed)
    fn walk_directory(
        &self,
        root: &Utf8Path,
        dir: &Utf8Path,
        files: &mut Vec<SelectedFile>,
    ) -> Result<()> {
        self.walk_directory_inner(root, None, dir, files)
    }

    /// Inner implementation with pre-canonicalized root for performance.
    ///
    /// The `canonical_root` is computed once and passed through recursive calls
    /// to avoid repeated `fs::canonicalize(root)` syscalls on every symlink check.
    fn walk_directory_inner(
        &self,
        root: &Utf8Path,
        canonical_root: Option<&std::path::Path>,
        dir: &Utf8Path,
        files: &mut Vec<SelectedFile>,
    ) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        // Pre-canonicalize root once at the top level, reuse in recursive calls
        let owned_canonical_root;
        let canonical_root = match canonical_root {
            Some(cr) => cr,
            None => {
                owned_canonical_root = fs::canonicalize(root).ok();
                match &owned_canonical_root {
                    Some(cr) => cr.as_path(),
                    // If root can't be canonicalized, fail-closed: skip all symlink validation
                    None => {
                        return self.walk_directory_no_symlinks(dir, files);
                    }
                }
            }
        };

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = Utf8PathBuf::try_from(entry.path()).context("Invalid UTF-8 path")?;
            let file_type = entry.file_type()?;

            // Security: Handle symlinks with sandbox validation
            if file_type.is_symlink() {
                if !self.allow_symlinks {
                    // Secure default: skip all symlinks
                    continue;
                }

                // Symlinks allowed: verify target stays within sandbox (root)
                // Fail-closed: if canonicalization fails, skip the entry
                let is_safe = match fs::canonicalize(&path) {
                    Ok(canonical_path) => canonical_path.starts_with(canonical_root),
                    Err(_) => false, // Broken symlink or resolution error - skip
                };

                if !is_safe {
                    // Symlink points outside sandbox or is broken - skip
                    continue;
                }
            }

            // Recurse into directories (including validated symlinked directories)
            // Use DirEntry's file_type for non-symlinks to avoid extra stat call
            let is_dir = if file_type.is_symlink() {
                // For symlinks we already validated, check if target is a directory
                path.is_dir()
            } else {
                file_type.is_dir()
            };

            if is_dir {
                self.walk_directory_inner(root, Some(canonical_root), &path, files)?;
            } else if self.should_include(&path) {
                // Check file size before reading to prevent DoS
                // Note: fs::metadata follows symlinks, which is correct here (we want target size)
                let metadata = fs::metadata(&path).context("Failed to get file metadata")?;

                if !metadata.is_file() {
                    // Skip special files (pipes, devices, etc.) that might block reading
                    continue;
                }

                let priority = self.get_priority(&path);

                if metadata.len() > self.max_file_size {
                    // For upstream files (critical context), fail hard if they exceed the limit
                    if priority == Priority::Upstream {
                        return Err(anyhow::anyhow!(
                            "Upstream file {} exceeds size limit of {} bytes (size: {}). \
                             Critical context files must fit within the configured limit.",
                            path,
                            self.max_file_size,
                            metadata.len()
                        ));
                    }

                    warn!(
                        "Skipping large file: {} ({} bytes > limit {})",
                        path,
                        metadata.len(),
                        self.max_file_size
                    );
                    continue;
                }

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

    /// Fallback directory walker when root canonicalization fails.
    ///
    /// Skips all symlinks unconditionally (fail-closed security).
    fn walk_directory_no_symlinks(
        &self,
        dir: &Utf8Path,
        files: &mut Vec<SelectedFile>,
    ) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = Utf8PathBuf::try_from(entry.path()).context("Invalid UTF-8 path")?;
            let file_type = entry.file_type()?;

            // Skip all symlinks in fallback mode
            if file_type.is_symlink() {
                continue;
            }

            if file_type.is_dir() {
                self.walk_directory_no_symlinks(&path, files)?;
            } else if self.should_include(&path) {
                // Check file size before reading to prevent DoS
                let metadata = fs::metadata(&path).context("Failed to get file metadata")?;

                if !metadata.is_file() {
                    // Skip special files (pipes, devices, etc.)
                    continue;
                }

                let priority = self.get_priority(&path);

                if metadata.len() > self.max_file_size {
                    // For upstream files (critical context), fail hard if they exceed the limit
                    if priority == Priority::Upstream {
                        return Err(anyhow::anyhow!(
                            "Upstream file {} exceeds size limit of {} bytes (size: {}). \
                             Critical context files must fit within the configured limit.",
                            path,
                            self.max_file_size,
                            metadata.len()
                        ));
                    }

                    warn!(
                        "Skipping large file: {} ({} bytes > limit {})",
                        path,
                        metadata.len(),
                        self.max_file_size
                    );
                    continue;
                }

                let content = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read file: {path}"))?;

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

    #[test]
    fn test_large_file_skipped() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let base_path = Utf8PathBuf::try_from(temp_dir.path().to_path_buf())?;

        // Create a large file (larger than our test limit)
        let large_content = "x".repeat(1024); // 1KB
        fs::write(base_path.join("large.md"), &large_content)?;

        // Create a small file
        let small_content = "small";
        fs::write(base_path.join("small.md"), small_content)?;

        // Set limit to 500 bytes (smaller than large file)
        let selector = ContentSelector::new()?.max_file_size(500);
        let files = selector.select_files(&base_path)?;

        // Should only include the small file
        assert_eq!(files.len(), 1);
        assert!(files[0].path.as_str().contains("small.md"));

        Ok(())
    }

    // ===== Symlink Security Tests =====
    //
    // These tests verify the symlink traversal protection (CVE-style path traversal fix).
    // Some tests are Unix-only because Windows symlinks require special permissions.

    #[test]
    fn test_allow_symlinks_defaults_to_false() -> Result<()> {
        let selector = ContentSelector::new()?;
        // The field is private, but we can verify behavior through selection
        // Default should skip symlinks entirely
        assert!(!selector.allow_symlinks);
        Ok(())
    }

    #[test]
    fn test_allow_symlinks_builder_method() -> Result<()> {
        let selector = ContentSelector::new()?.allow_symlinks(true);
        assert!(selector.allow_symlinks);

        let selector2 = ContentSelector::new()?.allow_symlinks(false);
        assert!(!selector2.allow_symlinks);
        Ok(())
    }

    #[cfg(unix)]
    mod symlink_tests {
        use super::*;
        use std::os::unix::fs::symlink;

        #[test]
        fn test_symlinks_skipped_by_default() -> Result<()> {
            let temp_dir = TempDir::new()?;
            let base_path = Utf8PathBuf::try_from(temp_dir.path().to_path_buf())?;

            // Create a real file
            fs::write(base_path.join("real.md"), "# Real file")?;

            // Create a symlink to the real file
            symlink(base_path.join("real.md"), base_path.join("link.md"))?;

            let selector = ContentSelector::new()?;
            let files = selector.select_files(&base_path)?;

            // Should only include the real file, not the symlink
            assert_eq!(files.len(), 1);
            assert!(files[0].path.as_str().contains("real.md"));

            Ok(())
        }

        #[test]
        fn test_symlinks_followed_when_allowed_and_safe() -> Result<()> {
            let temp_dir = TempDir::new()?;
            let base_path = Utf8PathBuf::try_from(temp_dir.path().to_path_buf())?;

            // Create a subdirectory with a file
            fs::create_dir_all(base_path.join("subdir"))?;
            fs::write(base_path.join("subdir/target.md"), "# Target file")?;

            // Create a symlink within the base directory pointing to the subdirectory file
            symlink(
                base_path.join("subdir/target.md"),
                base_path.join("safe_link.md"),
            )?;

            let selector = ContentSelector::new()?.allow_symlinks(true);
            let files = selector.select_files(&base_path)?;

            // Should include both the real file and the safe symlink
            assert_eq!(files.len(), 2);
            let paths: Vec<_> = files.iter().map(|f| f.path.as_str()).collect();
            assert!(paths.iter().any(|p| p.contains("target.md")));
            assert!(paths.iter().any(|p| p.contains("safe_link.md")));

            Ok(())
        }

        #[test]
        fn test_symlinks_outside_sandbox_rejected_even_when_allowed() -> Result<()> {
            let temp_dir = TempDir::new()?;
            let base_path = Utf8PathBuf::try_from(temp_dir.path().to_path_buf())?;

            // Create a symlink pointing outside the base directory (to /etc/passwd or similar)
            // Using a safe target that exists on most Unix systems
            let outside_target = if cfg!(target_os = "macos") {
                "/etc/hosts"
            } else {
                "/etc/passwd"
            };

            if std::path::Path::new(outside_target).exists() {
                symlink(outside_target, base_path.join("escape.md"))?;

                let selector = ContentSelector::new()?.allow_symlinks(true);
                let files = selector.select_files(&base_path)?;

                // Should NOT include the symlink pointing outside
                assert!(
                    files.is_empty() || !files.iter().any(|f| f.path.as_str().contains("escape"))
                );
            }

            Ok(())
        }

        #[test]
        fn test_broken_symlinks_safely_skipped() -> Result<()> {
            let temp_dir = TempDir::new()?;
            let base_path = Utf8PathBuf::try_from(temp_dir.path().to_path_buf())?;

            // Create a symlink to a non-existent target
            symlink(
                base_path.join("nonexistent.md"),
                base_path.join("broken_link.md"),
            )?;

            // Create a real file too
            fs::write(base_path.join("real.md"), "# Real file")?;

            let selector = ContentSelector::new()?.allow_symlinks(true);
            let files = selector.select_files(&base_path)?;

            // Should only include the real file, broken symlink should be skipped
            assert_eq!(files.len(), 1);
            assert!(files[0].path.as_str().contains("real.md"));

            Ok(())
        }

        #[test]
        fn test_symlink_directory_traversal_blocked() -> Result<()> {
            let temp_dir = TempDir::new()?;
            let base_path = Utf8PathBuf::try_from(temp_dir.path().to_path_buf())?;

            // Create a separate directory outside the base
            let outside_dir = TempDir::new()?;
            let outside_path = Utf8PathBuf::try_from(outside_dir.path().to_path_buf())?;
            fs::write(outside_path.join("secret.md"), "# Secret content")?;

            // Create a symlink to the outside directory
            symlink(&outside_path, base_path.join("escape_dir"))?;

            let selector = ContentSelector::new()?.allow_symlinks(true);
            let files = selector.select_files(&base_path)?;

            // Should NOT include files from the outside directory via symlink
            assert!(files.is_empty() || !files.iter().any(|f| f.content.contains("Secret")));

            Ok(())
        }
    }
}
