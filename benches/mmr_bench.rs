use criterion::{Criterion, black_box, criterion_group, criterion_main};
use jbig2_rs::common::reader::Reader;
use jbig2_rs::decoders::mmr::decode_mmr_bitmap;

fn bench_mmr_decode_white_large(c: &mut Criterion) {
    let width = 5000;
    let height = 5000;
    // For white image, we just need a buffer that is large enough to not cause OOB read.
    // However, decode_mmr_bitmap expects valid MMR.
    // The previous implementation used a dummy buffer which probably failed early or decoded garbage without error (if it just read 0s as something valid or exhausted buffer).
    // Let's keep it as is for continuity, but my new benchmark is more robust.
    let data = vec![0xFF; 625 + 10];

    c.bench_function("mmr_decode_white_5000x5000", |b| {
        b.iter(|| {
            let mut reader = Reader::new(black_box(&data), 0, data.len());
            let _ = decode_mmr_bitmap(&mut reader, width, height, false);
        })
    });
}

fn bench_mmr_decode_black_large(c: &mut Criterion) {
    let width = 2560;
    let height = 5000;

    // Generate valid MMR data for a black image of size 2560x5000
    // Using Horizontal mode: H(0, 2560) repeated for each line.

    let mut data = Vec::new();
    let mut current_byte = 0u8;
    let mut bit_pos = 0;

    // Codes:
    // H: 001 (3 bits)
    // White run 0: 00110101 (8 bits)
    // Black run 2560: Makeup 2560 (000000011111, 12 bits) + Term 0 (0000110111, 10 bits)

    let codes = [
        (0b001, 3),          // H
        (0b00110101, 8),     // W0
        (0b000000011111, 12),// B2560 (Makeup)
        (0b0000110111, 10),  // B0 (Term)
    ];

    // Pre-calculate one line of bits to speed up generation?
    // Actually, generation is outside the loop, so it's fine.

    for _ in 0..height {
        for (val, len) in codes.iter() {
            let val = *val;
            let len = *len;
            for i in (0..len).rev() {
                let bit = (val >> i) & 1;
                current_byte = (current_byte << 1) | (bit as u8);
                bit_pos += 1;
                if bit_pos == 8 {
                    data.push(current_byte);
                    current_byte = 0;
                    bit_pos = 0;
                }
            }
        }
    }

    if bit_pos > 0 {
        current_byte <<= 8 - bit_pos;
        data.push(current_byte);
    }
    // Padding
    data.extend_from_slice(&[0; 100]);

    c.bench_function("mmr_decode_black_2560x5000", |b| {
        b.iter(|| {
            let mut reader = Reader::new(black_box(&data), 0, data.len());
            let _ = decode_mmr_bitmap(&mut reader, width, height, false);
        })
    });
}

criterion_group!(benches, bench_mmr_decode_white_large, bench_mmr_decode_black_large);
criterion_main!(benches);
