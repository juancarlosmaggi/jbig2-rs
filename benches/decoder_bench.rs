use criterion::{Criterion, black_box, criterion_group, criterion_main};
use jbig2_rs::arithmetic::ArithmeticDecoder;

/// Benchmark arithmetic decoder bit reads with a synthetic buffer.
fn bench_arithmetic_decoder(c: &mut Criterion) {
    // Use a uniform buffer to focus on read_bit overhead.
    let data = vec![0xAA; 1024];

    c.bench_function("arithmetic_decode_bit", |b| {
        b.iter(|| {
            let mut decoder = ArithmeticDecoder::new(black_box(&data));
            let mut contexts = vec![0i8; 512];
            for i in 0..1000 {
                let _ = decoder.read_bit(&mut contexts, i % 512);
            }
        })
    });
}

criterion_group!(benches, bench_arithmetic_decoder);
criterion_main!(benches);
