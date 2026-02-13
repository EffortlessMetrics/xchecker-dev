use std::time::Instant;
use xchecker_redaction::SecretRedactor;

#[test]
fn test_crlf_content_preservation() {
    let mut redactor = SecretRedactor::new().unwrap();
    redactor
        .add_extra_pattern("custom_secret".to_string(), "SECRET")
        .unwrap();

    // "safe\r\n" (len 6)
    // "PRE_SECRET_POST\r\n" (len 4+6+5+2 = 17)
    let content = "safe\r\nPRE_SECRET_POST\r\n";

    // This test ensures that when CRLF line endings are used:
    // 1. Line offsets are calculated correctly (fixing previous bug)
    // 2. Surrounding content is preserved
    // 3. Secrets are redacted

    let result = redactor.redact_content(content, "test.txt").unwrap();

    println!("Original: {:?}", content);
    println!("Redacted: {:?}", result.content);

    // Check preservation of surrounding text
    assert!(
        result.content.contains("PRE_"),
        "Prefix corrupted: {:?}",
        result.content
    );
    assert!(
        result.content.contains("_POST"),
        "Suffix corrupted: {:?}",
        result.content
    );

    // Check redaction
    assert!(
        !result.content.contains("SECRET"),
        "Secret should be redacted"
    );
    assert!(
        result.content.contains("[REDACTED:custom_secret]"),
        "Redaction marker missing"
    );
}

#[test]
fn test_large_file_redaction_performance() {
    let mut redactor = SecretRedactor::new().unwrap();
    redactor
        .add_extra_pattern("custom_secret".to_string(), "SECRET")
        .unwrap();

    // Create a large content string (simulating a large file)
    // 5000 lines is enough to show the O(N) vs O(M*N) difference
    // Previous O(M*N) took ~2.7s for 5000 lines.
    // O(N) takes ~40ms.
    let mut content = String::with_capacity(5000 * 50);
    for i in 0..5000 {
        content.push_str(&format!("line {} with SECRET in it\n", i));
    }

    let start = Instant::now();
    let result = redactor.redact_content(&content, "perf.txt").unwrap();
    let duration = start.elapsed();

    println!("Redaction of 5000 lines with secrets took: {:?}", duration);

    assert!(result.has_secrets);

    // Performance assertion (loose, to avoid flaky CI)
    // It should be well under 500ms even on slow machines if optimized.
    assert!(
        duration.as_millis() < 500,
        "Redaction too slow: {:?} (expected < 500ms)",
        duration
    );
}
