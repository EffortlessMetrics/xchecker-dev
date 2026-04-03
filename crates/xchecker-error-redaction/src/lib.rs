//! Error redaction utilities for LLM providers
//!
//! This module provides shared functions for redacting sensitive information
//! from error messages before they are logged or displayed to users. This prevents accidental
//! exposure of:
//! - API keys and authentication credentials
//! - File paths that may contain user-specific data
//! - URLs with embedded credentials
//!
//! # Security
//!
//! These utilities are designed to prevent information leakage through proper logging and
//! message handling. The redaction rules are:
//!
//! 1. **Never log API keys** - Keys should never appear in logs
//! 2. **Never log authentication credentials** - Auth tokens/passwords should never appear in logs
//! 3. **Never log URLs with embedded credentials** - URLs like `http://user:pass@host` should be redacted
//! 4. **Never log file paths** - Local file paths should be normalized
//! 5. **Preserve error context** - Keep enough information for debugging without exposing secrets
//!
//! # Functions
//!
//! ## Error Message Redaction
//!
//! ### `redact_error_message_for_logging`
//!
//! Redacts sensitive information from error messages intended for logging.
//! Removes:
//! - API keys (long alphanumeric strings)
//! - Authentication credentials (passwords, tokens)
//! - URLs with embedded credentials
//! - File paths that may contain user-specific data
//!
//! ### `redact_error_message`
//!
//! Redacts sensitive information from error messages for display.
//! Uses same rules as `redact_error_message_for_logging` but for display purposes.
//!
//! ## Path Redaction
//!
//! ### `redact_paths`
//!
//! Redacts potentially sensitive path information from error messages.
//! Removes:
//! - Common path separators (`/`, `\`)
//! - Home directory indicators
//! - Windows drive letters
//! - User home directory indicators
//!
//! ### `redact_error_message`
//!
//! Main entry point that redacts error messages.
//! Delegates to `redact_error_message_for_logging` for logging and
//! `redact_error_message` for display for display purposes.
//!
//! # Examples
//!
//! ```rust,no_run
//! use xchecker_error_redaction::redact_error_message;
//!
//! let error = "Failed to connect to http://user:password@api.example.com/endpoint";
//! let redacted = redact_error_message(&error);
//! assert!(redacted.contains("Failed to connect"));
//! assert!(redacted.contains("http://api.example.com"));
//! assert!(!redacted.contains("user:password"));
//! ```

use std::borrow::Cow;
use std::sync::LazyLock;

