use criterion::{Criterion, black_box, criterion_group, criterion_main};
use jbig2_rs::common::reader::Reader;
use jbig2_rs::decoders::utils::read_uncompressed_bitmap;

fn bench_uncompressed_bitmap(c: &mut Criterion) {
    let width: usize = 1024;
    let height: usize = 1024;
    let stride = width.div_ceil(8);
    let data_size = height * stride;
    let data = vec![0xAA; data_size];

    c.bench_function("read_uncompressed_bitmap_1024x1024", |b| {
        b.iter(|| {
            let mut reader = Reader::new(black_box(&data), 0, data.len());
            read_uncompressed_bitmap(&mut reader, width, height).unwrap()
        })
    });

    let width_small: usize = 32;
    let height_small: usize = 32;
    let stride_small = width_small.div_ceil(8);
    let data_size_small = height_small * stride_small;
    let data_small = vec![0xAA; data_size_small];

    c.bench_function("read_uncompressed_bitmap_32x32", |b| {
        b.iter(|| {
            let mut reader = Reader::new(black_box(&data_small), 0, data_small.len());
            read_uncompressed_bitmap(&mut reader, width_small, height_small).unwrap()
        })
    });
}

criterion_group!(benches, bench_uncompressed_bitmap);
criterion_main!(benches);
