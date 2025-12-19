//! Secret redaction system for protecting sensitive information in packets
//!
//! This module implements configurable secret pattern detection and redaction
//! to prevent sensitive information from being included in Claude CLI packets.

use crate::config::Config;
use crate::error::XCheckerError;
use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Secret redactor with configurable patterns for detecting and redacting sensitive information
#[derive(Debug, Clone)]
pub struct SecretRedactor {
    /// Default secret patterns with their IDs
    default_patterns: HashMap<String, Regex>,
    /// Extra patterns added via configuration
    extra_patterns: HashMap<String, Regex>,
    /// Patterns to ignore (suppress detection)
    ignored_patterns: Vec<String>,
}

/// Information about a detected secret
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretMatch {
    /// Pattern ID that matched
    pub pattern_id: String,
    /// File path where secret was found
    pub file_path: String,
    /// Line number (1-based)
    pub line_number: usize,
    /// Column range within the line
    pub column_range: (usize, usize),
    /// Context around the match (never includes the actual secret)
    pub context: String,
}

/// Result of redaction operation
#[derive(Debug, Clone)]
pub struct RedactionResult {
    /// Redacted content with secrets replaced
    pub content: String,
    /// List of detected secrets (for logging)
    #[allow(dead_code)] // Reserved for detailed redaction reporting
    pub matches: Vec<SecretMatch>,
    /// Whether any secrets were found and redacted
    #[allow(dead_code)] // Reserved for structured reporting
    pub has_secrets: bool,
}

