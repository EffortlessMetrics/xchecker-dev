use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use xchecker_redaction::SecretRedactor;

pub fn redact_bench(c: &mut Criterion) {
    let redactor = SecretRedactor::new().unwrap();
    let token = "ghp_abcdefghijklmnopqrstuvwxyz0123456789";
    let mut content = String::new();
    for i in 0..1000 {
        if i % 10 == 0 {
            content.push_str(&format!("Line {} has token: {}\n", i, token));
        } else {
            content.push_str(&format!("Line {} is safe\n", i));
        }
    }

    c.bench_function("SecretRedactor::redact_content", |b| {
        b.iter(|| {
            black_box(redactor.redact_content(&content, "test.txt").unwrap());
        })
    });
}

criterion_group!(benches, redact_bench);
criterion_main!(benches);
