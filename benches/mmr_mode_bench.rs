use criterion::{Criterion, black_box, criterion_group, criterion_main};
use jbig2_rs::common::reader::Reader;
use jbig2_rs::decoders::mmr::decode_mmr_bitmap;

fn bench_mmr_vertical_mode(c: &mut Criterion) {
    let width = 2000;
    let height = 2000;

    let mut data = Vec::new();
    let mut current_byte = 0u8;
    let mut bit_pos = 0;

    // Helper to write bits
    let mut write_bits = |val: u32, len: u32| {
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
    };

    // Row 0: Horizontal mode for 101010...
    // Pairs of White(1), Black(1).
    // H = 001
    // W(1) = 000111
    // B(1) = 010
    // Total 12 bits per pair.
    for _ in 0..(width / 2) {
        write_bits(0b001, 3); // H
        write_bits(0b000111, 6); // W(1)
        write_bits(0b010, 3); // B(1)
    }

    // Row 1..height: Vertical mode V(0) for every transition.
    // Ref line has transitions at 1, 2, 3, ... 2000.
    // Curr line has transitions at 1, 2, 3, ... 2000.
    // So diff is 0.
    // V(0) is '1'.
    // We need 2000 transitions. So 2000 '1's.
    for _ in 1..height {
        for _ in 0..width {
            write_bits(1, 1); // V(0)
        }
    }

    // Flush last byte
    if bit_pos > 0 {
        current_byte <<= 8 - bit_pos;
        data.push(current_byte);
    }
    // Padding
    data.extend_from_slice(&[0; 100]);

    c.bench_function("mmr_decode_vertical_2000x2000", |b| {
        b.iter(|| {
            let mut reader = Reader::new(black_box(&data), 0, data.len());
            let _ = decode_mmr_bitmap(&mut reader, width, height, false);
        })
    });
}

criterion_group!(benches, bench_mmr_vertical_mode);
criterion_main!(benches);
