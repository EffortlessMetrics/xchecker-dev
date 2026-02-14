use xchecker_redaction::SecretRedactor;

#[test]
fn test_crlf_redaction_correctness() {
    let mut redactor = SecretRedactor::new().unwrap();
    redactor.add_extra_pattern("test_secret".to_string(), "SECRET").unwrap();

    // "line1\r\nline2 SECRET\r\nline3"
    let content = "line1\r\nline2 SECRET\r\nline3";

    // We expect normalization to \n and correct replacement.
    // The loop over lines() + push('\n') will result in a trailing newline.
    let expected = "line1\nline2 [REDACTED:test_secret]\nline3\n";

    let result = redactor.redact_content(content, "test.txt").unwrap();

    println!("Original content indices:");
    for (i, c) in content.char_indices() {
        println!("{}: {:?}", i, c);
    }

    println!("Redacted content: {:?}", result.content);

    assert_eq!(result.content, expected, "Content should be correctly redacted and normalized to LF");
}
