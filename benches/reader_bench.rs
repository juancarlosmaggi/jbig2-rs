use criterion::{Criterion, black_box, criterion_group, criterion_main};
use jbig2_rs::common::reader::Reader;

fn bench_reader(c: &mut Criterion) {
    let data = vec![0xAA; 1024 * 1024];

    c.bench_function("reader_read_bits_1", |b| {
        b.iter(|| {
            let mut reader = Reader::new(black_box(&data), 0, data.len());
            let mut sum = 0u64;
            for _ in 0..1000 {
                sum += u64::from(reader.read_bits(1).unwrap_or(0));
            }
            black_box(sum)
        })
    });

    c.bench_function("reader_read_bits_8", |b| {
        b.iter(|| {
            let mut reader = Reader::new(black_box(&data), 0, data.len());
            let mut sum = 0u64;
            for _ in 0..1000 {
                sum += u64::from(reader.read_bits(8).unwrap_or(0));
            }
            black_box(sum)
        })
    });

    c.bench_function("reader_read_bits_32", |b| {
        b.iter(|| {
            let mut reader = Reader::new(black_box(&data), 0, data.len());
            let mut sum = 0u64;
            for _ in 0..1000 {
                sum += u64::from(reader.read_bits(32).unwrap_or(0));
            }
            black_box(sum)
        })
    });

    c.bench_function("reader_read_bits_13", |b| {
        b.iter(|| {
            let mut reader = Reader::new(black_box(&data), 0, data.len());
            let mut sum = 0u64;
            for _ in 0..1000 {
                sum += u64::from(reader.read_bits(13).unwrap_or(0));
            }
            black_box(sum)
        })
    });
}

criterion_group!(benches, bench_reader);
criterion_main!(benches);
