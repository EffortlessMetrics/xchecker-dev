use xchecker_redaction::SecretRedactor;

fn main() {
    let redactor = SecretRedactor::new().unwrap();
    let token = "AIzaSy_some_random_base64_like_string_here_33c";
    let content = format!("GEMINI_API_KEY={}", token);

    let matches = redactor.scan_for_secrets(&content, "test.txt").unwrap();
    println!("Matches: {:?}", matches);
}