impl SecretRedactor {
    /// Create a new `SecretRedactor` with default patterns
    ///
    /// # Built-in Pattern Categories
    ///
    /// The default patterns cover the following secret categories as documented in SECURITY.md:
    ///
    /// ## AWS Credentials (FR-SEC-2)
    /// - `aws_access_key`: Access key IDs (AKIA prefix)
    /// - `aws_secret_key`: Secret access key environment variable assignments
    /// - `aws_secret_key_value`: 40-character base64 secret key values
    /// - `aws_session_token`: Session token environment variable assignments
    /// - `aws_session_token_value`: Session token values (longer base64 strings)
    ///
    /// ## GCP Credentials (FR-SEC-2)
    /// - `gcp_service_account_key`: Service account private key markers
    /// - `gcp_api_key`: API keys (AIza prefix)
    /// - `gcp_oauth_client_secret`: OAuth client secrets
    ///
    /// ## Azure Credentials (FR-SEC-2)
    /// - `azure_storage_key`: Storage account keys (88-char base64)
    /// - `azure_connection_string`: Connection strings with AccountKey
    /// - `azure_sas_token`: Shared Access Signature tokens
    /// - `azure_client_secret`: Client secrets in environment variables
    ///
    /// ## Generic API Tokens (FR-SEC-2)
    /// - `bearer_token`: Bearer authentication tokens
    /// - `api_key_header`: API-Key header values
    /// - `authorization_basic`: Basic auth credentials
    /// - `oauth_token`: OAuth access/refresh tokens
    /// - `jwt_token`: JSON Web Tokens (eyJ prefix)
    ///
    /// ## Database Connection Strings (FR-SEC-2)
    /// - `postgres_url`: PostgreSQL connection URLs with credentials
    /// - `mysql_url`: MySQL connection URLs with credentials
    /// - `sqlserver_url`: SQL Server connection URLs with credentials
    /// - `mongodb_url`: MongoDB connection URLs with credentials
    /// - `redis_url`: Redis connection URLs with credentials
    ///
    /// ## SSH and PEM Secrets (FR-SEC-2)
    /// - `ssh_private_key`: SSH private key markers (BEGIN/END blocks)
    /// - `rsa_private_key`: RSA private key markers
    /// - `ec_private_key`: EC private key markers
    /// - `pem_private_key`: Generic PEM private key markers
    /// - `openssh_private_key`: OpenSSH private key markers
    ///
    /// ## Platform-Specific Tokens (FR-SEC-2)
    /// - `github_pat`: GitHub personal access tokens
    /// - `github_oauth`: GitHub OAuth tokens
    /// - `github_app_token`: GitHub App tokens
    /// - `gitlab_token`: GitLab personal/project tokens
    /// - `slack_token`: Slack bot/user tokens
    /// - `stripe_key`: Stripe API keys (sk_live/sk_test)
    /// - `twilio_key`: Twilio API keys
    /// - `sendgrid_key`: SendGrid API keys
    /// - `npm_token`: NPM authentication tokens
    /// - `pypi_token`: PyPI API tokens
    /// - `nuget_key`: NuGet API keys
    /// - `docker_auth`: Docker registry auth tokens
    pub fn new() -> Result<Self> {
        let mut default_patterns = HashMap::new();

        // =========================================================================
        // AWS Credentials
        // =========================================================================

        // AWS access key IDs: AKIA[0-9A-Z]{16}
        // Standard AWS access key format used for IAM users
        default_patterns.insert(
            "aws_access_key".to_string(),
            Regex::new(r"AKIA[0-9A-Z]{16}").context("Failed to compile AWS access key regex")?,
        );

        // AWS secret access key environment variable assignments
        // Catches AWS_SECRET_ACCESS_KEY=... patterns in config files
        default_patterns.insert(
            "aws_secret_key".to_string(),
            Regex::new(r"AWS_SECRET_ACCESS_KEY[=:][A-Za-z0-9/+=]{40}")
                .context("Failed to compile AWS secret key regex")?,
        );

        // AWS secret access key values (40-char base64)
        // Standalone secret key values that look like AWS secrets
        default_patterns.insert(
            "aws_secret_key_value".to_string(),
            Regex::new(r"(?i)(?:aws_secret|secret_access_key)[=:][A-Za-z0-9/+=]{40}")
                .context("Failed to compile AWS secret key value regex")?,
        );

        // AWS session tokens (temporary credentials)
        // Session tokens are longer than secret keys
        default_patterns.insert(
            "aws_session_token".to_string(),
            Regex::new(r"(?i)AWS_SESSION_TOKEN[=:][A-Za-z0-9/+=]{100,}")
                .context("Failed to compile AWS session token regex")?,
        );

        // AWS session token values (longer base64 strings from STS)
        default_patterns.insert(
            "aws_session_token_value".to_string(),
            Regex::new(r"(?i)(?:session_token|security_token)[=:][A-Za-z0-9/+=]{100,}")
                .context("Failed to compile AWS session token value regex")?,
        );

        // =========================================================================
        // GCP Credentials
        // =========================================================================

        // GCP service account private key markers
        // Detects the BEGIN PRIVATE KEY marker in service account JSON files
        default_patterns.insert(
            "gcp_service_account_key".to_string(),
            Regex::new(r"-----BEGIN (RSA )?PRIVATE KEY-----")
                .context("Failed to compile GCP service account key regex")?,
        );

        // GCP API keys (AIza prefix)
        // Standard format for Google API keys
        default_patterns.insert(
            "gcp_api_key".to_string(),
            Regex::new(r"AIza[0-9A-Za-z_-]{35}").context("Failed to compile GCP API key regex")?,
        );

        // GCP OAuth client secrets
        // Client secrets in OAuth configurations
        default_patterns.insert(
            "gcp_oauth_client_secret".to_string(),
            Regex::new(r"(?i)client_secret[=:][A-Za-z0-9_-]{24,}")
                .context("Failed to compile GCP OAuth client secret regex")?,
        );

        // =========================================================================
        // Azure Credentials
        // =========================================================================

        // Azure storage account keys (88-char base64)
        // Storage keys are base64 encoded and typically 88 characters
        default_patterns.insert(
            "azure_storage_key".to_string(),
            Regex::new(r"(?i)(?:AccountKey|storage_key)[=:][A-Za-z0-9/+=]{86,90}")
                .context("Failed to compile Azure storage key regex")?,
        );

        // Azure connection strings with AccountKey
        // Full connection string format used by Azure Storage
        default_patterns.insert(
            "azure_connection_string".to_string(),
            Regex::new(r"DefaultEndpointsProtocol=https?;AccountName=[^;]+;AccountKey=[A-Za-z0-9/+=]{86,90}")
                .context("Failed to compile Azure connection string regex")?,
        );

        // Azure SAS tokens
        // Shared Access Signature tokens contain sig= parameter
        default_patterns.insert(
            "azure_sas_token".to_string(),
            Regex::new(r"[?&]sig=[A-Za-z0-9%/+=]{40,}")
                .context("Failed to compile Azure SAS token regex")?,
        );

        // Azure client secrets
        // Client secrets used in Azure AD authentication
        default_patterns.insert(
            "azure_client_secret".to_string(),
            Regex::new(r"(?i)(?:AZURE_CLIENT_SECRET|client_secret)[=:][A-Za-z0-9~._-]{34,}")
                .context("Failed to compile Azure client secret regex")?,
        );

        // =========================================================================
        // Generic API Tokens and OAuth
        // =========================================================================

        // Bearer tokens in Authorization headers
        // Standard OAuth 2.0 bearer token format
        default_patterns.insert(
            "bearer_token".to_string(),
            Regex::new(r"Bearer [A-Za-z0-9._-]{20,}")
                .context("Failed to compile Bearer token regex")?,
        );

        // API-Key header values
        // Common API key header format
        default_patterns.insert(
            "api_key_header".to_string(),
            Regex::new(r"(?i)(?:x-api-key|api-key|apikey)[=:][A-Za-z0-9_-]{20,}")
                .context("Failed to compile API key header regex")?,
        );

        // Basic authentication credentials
        // Base64 encoded username:password
        default_patterns.insert(
            "authorization_basic".to_string(),
            Regex::new(r"Basic [A-Za-z0-9+/=]{20,}")
                .context("Failed to compile Basic auth regex")?,
        );

        // OAuth access/refresh tokens
        // Generic OAuth token patterns
        default_patterns.insert(
            "oauth_token".to_string(),
            Regex::new(r"(?i)(?:access_token|refresh_token)[=:][A-Za-z0-9._-]{20,}")
                .context("Failed to compile OAuth token regex")?,
        );

        // JSON Web Tokens (JWT)
        // JWTs start with eyJ (base64 encoded {"alg":...)
        default_patterns.insert(
            "jwt_token".to_string(),
            Regex::new(r"eyJ[A-Za-z0-9_-]*\.eyJ[A-Za-z0-9_-]*\.[A-Za-z0-9_-]*")
                .context("Failed to compile JWT token regex")?,
        );

        // =========================================================================
        // Database Connection Strings
        // =========================================================================

        // PostgreSQL connection URLs with credentials
        // postgres://user:password@host:port/database
        default_patterns.insert(
            "postgres_url".to_string(),
            Regex::new(r"postgres(?:ql)?://[^:]+:[^@]+@[^\s]+")
                .context("Failed to compile PostgreSQL URL regex")?,
        );

        // MySQL connection URLs with credentials
        // mysql://user:password@host:port/database
        default_patterns.insert(
            "mysql_url".to_string(),
            Regex::new(r"mysql://[^:]+:[^@]+@[^\s]+")
                .context("Failed to compile MySQL URL regex")?,
        );

        // SQL Server connection URLs with credentials
        // sqlserver://user:password@host:port/database or mssql://...
        default_patterns.insert(
            "sqlserver_url".to_string(),
            Regex::new(r"(?:sqlserver|mssql)://[^:]+:[^@]+@[^\s]+")
                .context("Failed to compile SQL Server URL regex")?,
        );

        // MongoDB connection URLs with credentials
        // mongodb://user:password@host:port/database
        default_patterns.insert(
            "mongodb_url".to_string(),
            Regex::new(r"mongodb(\+srv)?://[^:]+:[^@]+@[^\s]+")
                .context("Failed to compile MongoDB URL regex")?,
        );

        // Redis connection URLs with credentials
        // redis://user:password@host:port or rediss://...
        default_patterns.insert(
            "redis_url".to_string(),
            Regex::new(r"rediss?://[^:]*:[^@]+@[^\s]+")
                .context("Failed to compile Redis URL regex")?,
        );

        // =========================================================================
        // SSH and PEM Private Keys
        // =========================================================================

        // SSH private key markers
        // Standard OpenSSH private key format
        default_patterns.insert(
            "ssh_private_key".to_string(),
            Regex::new(r"-----BEGIN (?:OPENSSH |DSA |EC |RSA )?PRIVATE KEY-----")
                .context("Failed to compile SSH private key regex")?,
        );

        // RSA private key markers (also catches GCP service account keys)
        default_patterns.insert(
            "rsa_private_key".to_string(),
            Regex::new(r"-----BEGIN RSA PRIVATE KEY-----")
                .context("Failed to compile RSA private key regex")?,
        );

        // EC private key markers
        default_patterns.insert(
            "ec_private_key".to_string(),
            Regex::new(r"-----BEGIN EC PRIVATE KEY-----")
                .context("Failed to compile EC private key regex")?,
        );

        // Generic PEM private key markers
        default_patterns.insert(
            "pem_private_key".to_string(),
            Regex::new(r"-----BEGIN PRIVATE KEY-----")
                .context("Failed to compile PEM private key regex")?,
        );

        // OpenSSH private key markers (newer format)
        default_patterns.insert(
            "openssh_private_key".to_string(),
            Regex::new(r"-----BEGIN OPENSSH PRIVATE KEY-----")
                .context("Failed to compile OpenSSH private key regex")?,
        );

        // =========================================================================
        // Platform-Specific Tokens
        // =========================================================================

        // GitHub personal access tokens (classic): ghp_[A-Za-z0-9]{36}
        default_patterns.insert(
            "github_pat".to_string(),
            Regex::new(r"ghp_[A-Za-z0-9]{36}").context("Failed to compile GitHub PAT regex")?,
        );

        // GitHub OAuth tokens: gho_[A-Za-z0-9]{36}
        default_patterns.insert(
            "github_oauth".to_string(),
            Regex::new(r"gho_[A-Za-z0-9]{36}").context("Failed to compile GitHub OAuth regex")?,
        );

        // GitHub App tokens: ghu_[A-Za-z0-9]{36} or ghs_[A-Za-z0-9]{36}
        default_patterns.insert(
            "github_app_token".to_string(),
            Regex::new(r"gh[us]_[A-Za-z0-9]{36}")
                .context("Failed to compile GitHub App token regex")?,
        );

        // GitLab personal/project tokens: glpat-[A-Za-z0-9_-]{20,}
        default_patterns.insert(
            "gitlab_token".to_string(),
            Regex::new(r"glpat-[A-Za-z0-9_-]{20,}")
                .context("Failed to compile GitLab token regex")?,
        );

        // Slack tokens: xox[baprs]-[A-Za-z0-9-]+
        default_patterns.insert(
            "slack_token".to_string(),
            Regex::new(r"xox[baprs]-[A-Za-z0-9-]+")
                .context("Failed to compile Slack token regex")?,
        );

        // Stripe API keys: sk_live_[A-Za-z0-9]{24,} or sk_test_[A-Za-z0-9]{24,}
        default_patterns.insert(
            "stripe_key".to_string(),
            Regex::new(r"sk_(?:live|test)_[A-Za-z0-9]{24,}")
                .context("Failed to compile Stripe key regex")?,
        );

        // Twilio API keys: SK[A-Za-z0-9]{32}
        default_patterns.insert(
            "twilio_key".to_string(),
            Regex::new(r"SK[A-Za-z0-9]{32}").context("Failed to compile Twilio key regex")?,
        );

        // SendGrid API keys: SG\.[A-Za-z0-9_-]{22}\.[A-Za-z0-9_-]{43}
        default_patterns.insert(
            "sendgrid_key".to_string(),
            Regex::new(r"SG\.[A-Za-z0-9_-]{22}\.[A-Za-z0-9_-]{43}")
                .context("Failed to compile SendGrid key regex")?,
        );

        // NPM tokens: npm_[A-Za-z0-9]{36}
        default_patterns.insert(
            "npm_token".to_string(),
            Regex::new(r"npm_[A-Za-z0-9]{36}").context("Failed to compile NPM token regex")?,
        );

        // PyPI API tokens: pypi-[A-Za-z0-9_-]{50,}
        default_patterns.insert(
            "pypi_token".to_string(),
            Regex::new(r"pypi-[A-Za-z0-9_-]{50,}").context("Failed to compile PyPI token regex")?,
        );

        // NuGet API keys (typically 46 chars)
        default_patterns.insert(
            "nuget_key".to_string(),
            Regex::new(r"(?i)nuget_?(?:api_?)?key[=:][A-Za-z0-9]{46}")
                .context("Failed to compile NuGet key regex")?,
        );

        // Docker registry auth tokens (base64 encoded)
        default_patterns.insert(
            "docker_auth".to_string(),
            Regex::new(r#""auth":\s*"[A-Za-z0-9+/=]{20,}""#)
                .context("Failed to compile Docker auth regex")?,
        );

        Ok(Self {
            default_patterns,
            extra_patterns: HashMap::new(),
            ignored_patterns: Vec::new(),
        })
    }

    /// Create a `SecretRedactor` from a `Config`.
    ///
    /// This method creates a redactor with the default patterns plus any
    /// extra patterns and ignore patterns specified in the config's security section.
    ///
    /// # Arguments
    /// * `config` - The configuration containing security settings
    ///
    /// # Returns
    /// A configured `SecretRedactor` instance
    ///
    /// # Errors
    /// Returns an error if any of the extra patterns fail to compile as regex.
    ///
    /// # Example
    /// ```rust,no_run
    /// use xchecker::Config;
    /// use xchecker::redaction::SecretRedactor;
    ///
    /// let config = Config::builder()
    ///     .extra_secret_patterns(vec!["CUSTOM_[A-Z0-9]{32}".to_string()])
    ///     .ignore_secret_patterns(vec!["test_token".to_string()])
    ///     .build()
    ///     .expect("Failed to build config");
    ///
    /// let redactor = SecretRedactor::from_config(&config)
    ///     .expect("Failed to create redactor");
    /// ```
    pub fn from_config(config: &Config) -> Result<Self> {
        let mut redactor = Self::new()?;

        // Add extra patterns from config
        for (idx, pattern) in config.security.extra_secret_patterns.iter().enumerate() {
            let pattern_id = format!("extra_pattern_{}", idx);
            redactor.add_extra_pattern(pattern_id, pattern)?;
        }

        // Add ignored patterns from config
        for pattern_id in &config.security.ignore_secret_patterns {
            redactor.add_ignored_pattern(pattern_id.clone());
        }

        Ok(redactor)
    }

    /// Redact secrets from a string, replacing them with *** (simplified version for user-facing strings)
    ///
    /// This is a lightweight redaction function for use in error messages, logs, and other
    /// user-facing output. It replaces detected secrets with "***" without detailed tracking.
    ///
    /// # Arguments
    /// * `text` - The text to redact
    ///
    /// # Returns
    /// The redacted text with secrets replaced by "***"
    #[must_use]
    pub fn redact_string(&self, text: &str) -> String {
        let mut redacted = text.to_string();

        // Apply default patterns
        for (pattern_id, regex) in &self.default_patterns {
            if self.is_pattern_ignored(pattern_id) {
                continue;
            }
            redacted = regex.replace_all(&redacted, "***").to_string();
        }

        // Apply extra patterns
        for (pattern_id, regex) in &self.extra_patterns {
            if self.is_pattern_ignored(pattern_id) {
                continue;
            }
            redacted = regex.replace_all(&redacted, "***").to_string();
        }

        redacted
    }

    /// Redact secrets from a vector of strings
    /// Extended API for batch operations
    ///
    /// # Arguments
    /// * `strings` - Vector of strings to redact
    ///
    /// # Returns
    /// Vector of redacted strings
    #[must_use]
    #[allow(dead_code)] // Extended API for batch redaction
    pub fn redact_strings(&self, strings: &[String]) -> Vec<String> {
        strings.iter().map(|s| self.redact_string(s)).collect()
    }

    /// Redact secrets from an optional string
    /// Extended API for optional field handling
    ///
    /// # Arguments
    /// * `text` - Optional string to redact
    ///
    /// # Returns
    /// Optional redacted string (None if input was None)
    #[must_use]
    #[allow(dead_code)] // Extended API for optional fields
    pub fn redact_optional(&self, text: &Option<String>) -> Option<String> {
        text.as_ref().map(|s| self.redact_string(s))
    }

    /// Add an extra secret pattern to detect
    /// Extended API for custom patterns
    #[allow(dead_code)] // Extended API for custom pattern configuration
    pub fn add_extra_pattern(&mut self, pattern_id: String, pattern: &str) -> Result<()> {
        let regex = Regex::new(pattern).with_context(|| {
            format!("Failed to compile extra pattern '{pattern_id}': {pattern}")
        })?;

        self.extra_patterns.insert(pattern_id, regex);
        Ok(())
    }

    /// Add a pattern to ignore (suppress detection)
    /// Extended API for pattern suppression
    #[allow(dead_code)] // Extended API for pattern configuration
    pub fn add_ignored_pattern(&mut self, pattern: String) {
        self.ignored_patterns.push(pattern);
    }

    /// Scan content for secrets and return matches without redacting
    pub fn scan_for_secrets(&self, content: &str, file_path: &str) -> Result<Vec<SecretMatch>> {
        let mut matches = Vec::new();

        // Scan with default patterns
        for (pattern_id, regex) in &self.default_patterns {
            if self.is_pattern_ignored(pattern_id) {
                continue;
            }

            let pattern_matches =
                self.find_matches_in_content(content, file_path, pattern_id, regex)?;
            matches.extend(pattern_matches);
        }

        // Scan with extra patterns
        for (pattern_id, regex) in &self.extra_patterns {
            if self.is_pattern_ignored(pattern_id) {
                continue;
            }

            let pattern_matches =
                self.find_matches_in_content(content, file_path, pattern_id, regex)?;
            matches.extend(pattern_matches);
        }

        Ok(matches)
    }

    /// Redact secrets from content, replacing them with placeholder text
    pub fn redact_content(&self, content: &str, file_path: &str) -> Result<RedactionResult> {
        let matches = self.scan_for_secrets(content, file_path)?;

        if matches.is_empty() {
            return Ok(RedactionResult {
                content: content.to_string(),
                matches,
                has_secrets: false,
            });
        }

        // Sort matches by position (reverse order to maintain indices during replacement)
        let mut sorted_matches = matches.clone();
        sorted_matches.sort_by(|a, b| {
            b.line_number
                .cmp(&a.line_number)
                .then_with(|| b.column_range.0.cmp(&a.column_range.0))
        });

        let mut redacted_content = content.to_string();
        let lines: Vec<&str> = content.lines().collect();

        // Replace secrets with redaction markers
        for secret_match in &sorted_matches {
            if let Some(line) = lines.get(secret_match.line_number - 1) {
                let (start, end) = secret_match.column_range;
                if start < line.len() && end <= line.len() {
                    let before = &line[..start];
                    let after = &line[end..];
                    let redacted_line =
                        format!("{}[REDACTED:{}]{}", before, secret_match.pattern_id, after);

                    // Replace the line in the content
                    let line_start = content
                        .lines()
                        .take(secret_match.line_number - 1)
                        .map(|l| l.len() + 1) // +1 for newline
                        .sum::<usize>();
                    let line_end = line_start + line.len();

                    redacted_content.replace_range(line_start..line_end, &redacted_line);
                }
            }
        }

        Ok(RedactionResult {
            content: redacted_content,
            matches,
            has_secrets: true,
        })
    }

    /// Check if any secrets would be detected in the content (fail-fast check)
    pub fn has_secrets(&self, content: &str, file_path: &str) -> Result<bool> {
        let matches = self.scan_for_secrets(content, file_path)?;
        Ok(!matches.is_empty())
    }

    /// Check if a pattern ID is in the ignored list
    fn is_pattern_ignored(&self, pattern_id: &str) -> bool {
        self.ignored_patterns
            .iter()
            .any(|ignored| ignored == pattern_id)
    }

    /// Find all matches for a specific pattern in content
    fn find_matches_in_content(
        &self,
        content: &str,
        file_path: &str,
        pattern_id: &str,
        regex: &Regex,
    ) -> Result<Vec<SecretMatch>> {
        let mut matches = Vec::new();

        for (line_number, line) in content.lines().enumerate() {
            for regex_match in regex.find_iter(line) {
                let start = regex_match.start();
                let end = regex_match.end();

                // Create context without revealing the secret
                let context = self.create_safe_context(line, start, end);

                matches.push(SecretMatch {
                    pattern_id: pattern_id.to_string(),
                    file_path: file_path.to_string(),
                    line_number: line_number + 1, // 1-based line numbers
                    column_range: (start, end),
                    context,
                });
            }
        }

        Ok(matches)
    }

    /// Create safe context around a match without revealing the secret
    fn create_safe_context(&self, line: &str, start: usize, end: usize) -> String {
        let before_len = 10; // Show up to 10 chars before
        let after_len = 10; // Show up to 10 chars after

        let context_start = start.saturating_sub(before_len);
        let context_end = std::cmp::min(line.len(), end + after_len);

        let before = &line[context_start..start];
        let after = &line[end..context_end];

        format!("{before}[REDACTED]{after}")
    }

    /// Get list of all pattern IDs (for configuration and logging)
    /// Extended API for pattern introspection
    #[must_use]
    #[allow(dead_code)] // Extended API for pattern introspection
    pub fn get_pattern_ids(&self) -> Vec<String> {
        let mut ids = Vec::new();
        ids.extend(self.default_patterns.keys().cloned());
        ids.extend(self.extra_patterns.keys().cloned());
        ids.sort();
        ids
    }

    /// Get list of ignored pattern IDs
    /// Extended API for pattern introspection
    #[must_use]
    #[allow(dead_code)] // Extended API for pattern introspection
    pub fn get_ignored_patterns(&self) -> &[String] {
        &self.ignored_patterns
    }
}

impl Default for SecretRedactor {
    fn default() -> Self {
        Self::new().expect("Failed to create default SecretRedactor")
    }
}

static DEFAULT_REDACTOR: LazyLock<SecretRedactor> =
    LazyLock::new(|| SecretRedactor::new().expect("Failed to create default SecretRedactor"));

/// Get a process-global default redactor instance.
///
/// This is used by helpers like [`redact_user_string`] when a configured redactor
/// is not available.
#[must_use]
pub fn default_redactor() -> &'static SecretRedactor {
    &DEFAULT_REDACTOR
}

