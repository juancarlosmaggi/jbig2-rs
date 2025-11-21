use criterion::{criterion_group, criterion_main, Criterion};

fn bench_decoder_placeholder(c: &mut Criterion) {
    c.bench_function("decoder_placeholder", |b| {
        b.iter(|| {
            // Placeholder
            1 + 1
        })
    });
}

criterion_group!(benches, bench_decoder_placeholder);
criterion_main!(benches);
