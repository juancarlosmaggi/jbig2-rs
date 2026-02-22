use criterion::{Criterion, black_box, criterion_group, criterion_main};
use jbig2_rs::huffman::{HuffmanLine, HuffmanTable};

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

criterion_group!(benches, bench_huffman_new);
criterion_main!(benches);