/// Create a `SecretRedactor` error for detected secrets
#[must_use]
pub fn create_secret_detected_error(matches: &[SecretMatch]) -> XCheckerError {
    if matches.is_empty() {
        return XCheckerError::SecretDetected {
            pattern: "unknown".to_string(),
            location: "unknown".to_string(),
        };
    }

    let first_match = &matches[0];
    let location = format!(
        "{}:{}:{}",
        first_match.file_path, first_match.line_number, first_match.column_range.0
    );

    XCheckerError::SecretDetected {
        pattern: first_match.pattern_id.clone(),
        location,
    }
}

/// Global redaction function for user-facing strings
///
/// This function provides a simple way to redact secrets from any user-facing string
/// before it is displayed, logged, or persisted. It uses a default `SecretRedactor`
/// instance with all standard patterns enabled.
///
/// # Arguments
/// * `text` - The text to redact
///
/// # Returns
/// The redacted text with secrets replaced by "***"
///
/// # Example
/// ```
/// use xchecker::redaction::redact_user_string;
///
/// let error_msg = "Failed to authenticate with token ghp_1234567890123456789012345678901234567890";
/// let safe_msg = redact_user_string(&error_msg);
/// assert!(safe_msg.contains("***"));
/// assert!(!safe_msg.contains("ghp_"));
/// ```
#[must_use]
pub fn redact_user_string(text: &str) -> String {
    default_redactor().redact_string(text)
}

