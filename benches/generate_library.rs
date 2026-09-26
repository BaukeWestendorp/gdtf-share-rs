use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn criterion_benchmark(c: &mut Criterion) {
    let entries_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("benches").join("entries.json");
    let entries_file = std::fs::File::open(entries_path).expect("Failed to open entries file");
    let entries: Vec<gdtf_share::Entry> =
        serde_json::from_reader(entries_file).expect("Failed to deserialize entries");

    c.bench_function("Generate Library", |b| {
        b.iter_batched(
            || entries.clone(),
            |entries_clone| gdtf_share::Library::new(black_box(entries_clone)),
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