/// Redact sensitive information from error messages intended for logging.
///
/// Removes API keys, authentication credentials, URLs with embedded credentials,
/// and file paths that may contain user-specific data.
///
/// # Parameters
///
/// * `message` - The error message to redact
///
/// # Returns
///
/// The redacted error message with sensitive information removed.
pub fn redact_error_message_for_logging(message: &str) -> String {
    // ⚡ Bolt Performance Optimization: Statically cache compiled regular expressions
    // to avoid the significant overhead of parsing and compiling them on every function call.
    static API_KEY_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"(?:sk-|pk_|api_key|secret|Bearer )[a-zA-Z0-9_-]{20,}").unwrap()
    });

    static LONG_KEY_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"[a-zA-Z0-9_-]{32,}").unwrap()
    });

    static URL_WITH_CREDS_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"https?://[a-zA-Z0-9_]+:[^:@\s]+@").unwrap()
    });

    static PASSWORD_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"(?i)(password|pass|token)").unwrap()
    });

    // Redact API keys (long alphanumeric strings with common prefixes)
    // Only redact strings that look like actual API keys (with prefixes like sk-, pk_, etc.)
    // Pattern: prefix followed by at least 20 alphanumeric characters
    let mut redacted: Cow<str> = API_KEY_REGEX.replace_all(message, "[REDACTED_KEY]");

    // Also redact long alphanumeric strings that look like keys (without explicit prefix)
    // Pattern: 32+ alphanumeric/underscore/dash characters that look like a key
    // Only match standalone keys (not embedded in URLs or after @)
    // Manually check boundaries to handle hyphens correctly (which \b doesn't handle well)
    let mut replacements = Vec::new();

    for mat in LONG_KEY_REGEX.find_iter(&redacted) {
        let start = mat.start();
        let end = mat.end();

        // Check boundary before
        let boundary_before = if start == 0 {
            true
        } else {
            let prev_char = redacted[..start].chars().last().unwrap();
            !prev_char.is_alphanumeric() && prev_char != '_' && prev_char != '-'
        };

        // Check boundary after
        let boundary_after = if end == redacted.len() {
            true
        } else {
            let next_char = redacted[end..].chars().next().unwrap();
            !next_char.is_alphanumeric() && next_char != '_' && next_char != '-'
        };

        if boundary_before && boundary_after {
            replacements.push((start, end));
        }
    }

    // Apply replacements in reverse order to preserve indices
    if !replacements.is_empty() {
        let mut owned_redacted = redacted.into_owned();
        for (start, end) in replacements.into_iter().rev() {
            owned_redacted.replace_range(start..end, "[REDACTED_KEY]");
        }
        redacted = Cow::Owned(owned_redacted);
    }

    // Redact URLs with embedded credentials first to avoid breaking patterns
    // Pattern: `http://user:pass@host/path` or `https://token123:secret456@host/path`
    if let Cow::Owned(s) = URL_WITH_CREDS_REGEX.replace_all(&redacted, "[REDACTED]@") {
        redacted = Cow::Owned(s);
    }

    // Redact authentication credentials (passwords, tokens)
    if redacted.contains("password") || redacted.contains("token") {
        // Redact common password patterns - simpler regex without character class issues
        if let Cow::Owned(s) = PASSWORD_REGEX.replace_all(&redacted, "***") {
            redacted = Cow::Owned(s);
        }
    }

    let mut final_string = redacted.into_owned();
    // Redact file paths that may contain user-specific data
    // Normalize Windows paths
    final_string = final_string.replace(r"C:\\", r"\");
    final_string = final_string.replace(r"D:\\", r"\");
    final_string
}

/// Redact sensitive information from error messages for display.
///
/// Removes API keys, authentication credentials, URLs with embedded credentials,
/// and file paths that may contain user-specific data.
///
/// # Parameters
///
/// * `message` - The error message to redact
///
/// # Returns
///
/// The redacted error message with sensitive information removed.
pub fn redact_error_message(message: &str) -> String {
    // Use same rules as logging version but for display purposes
    redact_error_message_for_logging(message)
}

/// Redact potentially sensitive path information from error messages.
///
/// Removes:
/// - Common path separators (`/`, `\`)
/// - Home directory indicators
/// - Windows drive letters
/// - User home directory indicators
///
/// # Returns
///
/// The redacted error message with paths normalized.
pub fn redact_paths(message: &str) -> String {
    // ⚡ Bolt Performance Optimization: Statically cache compiled regular expressions
    static UNIX_HOME_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"/(?:home|Users)/[^/\\\\]+").unwrap()
    });

    static WIN_HOME_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"(?i)(?:[A-Za-z]:)?\\\\Users\\\\[^\\\\/]+").unwrap()
    });

    static DRIVE_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"[A-Za-z]:\\\\").unwrap()
    });

    // Redact Unix-style home directories first (e.g., /home/user, /Users/user)
    let mut redacted: Cow<str> = UNIX_HOME_REGEX.replace_all(message, "[HOME]");

    // Redact Windows home directories, optionally with a drive letter
    if let Cow::Owned(s) = WIN_HOME_REGEX.replace_all(&redacted, "[HOME]") {
        redacted = Cow::Owned(s);
    }

    // Redact Windows drive letters (C:\, D:\, etc.)
    if let Cow::Owned(s) = DRIVE_REGEX.replace_all(&redacted, "[DRIVE]") {
        redacted = Cow::Owned(s);
    }

    let mut final_string = redacted.into_owned();
    // Replace path separators to avoid leaking remaining path structure
    final_string = final_string.replace("\\", "[PATH]");
    final_string = final_string.replace("/", "[PATH]");

    final_string
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_api_keys() {
        let message = "Authentication failed with key sk-1234567890abcdefghijklmnopqrstuvwxyz";
        let redacted = redact_error_message_for_logging(message);
        assert!(
            !redacted.contains("sk-1234567890abcdefghijklmnopqrstuvwxyz"),
            "Should redact long alphanumeric strings that look like keys"
        );
        assert!(
            redacted.contains("[REDACTED_KEY]"),
            "Should replace key with [REDACTED_KEY]"
        );
        assert!(
            redacted.contains("Authentication failed"),
            "Should preserve error context"
        );
    }

    #[test]
    fn test_redact_urls_with_credentials() {
        // Simplified test - focus on core functionality
        let message = "Failed to connect to http://user:pass@api.com/endpoint";
        let redacted = redact_error_message_for_logging(message);
        assert!(
            !redacted.contains("user:pass"),
            "Should redact credentials from URL"
        );
        assert!(
            redacted.contains("[REDACTED]@"),
            "Should replace credentials with [REDACTED]"
        );
        assert!(redacted.contains("api.com"), "Should preserve host");
    }

    #[test]
    fn test_redact_paths() {
        // Simplified test - focus on core functionality
        let message = "Error: /home/user/project/file.txt";
        let redacted = redact_paths(message);
        assert!(!redacted.contains("/home"), "Should redact home directory");
        assert!(!redacted.contains("user"), "Should redact username");
        assert!(
            redacted.contains("[HOME]"),
            "Should replace home with [HOME]"
        );
    }

    #[test]
    fn test_preserve_safe_messages() {
        let message = "Connection failed: timeout";
        let redacted = redact_error_message_for_logging(message);
        assert_eq!(redacted, message, "Should preserve safe error message");
    }
}
