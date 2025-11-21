#![no_main]

use libfuzzer_sys::fuzz_target;
use jbig2_rs::arithmetic::ArithmeticDecoder;

fuzz_target!(|data: &[u8]| {
    if data.len() < 5 {
        return;
    }
    
    // Create decoder
    let mut decoder = ArithmeticDecoder::new(data);
    
    // Create contexts of various sizes
    let context_size = (data[0] as usize % 512) + 1;
    let mut contexts = vec![0i8; context_size];
    
    // Try reading bits with different context indices
    for i in 1..data.len().min(20) {
        let ctx_idx = (data[i] as usize) % context_size;
        let _ = decoder.read_bit(&mut contexts, ctx_idx);
    }
    
    // Try reading bounded integers if enough data
    if data.len() > 10 {
        let _ = decoder.read_bounded_int(&mut contexts, 0, 10);
        let _ = decoder.read_bounded_int(&mut contexts, 0, 100);
    }
});
