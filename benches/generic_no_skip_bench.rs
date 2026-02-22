use criterion::{Criterion, black_box, criterion_group, criterion_main};
use jbig2_rs::arithmetic::contexts::DecodingContext;
use jbig2_rs::decoders::generic::{DecodeBitmapParams, decode_bitmap};

fn bench_generic_no_skip(c: &mut Criterion) {
    // Parameters that bypass the template-0 fast path.
    // Changing template_index to 1 forces the general path.
    let at = [(0, 0); 4]; // Dummy AT pixels
    let params = DecodeBitmapParams {
        mmr: false,
        width: 1024,
        height: 1024,
        template_index: 1,
        prediction: false,
        skip: None,
        at: &at,
    };

    // Create sufficient dummy data.
    let data = vec![0u8; 1024 * 1024]; // 1MB should be enough

    c.bench_function("generic_no_skip", |b| {
        b.iter(|| {
            // Re-create context each iteration
            let mut decoding_context = DecodingContext::new(black_box(&data), 0, data.len());
            let _ = decode_bitmap(&params, &mut decoding_context);
        })
    });
}

criterion_group!(benches, bench_generic_no_skip);
criterion_main!(benches);
