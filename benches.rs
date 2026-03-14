use xchecker_error_redaction::redact_error_message;
use std::time::Instant;

fn main() {
    let start = Instant::now();
    for _ in 0..10000 {
        redact_error_message("Authentication failed with key sk-1234567890abcdefghijklmnopqrstuvwxyz");
    }
    println!("Elapsed: {:?}", start.elapsed());
}
