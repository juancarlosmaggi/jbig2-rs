use criterion::{Criterion, black_box, criterion_group, criterion_main};
use jbig2_rs::common::reader::Reader;
use jbig2_rs::decoders::mmr::decode_mmr_bitmap;

fn bench_mmr_decode_white_large(c: &mut Criterion) {
    // 5000x5000 pixels, all white.
    // In MMR, an all white image is encoded as a series of "pass" or "horizontal" codes.
    // But wait, if it's all white, the first line is all white (starting with white).
    // The reference line starts as all white.
    // So the encoder would just say "pass" or "vertical(0)"?
    // Actually, if the line is unchanged from reference (which is initially all white),
    // it can be encoded very efficiently with Vertical(0) or Pass modes.

    // However, I don't have an encoder to generate valid MMR data.
    // I will use a simple case: a 0-byte buffer.
    // If the data is empty, the decoder might error or finish early.
    // But wait, decode_mmr_bitmap expects valid data.

    // Let's use the property that 0-bits often mean something.
    // But without valid MMR codes, it will fail.

    // I'll try to construct a minimal valid MMR stream for a large white image.
    // For a white line, if the reference line is white, and current line is white.
    // The changing elements:
    // ref: | 0 (white) .......................... | width (changing element)
    // curr: | 0 (white) .......................... | width

    // a0 = -1.
    // b1 = find_changing(ref, -1, white) -> width
    // b2 = find_changing(ref, width, ...) -> width

    // We are at x=0.
    // We want to fill white until width.
    // b1 = width. b2 = width.
    // If we emit V(0) (code 1), a1 = b1 = width.
    // x becomes width.
    // Loop finishes.

    // So a sequence of V(0) codes (bit 1) should decode to identical lines.
    // If I have 1000 lines, I need 1000 '1' bits.
    // 1000 bits = 125 bytes of 0xFF.

    let width = 5000;
    let height = 5000;

    // V(0) is '1'. 5000 lines need 5000 '1' bits.
    // We need 5000 bits. 5000 / 8 = 625 bytes.
    let data = vec![0xFF; 625 + 10]; // +10 padding

    c.bench_function("mmr_decode_white_5000x5000", |b| {
        b.iter(|| {
            let mut reader = Reader::new(black_box(&data), 0, data.len());
            let _ = decode_mmr_bitmap(&mut reader, width, height, false);
        })
    });
}

criterion_group!(benches, bench_mmr_decode_white_large);
criterion_main!(benches);