/// Global redaction function for optional user-facing strings
///
/// # Arguments
/// * `text` - Optional text to redact
///
/// # Returns
/// Optional redacted text (None if input was None)
#[must_use]
#[allow(dead_code)] // Duplicate of SecretRedactor method, candidate for removal
pub fn redact_user_optional(text: &Option<String>) -> Option<String> {
    text.as_ref().map(|s| redact_user_string(s))
}

/// Global redaction function for vectors of user-facing strings
///
/// # Arguments
/// * `strings` - Vector of strings to redact
///
/// # Returns
/// Vector of redacted strings
#[must_use]
#[allow(dead_code)] // Duplicate of SecretRedactor method, candidate for removal
pub fn redact_user_strings(strings: &[String]) -> Vec<String> {
    strings.iter().map(|s| redact_user_string(s)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_redactor_creation() {
        let redactor = SecretRedactor::new().unwrap();
        let pattern_ids = redactor.get_pattern_ids();

        // Should have all default patterns
        assert!(pattern_ids.contains(&"github_pat".to_string()));
        assert!(pattern_ids.contains(&"aws_access_key".to_string()));
        assert!(pattern_ids.contains(&"aws_secret_key".to_string()));
        assert!(pattern_ids.contains(&"slack_token".to_string()));
        assert!(pattern_ids.contains(&"bearer_token".to_string()));
    }

    #[test]
    fn test_github_pat_detection() {
        let redactor = SecretRedactor::new().unwrap();
        let content = "token = ghp_1234567890123456789012345678901234567890";

        let matches = redactor.scan_for_secrets(content, "test.txt").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].pattern_id, "github_pat");
        assert_eq!(matches[0].line_number, 1);
    }

    #[test]
    fn test_aws_access_key_detection() {
        let redactor = SecretRedactor::new().unwrap();
        let content = "access_key = AKIA1234567890123456";

        let matches = redactor.scan_for_secrets(content, "test.txt").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].pattern_id, "aws_access_key");
    }

    #[test]
    fn test_aws_secret_key_detection() {
        let redactor = SecretRedactor::new().unwrap();
        let content = "AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";

        let matches = redactor.scan_for_secrets(content, "test.txt").unwrap();
        // May match multiple patterns (aws_secret_key and aws_secret_key_value)
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.pattern_id == "aws_secret_key" || m.pattern_id == "aws_secret_key_value"));
    }

    #[test]
    fn test_slack_token_detection() {
        let redactor = SecretRedactor::new().unwrap();
        let content = "slack_token = xoxb-1234567890-abcdefghijklmnop";

        let matches = redactor.scan_for_secrets(content, "test.txt").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].pattern_id, "slack_token");
    }

    #[test]
    fn test_bearer_token_detection() {
        let redactor = SecretRedactor::new().unwrap();
        let content = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";

        let matches = redactor.scan_for_secrets(content, "test.txt").unwrap();
        assert!(!matches.is_empty());
        // Should match bearer_token pattern
        assert!(matches.iter().any(|m| m.pattern_id == "bearer_token"));
    }

    #[test]
    fn test_no_secrets_detected() {
        let redactor = SecretRedactor::new().unwrap();
        let content = "This is just normal content with no secrets.";

        let matches = redactor.scan_for_secrets(content, "test.txt").unwrap();
        assert_eq!(matches.len(), 0);
        assert!(!redactor.has_secrets(content, "test.txt").unwrap());
    }

    #[test]
    fn test_extra_pattern_addition() {
        let mut redactor = SecretRedactor::new().unwrap();
        redactor
            .add_extra_pattern("custom_key".to_string(), r"CUSTOM_[A-Z0-9]{10}")
            .unwrap();

        let content = "key = CUSTOM_1234567890";
        let matches = redactor.scan_for_secrets(content, "test.txt").unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].pattern_id, "custom_key");
    }

    #[test]
    fn test_pattern_ignoring() {
        let mut redactor = SecretRedactor::new().unwrap();
        redactor.add_ignored_pattern("github_pat".to_string());

        let content = "token = ghp_1234567890123456789012345678901234567890";
        let matches = redactor.scan_for_secrets(content, "test.txt").unwrap();

        // Should not detect GitHub PAT because it's ignored
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_content_redaction() {
        let redactor = SecretRedactor::new().unwrap();
        let content = "token = ghp_1234567890123456789012345678901234567890\nother_line = safe";

        let result = redactor.redact_content(content, "test.txt").unwrap();

        assert!(result.has_secrets);
        assert_eq!(result.matches.len(), 1);
        assert!(result.content.contains("[REDACTED:github_pat]"));
        assert!(
            !result
                .content
                .contains("ghp_1234567890123456789012345678901234567890")
        );
        assert!(result.content.contains("other_line = safe")); // Safe content preserved
    }

    #[test]
    fn test_safe_context_creation() {
        let redactor = SecretRedactor::new().unwrap();
        let line = "prefix_ghp_1234567890123456789012345678901234567890_suffix";
        let context = redactor.create_safe_context(line, 7, 43); // Position of the token

        assert!(context.contains("prefix_"));
        assert!(context.contains("[REDACTED]"));
        assert!(!context.contains("ghp_1234567890123456789012345678901234567890"));
    }

    #[test]
    fn test_multiple_secrets_in_content() {
        let redactor = SecretRedactor::new().unwrap();
        let content = "github_token = ghp_1234567890123456789012345678901234567890\naws_key = AKIA1234567890123456";

        let matches = redactor.scan_for_secrets(content, "test.txt").unwrap();
        assert_eq!(matches.len(), 2);

        let result = redactor.redact_content(content, "test.txt").unwrap();
        assert!(result.has_secrets);
        assert!(result.content.contains("[REDACTED:github_pat]"));
        assert!(result.content.contains("[REDACTED:aws_access_key]"));
    }

    #[test]
    fn test_line_number_accuracy() {
        let redactor = SecretRedactor::new().unwrap();
        let content = "line 1\nline 2 with ghp_1234567890123456789012345678901234567890\nline 3";

        let matches = redactor.scan_for_secrets(content, "test.txt").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].line_number, 2); // Should be line 2
    }

    #[test]
    fn test_error_creation() {
        let matches = vec![SecretMatch {
            pattern_id: "github_pat".to_string(),
            file_path: "config.yaml".to_string(),
            line_number: 5,
            column_range: (10, 46),
            context: "token = [REDACTED]".to_string(),
        }];

        let error = create_secret_detected_error(&matches);
        match error {
            XCheckerError::SecretDetected { pattern, location } => {
                assert_eq!(pattern, "github_pat");
                assert_eq!(location, "config.yaml:5:10");
            }
            _ => panic!("Expected SecretDetected error"),
        }
    }

    #[test]
    fn test_redact_string() {
        let redactor = SecretRedactor::new().unwrap();

        // Test GitHub PAT redaction
        let text = "token = ghp_1234567890123456789012345678901234567890";
        let redacted = redactor.redact_string(text);
        assert!(redacted.contains("***"));
        assert!(!redacted.contains("ghp_"));

        // Test AWS key redaction
        let text2 = "access_key = AKIA1234567890123456";
        let redacted2 = redactor.redact_string(text2);
        assert!(redacted2.contains("***"));
        assert!(!redacted2.contains("AKIA"));

        // Test no secrets
        let text3 = "This is safe text with no secrets";
        let redacted3 = redactor.redact_string(text3);
        assert_eq!(redacted3, text3);
    }

    #[test]
    fn test_redact_strings() {
        let redactor = SecretRedactor::new().unwrap();

        let strings = vec![
            "token = ghp_1234567890123456789012345678901234567890".to_string(),
            "safe text".to_string(),
            "key = AKIA1234567890123456".to_string(),
        ];

        let redacted = redactor.redact_strings(&strings);
        assert_eq!(redacted.len(), 3);
        assert!(redacted[0].contains("***"));
        assert!(!redacted[0].contains("ghp_"));
        assert_eq!(redacted[1], "safe text");
        assert!(redacted[2].contains("***"));
        assert!(!redacted[2].contains("AKIA"));
    }

    #[test]
    fn test_redact_optional() {
        let redactor = SecretRedactor::new().unwrap();

        // Test Some with secret
        let text = Some("token = ghp_1234567890123456789012345678901234567890".to_string());
        let redacted = redactor.redact_optional(&text);
        assert!(redacted.is_some());
        assert!(redacted.unwrap().contains("***"));

        // Test None
        let none_text: Option<String> = None;
        let redacted_none = redactor.redact_optional(&none_text);
        assert!(redacted_none.is_none());
    }

    #[test]
    fn test_global_redact_user_string() {
        // Test GitHub PAT
        let text = "Failed with token ghp_1234567890123456789012345678901234567890";
        let redacted = redact_user_string(text);
        assert!(redacted.contains("***"));
        assert!(!redacted.contains("ghp_"));

        // Test AWS key
        let text2 = "Error: AKIA1234567890123456 not found";
        let redacted2 = redact_user_string(text2);
        assert!(redacted2.contains("***"));
        assert!(!redacted2.contains("AKIA"));

        // Test Bearer token
        let text3 = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let redacted3 = redact_user_string(text3);
        assert!(redacted3.contains("***"));
        assert!(!redacted3.contains("Bearer eyJ"));
    }

    #[test]
    fn test_global_redact_user_optional() {
        // Test Some with secret
        let text = Some("token = ghp_1234567890123456789012345678901234567890".to_string());
        let redacted = redact_user_optional(&text);
        assert!(redacted.is_some());
        assert!(redacted.unwrap().contains("***"));

        // Test None
        let none_text: Option<String> = None;
        let redacted_none = redact_user_optional(&none_text);
        assert!(redacted_none.is_none());
    }

    #[test]
    fn test_global_redact_user_strings() {
        let strings = vec![
            "error with ghp_1234567890123456789012345678901234567890".to_string(),
            "safe message".to_string(),
            "AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(),
        ];

        let redacted = redact_user_strings(&strings);
        assert_eq!(redacted.len(), 3);
        assert!(redacted[0].contains("***"));
        assert!(!redacted[0].contains("ghp_"));
        assert_eq!(redacted[1], "safe message");
        assert!(redacted[2].contains("***"));
        assert!(!redacted[2].contains("AWS_SECRET_ACCESS_KEY"));
    }

    #[test]
    fn test_redaction_in_error_messages() {
        // Simulate error message with secret
        let error_msg =
            "Authentication failed with token ghp_1234567890123456789012345678901234567890";
        let redacted = redact_user_string(error_msg);

        assert!(redacted.contains("Authentication failed"));
        assert!(redacted.contains("***"));
        assert!(!redacted.contains("ghp_"));
    }

    #[test]
    fn test_redaction_in_context_strings() {
        // Simulate context string with secret
        let context = "Request failed: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9 was invalid";
        let redacted = redact_user_string(context);

        assert!(redacted.contains("Request failed"));
        assert!(redacted.contains("***"));
        assert!(!redacted.contains("Bearer eyJ"));
    }

    #[test]
    fn test_redaction_preserves_safe_content() {
        let safe_text = "This is a normal error message with no secrets at all";
        let redacted = redact_user_string(safe_text);

        // Should be unchanged
        assert_eq!(redacted, safe_text);
    }

    #[test]
    fn test_multiple_secrets_in_one_string() {
        let text = "Error: ghp_1234567890123456789012345678901234567890 and AKIA1234567890123456 both failed";
        let redacted = redact_user_string(text);

        // Both secrets should be redacted
        assert!(!redacted.contains("ghp_"));
        assert!(!redacted.contains("AKIA"));
        assert!(redacted.contains("***"));
        assert!(redacted.contains("Error:"));
        assert!(redacted.contains("both failed"));
    }

    // ===== Empty Input Handling Tests =====

    #[test]
    fn test_empty_content_no_secrets() {
        let redactor = SecretRedactor::new().unwrap();
        let empty_content = "";

        // Empty content should not trigger secret detection
        assert!(!redactor.has_secrets(empty_content, "empty.txt").unwrap());

        // Scanning empty content should return no matches
        let matches = redactor
            .scan_for_secrets(empty_content, "empty.txt")
            .unwrap();
        assert!(matches.is_empty());

        // Redacting empty content should return empty content
        let result = redactor.redact_content(empty_content, "empty.txt").unwrap();
        assert_eq!(result.content, "");
        assert!(!result.has_secrets);
        assert!(result.matches.is_empty());
    }

    #[test]
    fn test_whitespace_only_content_no_secrets() {
        let redactor = SecretRedactor::new().unwrap();
        let whitespace_content = "   \n\t\n   ";

        // Whitespace-only content should not trigger secret detection
        assert!(
            !redactor
                .has_secrets(whitespace_content, "whitespace.txt")
                .unwrap()
        );

        // Redacting whitespace content should preserve it
        let result = redactor
            .redact_content(whitespace_content, "whitespace.txt")
            .unwrap();
        assert_eq!(result.content, whitespace_content);
        assert!(!result.has_secrets);
    }

    #[test]
    fn test_empty_string_redaction() {
        let empty = "";
        let redacted = redact_user_string(empty);
        assert_eq!(redacted, "");
    }

    #[test]
    fn test_empty_optional_redaction() {
        let redactor = SecretRedactor::new().unwrap();
        let none_value: Option<String> = None;
        let result = redactor.redact_optional(&none_value);
        assert_eq!(result, None);
    }

    #[test]
    fn test_empty_strings_vec_redaction() {
        let redactor = SecretRedactor::new().unwrap();
        let empty_vec: Vec<String> = vec![];
        let result = redactor.redact_strings(&empty_vec);
        assert!(result.is_empty());
    }

    #[test]
    fn test_vec_with_empty_strings_redaction() {
        let redactor = SecretRedactor::new().unwrap();
        let strings = vec![String::new(), "   ".to_string(), "normal text".to_string()];
        let result = redactor.redact_strings(&strings);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "");
        assert_eq!(result[1], "   ");
        assert_eq!(result[2], "normal text");
    }

    #[test]
    fn test_global_redact_empty_string() {
        let empty = "";
        let redacted = redact_user_string(empty);
        assert_eq!(redacted, "");
    }

    #[test]
    fn test_global_redact_empty_optional() {
        let none_value: Option<String> = None;
        let result = redact_user_optional(&none_value);
        assert_eq!(result, None);
    }

    #[test]
    fn test_global_redact_empty_vec() {
        let empty_vec: Vec<String> = vec![];
        let result = redact_user_strings(&empty_vec);
        assert!(result.is_empty());
    }

    #[test]
    fn test_scan_empty_file_path() {
        let redactor = SecretRedactor::new().unwrap();
        let content = "Some content with ghp_1234567890123456789012345678901234567890";

        // Empty file path should still work
        let matches = redactor.scan_for_secrets(content, "").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].file_path, "");
    }

    // ===== New Pattern Tests (Task 23.1) =====

    #[test]
    fn test_gcp_api_key_detection() {
        let redactor = SecretRedactor::new().unwrap();
        let content = "api_key = AIzaSyDaGmWKa4JsXZ-HjGw7ISLn_3namBGewQe";

        let matches = redactor.scan_for_secrets(content, "test.txt").unwrap();
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.pattern_id == "gcp_api_key"));
    }

    #[test]
    fn test_gcp_service_account_key_detection() {
        let redactor = SecretRedactor::new().unwrap();
        let content = "-----BEGIN PRIVATE KEY-----\nMIIEvgIBADANBg...";

        let matches = redactor.scan_for_secrets(content, "test.txt").unwrap();
        assert!(!matches.is_empty());
        // Should match one of the private key patterns
        assert!(matches.iter().any(|m| m.pattern_id.contains("private_key")));
    }

    #[test]
    fn test_azure_connection_string_detection() {
        let redactor = SecretRedactor::new().unwrap();
        // 88-char base64 key
        let key = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+/abcdefghijklmnopqrstuv==";
        let content = format!(
            "DefaultEndpointsProtocol=https;AccountName=myaccount;AccountKey={}",
            key
        );

        let matches = redactor.scan_for_secrets(&content, "test.txt").unwrap();
        assert!(!matches.is_empty());
        assert!(
            matches
                .iter()
                .any(|m| m.pattern_id == "azure_connection_string")
        );
    }

    #[test]
    fn test_azure_sas_token_detection() {
        let redactor = SecretRedactor::new().unwrap();
        let content = "https://myaccount.blob.core.windows.net/container?sv=2020-08-04&sig=abcdefghijklmnopqrstuvwxyz1234567890ABCDEF%3D";

        let matches = redactor.scan_for_secrets(content, "test.txt").unwrap();
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.pattern_id == "azure_sas_token"));
    }

    #[test]
    fn test_jwt_token_detection() {
        let redactor = SecretRedactor::new().unwrap();
        let content = "token = eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";

        let matches = redactor.scan_for_secrets(content, "test.txt").unwrap();
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.pattern_id == "jwt_token"));
    }

    #[test]
    fn test_postgres_url_detection() {
        let redactor = SecretRedactor::new().unwrap();
        let content = "DATABASE_URL=postgres://user:password123@localhost:5432/mydb";

        let matches = redactor.scan_for_secrets(content, "test.txt").unwrap();
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.pattern_id == "postgres_url"));
    }

    #[test]
    fn test_mysql_url_detection() {
        let redactor = SecretRedactor::new().unwrap();
        let content = "DATABASE_URL=mysql://admin:secretpass@db.example.com:3306/production";

        let matches = redactor.scan_for_secrets(content, "test.txt").unwrap();
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.pattern_id == "mysql_url"));
    }

    #[test]
    fn test_mongodb_url_detection() {
        let redactor = SecretRedactor::new().unwrap();
        let content = "MONGO_URI=mongodb+srv://user:pass123@cluster0.mongodb.net/mydb";

        let matches = redactor.scan_for_secrets(content, "test.txt").unwrap();
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.pattern_id == "mongodb_url"));
    }

    #[test]
    fn test_redis_url_detection() {
        let redactor = SecretRedactor::new().unwrap();
        let content = "REDIS_URL=redis://:mypassword@redis.example.com:6379";

        let matches = redactor.scan_for_secrets(content, "test.txt").unwrap();
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.pattern_id == "redis_url"));
    }

    #[test]
    fn test_ssh_private_key_detection() {
        let redactor = SecretRedactor::new().unwrap();
        let content = "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...";

        let matches = redactor.scan_for_secrets(content, "test.txt").unwrap();
        assert!(!matches.is_empty());
        // Should match RSA private key pattern
        assert!(
            matches
                .iter()
                .any(|m| m.pattern_id == "rsa_private_key" || m.pattern_id == "ssh_private_key")
        );
    }

    #[test]
    fn test_openssh_private_key_detection() {
        let redactor = SecretRedactor::new().unwrap();
        let content = "-----BEGIN OPENSSH PRIVATE KEY-----\nb3BlbnNzaC1rZXktdjEAAAAA...";

        let matches = redactor.scan_for_secrets(content, "test.txt").unwrap();
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.pattern_id == "openssh_private_key" || m.pattern_id == "ssh_private_key"));
    }

    #[test]
    fn test_stripe_key_detection() {
        let redactor = SecretRedactor::new().unwrap();
        let content = ["STRIPE_SECRET_KEY=sk_live_", "1234567890abcdefghijklmnop"].join("");

        let matches = redactor.scan_for_secrets(&content, "test.txt").unwrap();
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.pattern_id == "stripe_key"));
    }

    #[test]
    fn test_sendgrid_key_detection() {
        let redactor = SecretRedactor::new().unwrap();
        // SendGrid keys have format: SG.<22 chars>.<43 chars>
        let content = [
            "SENDGRID_API_KEY=SG.",
            "1234567890abcdefghijkl",
            ".abcdefghijklmnopqrstuvwxyz1234567890ABCDEFG",
        ]
        .join("");

        let matches = redactor.scan_for_secrets(&content, "test.txt").unwrap();
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.pattern_id == "sendgrid_key"));
    }

    #[test]
    fn test_gitlab_token_detection() {
        let redactor = SecretRedactor::new().unwrap();
        let content = "GITLAB_TOKEN=glpat-1234567890abcdefghij";

        let matches = redactor.scan_for_secrets(content, "test.txt").unwrap();
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.pattern_id == "gitlab_token"));
    }

    #[test]
    fn test_github_oauth_token_detection() {
        let redactor = SecretRedactor::new().unwrap();
        let content = "token = gho_1234567890123456789012345678901234567890";

        let matches = redactor.scan_for_secrets(content, "test.txt").unwrap();
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.pattern_id == "github_oauth"));
    }

    #[test]
    fn test_npm_token_detection() {
        let redactor = SecretRedactor::new().unwrap();
        let content = "NPM_TOKEN=npm_1234567890123456789012345678901234567890";

        let matches = redactor.scan_for_secrets(content, "test.txt").unwrap();
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.pattern_id == "npm_token"));
    }

    #[test]
    fn test_basic_auth_detection() {
        let redactor = SecretRedactor::new().unwrap();
        let content = "Authorization: Basic dXNlcm5hbWU6cGFzc3dvcmQxMjM0NTY3ODkw";

        let matches = redactor.scan_for_secrets(content, "test.txt").unwrap();
        assert!(!matches.is_empty());
        assert!(
            matches
                .iter()
                .any(|m| m.pattern_id == "authorization_basic")
        );
    }

    #[test]
    fn test_all_new_pattern_categories_exist() {
        let redactor = SecretRedactor::new().unwrap();
        let pattern_ids = redactor.get_pattern_ids();

        // AWS patterns
        assert!(pattern_ids.contains(&"aws_access_key".to_string()));
        assert!(pattern_ids.contains(&"aws_secret_key".to_string()));
        assert!(pattern_ids.contains(&"aws_session_token".to_string()));

        // GCP patterns
        assert!(pattern_ids.contains(&"gcp_api_key".to_string()));
        assert!(pattern_ids.contains(&"gcp_service_account_key".to_string()));

        // Azure patterns
        assert!(pattern_ids.contains(&"azure_storage_key".to_string()));
        assert!(pattern_ids.contains(&"azure_connection_string".to_string()));
        assert!(pattern_ids.contains(&"azure_sas_token".to_string()));

        // Generic API tokens
        assert!(pattern_ids.contains(&"bearer_token".to_string()));
        assert!(pattern_ids.contains(&"api_key_header".to_string()));
        assert!(pattern_ids.contains(&"authorization_basic".to_string()));
        assert!(pattern_ids.contains(&"oauth_token".to_string()));
        assert!(pattern_ids.contains(&"jwt_token".to_string()));

        // Database URLs
        assert!(pattern_ids.contains(&"postgres_url".to_string()));
        assert!(pattern_ids.contains(&"mysql_url".to_string()));
        assert!(pattern_ids.contains(&"sqlserver_url".to_string()));
        assert!(pattern_ids.contains(&"mongodb_url".to_string()));
        assert!(pattern_ids.contains(&"redis_url".to_string()));

        // SSH/PEM keys
        assert!(pattern_ids.contains(&"ssh_private_key".to_string()));
        assert!(pattern_ids.contains(&"rsa_private_key".to_string()));
        assert!(pattern_ids.contains(&"ec_private_key".to_string()));
        assert!(pattern_ids.contains(&"pem_private_key".to_string()));
        assert!(pattern_ids.contains(&"openssh_private_key".to_string()));

        // Platform-specific tokens
        assert!(pattern_ids.contains(&"github_pat".to_string()));
        assert!(pattern_ids.contains(&"github_oauth".to_string()));
        assert!(pattern_ids.contains(&"github_app_token".to_string()));
        assert!(pattern_ids.contains(&"gitlab_token".to_string()));
        assert!(pattern_ids.contains(&"slack_token".to_string()));
        assert!(pattern_ids.contains(&"stripe_key".to_string()));
        assert!(pattern_ids.contains(&"twilio_key".to_string()));
        assert!(pattern_ids.contains(&"sendgrid_key".to_string()));
        assert!(pattern_ids.contains(&"npm_token".to_string()));
        assert!(pattern_ids.contains(&"pypi_token".to_string()));
    }

    // ===== from_config Tests (Task 23.2) =====

    #[test]
    fn test_from_config_with_default_security() {
        let config = Config::builder().build().unwrap();
        let redactor = SecretRedactor::from_config(&config).unwrap();

        // Should have all default patterns
        let pattern_ids = redactor.get_pattern_ids();
        assert!(pattern_ids.contains(&"github_pat".to_string()));
        assert!(pattern_ids.contains(&"aws_access_key".to_string()));

        // Should have no extra patterns
        assert!(
            !pattern_ids
                .iter()
                .any(|id| id.starts_with("extra_pattern_"))
        );

        // Should have no ignored patterns
        assert!(redactor.get_ignored_patterns().is_empty());
    }

    #[test]
    fn test_from_config_with_extra_patterns() {
        let config = Config::builder()
            .extra_secret_patterns(vec![
                "CUSTOM_[A-Z0-9]{32}".to_string(),
                "MY_SECRET_[A-Za-z0-9]{20}".to_string(),
            ])
            .build()
            .unwrap();

        let redactor = SecretRedactor::from_config(&config).unwrap();

        // Should have extra patterns
        let pattern_ids = redactor.get_pattern_ids();
        assert!(pattern_ids.contains(&"extra_pattern_0".to_string()));
        assert!(pattern_ids.contains(&"extra_pattern_1".to_string()));

        // Extra patterns should detect custom secrets
        let content = "key = CUSTOM_12345678901234567890123456789012";
        let matches = redactor.scan_for_secrets(content, "test.txt").unwrap();
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.pattern_id == "extra_pattern_0"));
    }

    #[test]
    fn test_from_config_with_ignore_patterns() {
        let config = Config::builder()
            .ignore_secret_patterns(vec!["github_pat".to_string()])
            .build()
            .unwrap();

        let redactor = SecretRedactor::from_config(&config).unwrap();

        // Should have ignored pattern
        assert!(
            redactor
                .get_ignored_patterns()
                .contains(&"github_pat".to_string())
        );

        // GitHub PAT should not be detected
        let content = "token = ghp_1234567890123456789012345678901234567890";
        let matches = redactor.scan_for_secrets(content, "test.txt").unwrap();
        assert!(matches.is_empty());
    }

    #[test]
    fn test_from_config_with_both_extra_and_ignore() {
        let config = Config::builder()
            .extra_secret_patterns(vec!["CUSTOM_[A-Z0-9]{32}".to_string()])
            .ignore_secret_patterns(vec!["github_pat".to_string()])
            .build()
            .unwrap();

        let redactor = SecretRedactor::from_config(&config).unwrap();

        // Should have extra pattern
        let pattern_ids = redactor.get_pattern_ids();
        assert!(pattern_ids.contains(&"extra_pattern_0".to_string()));

        // Should have ignored pattern
        assert!(
            redactor
                .get_ignored_patterns()
                .contains(&"github_pat".to_string())
        );

        // Custom secret should be detected
        let content1 = "key = CUSTOM_12345678901234567890123456789012";
        let matches1 = redactor.scan_for_secrets(content1, "test.txt").unwrap();
        assert!(!matches1.is_empty());

        // GitHub PAT should not be detected
        let content2 = "token = ghp_1234567890123456789012345678901234567890";
        let matches2 = redactor.scan_for_secrets(content2, "test.txt").unwrap();
        assert!(matches2.is_empty());
    }

    #[test]
    fn test_from_config_with_invalid_extra_pattern() {
        let config = Config::builder()
            .extra_secret_patterns(vec!["[invalid regex".to_string()])
            .build()
            .unwrap();

        // Should fail to create redactor with invalid regex
        let result = SecretRedactor::from_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_config_add_extra_secret_pattern_method() {
        let config = Config::builder()
            .add_extra_secret_pattern("SINGLE_[A-Z]{10}")
            .add_extra_secret_pattern("ANOTHER_[0-9]{8}")
            .build()
            .unwrap();

        let redactor = SecretRedactor::from_config(&config).unwrap();

        // Should have both extra patterns
        let pattern_ids = redactor.get_pattern_ids();
        assert!(pattern_ids.contains(&"extra_pattern_0".to_string()));
        assert!(pattern_ids.contains(&"extra_pattern_1".to_string()));
    }

    #[test]
    fn test_from_config_add_ignore_secret_pattern_method() {
        let config = Config::builder()
            .add_ignore_secret_pattern("github_pat")
            .add_ignore_secret_pattern("aws_access_key")
            .build()
            .unwrap();

        let redactor = SecretRedactor::from_config(&config).unwrap();

        // Should have both ignored patterns
        let ignored = redactor.get_ignored_patterns();
        assert!(ignored.contains(&"github_pat".to_string()));
        assert!(ignored.contains(&"aws_access_key".to_string()));
    }
}
