use criterion::{Criterion, black_box, criterion_group, criterion_main};
use jbig2_rs::common::reader::Reader;
use jbig2_rs::huffman::{HuffmanLine, HuffmanTable, get_standard_table};

fn get_table_lines() -> Vec<HuffmanLine> {
    // Table 10 (from standard_tables.rs)
    vec![
        HuffmanLine::new(vec![-21, 7, 4, 0x7a]),
        HuffmanLine::new(vec![-5, 8, 0, 0xfc]),
        HuffmanLine::new(vec![-4, 7, 0, 0x7b]),
        HuffmanLine::new(vec![-3, 5, 0, 0x18]),
        HuffmanLine::new(vec![-2, 2, 2, 0x0]),
        HuffmanLine::new(vec![2, 5, 0, 0x19]),
        HuffmanLine::new(vec![3, 6, 0, 0x36]),
        HuffmanLine::new(vec![4, 7, 0, 0x7c]),
        HuffmanLine::new(vec![5, 8, 0, 0xfd]),
        HuffmanLine::new(vec![6, 2, 6, 0x1]),
        HuffmanLine::new(vec![70, 5, 5, 0x1a]),
        HuffmanLine::new(vec![102, 6, 5, 0x37]),
        HuffmanLine::new(vec![134, 6, 6, 0x38]),
        HuffmanLine::new(vec![198, 6, 7, 0x39]),
        HuffmanLine::new(vec![326, 6, 8, 0x3a]),
        HuffmanLine::new(vec![582, 6, 9, 0x3b]),
        HuffmanLine::new(vec![1094, 6, 10, 0x3c]),
        HuffmanLine::new(vec![2118, 7, 11, 0x7d]),
        HuffmanLine::new(vec![-22, 8, 32, 0xfe, 1]), // lower
        HuffmanLine::new(vec![4166, 8, 32, 0xff]),   // upper
        HuffmanLine::new(vec![2, 0x2]),              // OOB
    ]
}

fn bench_huffman_new(c: &mut Criterion) {
    let lines = get_table_lines();
    c.bench_function("huffman_new_table_10", |b| {
        b.iter(|| {
            // We clone `lines` because `HuffmanTable::new` consumes it.
            // The cloning cost is included in the measurement, but since it's a small vector
            // of structs with mostly primitive fields, it should be fast compared to the tree building.
            // More importantly, it's constant between baseline and optimized runs.
            HuffmanTable::new(black_box(lines.clone()), black_box(true))
        })
    });
}

fn bench_huffman_decode_zeros(c: &mut Criterion) {
    let table = get_standard_table(1).unwrap();
    let data_len = 10000;
    let data = vec![0u8; data_len];

    c.bench_function("huffman_decode_table_1_zeros", |b| {
        b.iter(|| {
            let mut reader = Reader::new(black_box(&data), 0, data.len());
            while let Ok(_) = table.decode_entry(&mut reader) {
                if reader.get_position() >= data_len - 1 {
                    break;
                }
            }
        })
    });
}

fn bench_huffman_decode_mixed(c: &mut Criterion) {
    let table = get_standard_table(1).unwrap();
    let data_len = 10000;
    let pattern = vec![0xAA, 0x55, 0xFF, 0x00, 0x12, 0x34];
    let data: Vec<u8> = pattern.iter().cycle().take(data_len).cloned().collect();

    c.bench_function("huffman_decode_table_1_mixed", |b| {
        b.iter(|| {
            let mut reader = Reader::new(black_box(&data), 0, data.len());
            while let Ok(_) = table.decode_entry(&mut reader) {
                if reader.get_position() >= data_len - 1 {
                    break;
                }
            }
        })
    });
}

fn bench_huffman_decode_table_10_mixed(c: &mut Criterion) {
    let lines = get_table_lines();
    let table = HuffmanTable::new(lines, true);
    let data_len = 10000;
    let pattern = vec![0xAA, 0x55, 0xFF, 0x00, 0x12, 0x34, 0x9A, 0xBC];
    let data: Vec<u8> = pattern.iter().cycle().take(data_len).cloned().collect();

    c.bench_function("huffman_decode_table_10_mixed", |b| {
        b.iter(|| {
            let mut reader = Reader::new(black_box(&data), 0, data.len());
            while let Ok(_) = table.decode_entry(&mut reader) {
                if reader.get_position() >= data_len - 1 {
                    break;
                }
            }
        })
    });
}

criterion_group!(
    benches,
    bench_huffman_new,
    bench_huffman_decode_zeros,
    bench_huffman_decode_mixed,
    bench_huffman_decode_table_10_mixed
);
criterion_main!(benches);
