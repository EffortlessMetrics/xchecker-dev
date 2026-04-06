use std::time::Instant;
use std::borrow::Cow;
use regex::Regex;

fn redact_error_message_for_logging(message: &str) -> String {
    let mut redacted = message.to_string();

    let api_key_regex =
        Regex::new(r"(?:sk-|pk_|api_key|secret|Bearer )[a-zA-Z0-9_-]{20,}").unwrap();
    redacted = api_key_regex
        .replace_all(&redacted, "[REDACTED_KEY]")
        .to_string();

    let long_key_regex = Regex::new(r"[a-zA-Z0-9_-]{32,}").unwrap();
    let mut replacements = Vec::new();

    for mat in long_key_regex.find_iter(&redacted) {
        let start = mat.start();
        let end = mat.end();

        let boundary_before = if start == 0 {
            true
        } else {
            let prev_char = redacted[..start].chars().last().unwrap();
            !prev_char.is_alphanumeric() && prev_char != '_' && prev_char != '-'
        };

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

    for (start, end) in replacements.into_iter().rev() {
        redacted.replace_range(start..end, "[REDACTED_KEY]");
    }

    let url_with_creds_regex = Regex::new(r"https?://[a-zA-Z0-9_]+:[^:@\s]+@").unwrap();
    redacted = url_with_creds_regex
        .replace_all(&redacted, "[REDACTED]@")
        .to_string();

    if redacted.contains("password") || redacted.contains("token") {
        let password_regex = Regex::new(r"(?i)(password|pass|token)").unwrap();
        redacted = password_regex.replace_all(&redacted, "***").to_string();
    }

    redacted = redacted.replace(r"C:\\", r"\");
    redacted = redacted.replace(r"D:\\", r"\");
    redacted
}

fn main() {
    let message = "Authentication failed with key sk-1234567890abcdefghijklmnopqrstuvwxyz and https://user:pass@api.com/endpoint and password is secret";

    // Warmup
    for _ in 0..10 {
        redact_error_message_for_logging(message);
    }

    let start = Instant::now();
    for _ in 0..10000 {
        redact_error_message_for_logging(message);
    }
    let duration = start.elapsed();

    println!("Duration: {:?}", duration);
}
