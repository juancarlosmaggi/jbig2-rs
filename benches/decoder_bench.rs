use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jbig2_rs::arithmetic::ArithmeticDecoder;

fn bench_arithmetic_decoder(c: &mut Criterion) {
    // Create some dummy data for the decoder
    // In a real scenario, this would be valid arithmetic coded data.
    // For benchmarking the `read_bit` overhead, random data is sufficient
    // to exercise the decoding loop, although it might not trigger all
    // renormalization paths realistically.
    let data = vec![0xAA; 1024]; 
    
    c.bench_function("arithmetic_decode_bit", |b| {
        b.iter(|| {
            let mut decoder = ArithmeticDecoder::new(black_box(&data));
            let mut contexts = vec![0i8; 512];
            // Decode a sequence of bits
            for i in 0..1000 {
                let _ = decoder.read_bit(&mut contexts, i % 512);
            }
        })
    });
}

criterion_group!(benches, bench_arithmetic_decoder);
criterion_main!(benches);
