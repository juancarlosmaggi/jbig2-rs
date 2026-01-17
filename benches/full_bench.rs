use criterion::{Criterion, black_box, criterion_group, criterion_main};
use jbig2_rs::image::Jbig2Document;
use std::fs;
use std::path::Path;

/// Benchmark end-to-end decoding for representative test files.
fn bench_full_file_decoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_file_decoding");

    // Benchmark a symbol dictionary-heavy file if present.
    let symbol_dict_path = Path::new("tests/resources/symbol_dictionary.jb2");
    if symbol_dict_path.exists() {
        let data = fs::read(symbol_dict_path).expect("Failed to read symbol_dictionary.jb2");
        group.bench_function("symbol_dictionary", |b| {
            b.iter(|| {
                let _ = Jbig2Document::parse(black_box(&data));
            })
        });
    } else {
        eprintln!("Warning: tests/resources/symbol_dictionary.jb2 not found");
    }

    // Benchmark a text region-heavy file if present.
    let text_region_path = Path::new("tests/resources/text_region.jb2");
    if text_region_path.exists() {
        let data = fs::read(text_region_path).expect("Failed to read text_region.jb2");
        group.bench_function("text_region", |b| {
            b.iter(|| {
                let _ = Jbig2Document::parse(black_box(&data));
            })
        });
    } else {
        eprintln!("Warning: tests/resources/text_region.jb2 not found");
    }

    group.finish();
}

criterion_group!(benches, bench_full_file_decoding);
criterion_main!(benches);
