use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn dummy(c: &mut Criterion) {
    c.bench_function("dummy", |b| b.iter(|| black_box(1)));
}

criterion_group!(benches, dummy);
criterion_main!(benches);
