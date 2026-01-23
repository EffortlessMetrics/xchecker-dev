use globset::Glob;

use crate::error::{ConfigError, XCheckerError};

use super::Selectors;

impl Default for Selectors {
    fn default() -> Self {
        Self {
            include: vec![
                "docs/**/SPEC*.md".to_string(),
                "docs/**/ADR*.md".to_string(),
                "README.md".to_string(),
                "SCHEMASET.*".to_string(),
                "**/Cargo.toml".to_string(),
                "**/*.core.yaml".to_string(),
            ],
            exclude: vec![
                "target/**".to_string(),
                "node_modules/**".to_string(),
                ".git/**".to_string(),
                "**/.DS_Store".to_string(),
                // Mandatory security exclusions (mirrored from xchecker-engine)
                "**/.env".to_string(),
                "**/.env.*".to_string(),
                "**/*.pem".to_string(),
                "**/id_rsa".to_string(),
                "**/id_ed25519".to_string(),
                "**/.ssh/**".to_string(),
                "**/*.pfx".to_string(),
                "**/*.p12".to_string(),
            ],
        }
    }
}

impl Selectors {
    pub(crate) fn validate(&self) -> Result<(), XCheckerError> {
        // Validate glob patterns in selectors
        for pattern in &self.include {
            Glob::new(pattern).map_err(|e| {
                XCheckerError::Config(ConfigError::InvalidValue {
                    key: "selectors.include".to_string(),
                    value: format!("Invalid glob pattern '{pattern}': {e}"),
                })
            })?;
        }

        for pattern in &self.exclude {
            Glob::new(pattern).map_err(|e| {
                XCheckerError::Config(ConfigError::InvalidValue {
                    key: "selectors.exclude".to_string(),
                    value: format!("Invalid glob pattern '{pattern}': {e}"),
                })
            })?;
        }

        Ok(())
    }
}
