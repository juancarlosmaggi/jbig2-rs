#![no_main]

use libfuzzer_sys::fuzz_target;
use jbig2_rs::arithmetic::ArithmeticDecoder;

fuzz_target!(|data: &[u8]| {
    if data.len() < 5 {
        return;
    }

    // Initialize decoder and context state.
    let mut decoder = ArithmeticDecoder::new(data);

    let context_size = (data[0] as usize % 512) + 1;
    let mut contexts = vec![0i8; context_size];

    // Read bits with varying context indices.
    for i in 1..data.len().min(20) {
        let ctx_idx = (data[i] as usize) % context_size;
        let _ = decoder.read_bit(&mut contexts, ctx_idx);
    }
});
