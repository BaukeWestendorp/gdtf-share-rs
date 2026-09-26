use criterion::{Criterion, criterion_group, criterion_main};
use gdtf_share::{Library, Rating, Uploader};
use std::hint::black_box;

fn benchmark_queries(c: &mut Criterion) {
    let entries_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("benches").join("entries.json");
    let entries_file = std::fs::File::open(entries_path).expect("Failed to open entries file");
    let entries: Vec<gdtf_share::Entry> =
        serde_json::from_reader(entries_file).expect("Failed to deserialize entries");

    let library = Library::new(entries);

    c.bench_function("Query Exact Manufacturer", |b| {
        b.iter(|| library.query().manufacturer(black_box("ROBE lighting")).execute().count());
    });

    c.bench_function("Query Rating Range (10..=50)", |b| {
        b.iter(|| {
            library
                .query()
                .rating_range(black_box(Rating::Value(10))..=black_box(Rating::Value(50)))
                .execute()
                .count()
        });
    });

    c.bench_function("Query Manufacturer and Uploader (Latest Only)", |b| {
        b.iter(|| {
            library
                .query()
                .manufacturer(black_box("Martin Professional"))
                .uploader(black_box(&Uploader::Manufacturer))
                .latest_only(black_box(true))
                .execute()
                .count()
        });
    });

    c.bench_function("Search Latest Text ('15c ii')", |b| {
        b.iter(|| library.search_latest(black_box("15c ii")).count());
    });

    c.bench_function("Search Latest Text ('Showtec')", |b| {
        b.iter(|| library.search_latest(black_box("Showtec")).count());
    });
}

criterion_group!(benches, benchmark_queries);
criterion_main!(benches);
