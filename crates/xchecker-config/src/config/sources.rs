use std::collections::HashMap;
use std::path::PathBuf;

use super::Config;

/// Source of a configuration value for attribution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigSource {
    Cli,
    ConfigFile(PathBuf),
    Defaults,
    Programmatic,
}

impl std::fmt::Display for ConfigSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cli => write!(f, "CLI"),
            Self::ConfigFile(path) => write!(f, "config file ({})", path.display()),
            Self::Defaults => write!(f, "defaults"),
            Self::Programmatic => write!(f, "programmatic"),
        }
    }
}

impl ConfigSource {
    #[must_use]
    pub const fn stable(&self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::ConfigFile(_) => "config",
            Self::Defaults => "default",
            Self::Programmatic => "programmatic",
        }
    }
}

impl From<ConfigSource> for crate::types::ConfigSource {
    fn from(source: ConfigSource) -> Self {
        match source {
            ConfigSource::Cli => crate::types::ConfigSource::Cli,
            ConfigSource::ConfigFile(_) => crate::types::ConfigSource::Config,
            ConfigSource::Defaults => crate::types::ConfigSource::Default,
            ConfigSource::Programmatic => crate::types::ConfigSource::Programmatic,
        }
    }
}

impl Config {
    /// Get effective configuration as key-value pairs with source attribution
    #[must_use]
    pub fn effective_config(&self) -> HashMap<String, (String, String)> {
        let mut config = HashMap::new();

        // Helper to add config value with source
        let mut add_config = |key: &str, value: Option<&str>| {
            if let Some(val) = value {
                let source = self
                    .source_attribution
                    .get(key)
                    .map_or_else(|| ConfigSource::Defaults.stable().to_string(), |src| {
                        src.stable().to_string()
                    });
                config.insert(key.to_string(), (val.to_string(), source));
            }
        };

        // Add all configuration values
        add_config("model", self.defaults.model.as_deref());

        if let Some(max_turns) = self.defaults.max_turns {
            add_config("max_turns", Some(&max_turns.to_string()));
        }

        if let Some(packet_max_bytes) = self.defaults.packet_max_bytes {
            add_config("packet_max_bytes", Some(&packet_max_bytes.to_string()));
        }

        if let Some(packet_max_lines) = self.defaults.packet_max_lines {
            add_config("packet_max_lines", Some(&packet_max_lines.to_string()));
        }

        add_config("output_format", self.defaults.output_format.as_deref());

        if let Some(verbose) = self.defaults.verbose {
            add_config("verbose", Some(&verbose.to_string()));
        }

        add_config("runner_mode", self.runner.mode.as_deref());
        add_config("runner_distro", self.runner.distro.as_deref());
        add_config("claude_path", self.runner.claude_path.as_deref());

        // Add selector information
        let include_patterns = self.selectors.include.join(", ");
        let exclude_patterns = self.selectors.exclude.join(", ");

        let include_source = self
            .source_attribution
            .get("selectors_include")
            .map_or_else(|| ConfigSource::Defaults.stable().to_string(), |src| {
                src.stable().to_string()
            });
        let exclude_source = self
            .source_attribution
            .get("selectors_exclude")
            .map_or_else(|| ConfigSource::Defaults.stable().to_string(), |src| {
                src.stable().to_string()
            });

        config.insert(
            "selectors_include".to_string(),
            (include_patterns, include_source),
        );
        config.insert(
            "selectors_exclude".to_string(),
            (exclude_patterns, exclude_source),
        );

        config
    }
}
