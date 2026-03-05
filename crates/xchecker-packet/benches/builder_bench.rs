use camino::Utf8PathBuf;
use criterion::{Criterion, criterion_group, criterion_main};
use std::fs;
use std::hint::black_box;
use tempfile::TempDir;
use xchecker_packet::PacketBuilder;

pub fn build_packet_bench(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let base_path = Utf8PathBuf::try_from(temp_dir.path().to_path_buf()).unwrap();
    let context_dir = base_path.join("context");

    fs::create_dir_all(&context_dir).unwrap();

    // Create 100 dummy files
    for i in 0..100 {
        fs::write(
            base_path.join(format!("file_{}.txt", i)),
            format!(
                "This is test file number {}\nIt has some content to process.",
                i
            ),
        )
        .unwrap();
    }

    c.bench_function("PacketBuilder::build_packet(100 files)", |b| {
        // Create builder once outside loop to bench just `build_packet`
        let mut builder = PacketBuilder::new().unwrap();
        b.iter(|| {
            black_box(
                builder
                    .build_packet(&base_path, "requirements", &context_dir, None)
                    .unwrap(),
            );
        })
    });
}

criterion_group!(benches, build_packet_bench);
criterion_main!(benches);
