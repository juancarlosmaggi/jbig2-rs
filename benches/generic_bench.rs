use criterion::{Criterion, black_box, criterion_group, criterion_main};
use jbig2_rs::arithmetic::contexts::DecodingContext;
use jbig2_rs::common::error::Jbig2Error;
use jbig2_rs::decoders::generic::{DecodeBitmapParams, decode_bitmap};

fn bench_generic_decode_template0_y0(c: &mut Criterion) {
    // Setup parameters for the optimized path
    let at = [(3, -1), (-3, -1), (2, -2), (-2, -2)];

    let params = DecodeBitmapParams {
        mmr: false,
        width: 2000, // Wide image to stress the y=0 loop
        height: 1,   // Only y=0
        template_index: 0,
        prediction: false,
        skip: None,
        at: &at,
    };

    // Create some dummy data for the arithmetic decoder.
    // A buffer of 0s should suffice to keep it running without erroring (too quickly).
    // The decoder might consume data.
    let data = vec![0u8; 10000];

    c.bench_function("generic_decode_template0_y0", |b| {
        b.iter(|| {
            // We need to create a new context each time because the decoder consumes data.
            // But decoding_context takes a slice.
            let mut decoding_context = DecodingContext::new(black_box(&data), 0, data.len());

            let _ = decode_bitmap(&params, &mut decoding_context);
        })
    });
}

criterion_group!(benches, bench_generic_decode_template0_y0);
criterion_main!(benches);
