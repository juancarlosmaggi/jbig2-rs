use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jbig2_rs::bitmap::Bitmap;
use jbig2_rs::bitmap_utils;

fn bench_bitmap_new(c: &mut Criterion) {
    c.bench_function("bitmap_new_1000x1000", |b| {
        b.iter(|| Bitmap::new(black_box(1000), black_box(1000)))
    });
}

fn bench_bitmap_get_pixel(c: &mut Criterion) {
    let bitmap = Bitmap::new(1000, 1000);
    c.bench_function("bitmap_get_pixel", |b| {
        b.iter(|| {
            for y in 0..100 {
                for x in 0..100 {
                    black_box(bitmap.get_pixel(x, y));
                }
            }
        })
    });
}

fn bench_bitmap_set_pixel(c: &mut Criterion) {
    let mut bitmap = Bitmap::new(1000, 1000);
    c.bench_function("bitmap_set_pixel", |b| {
        b.iter(|| {
            for y in 0..100 {
                for x in 0..100 {
                    bitmap.set_pixel(x, y, 1);
                }
            }
        })
    });
}

fn bench_draw_symbol(c: &mut Criterion) {
    let mut dst = Bitmap::new(2000, 2000);
    let src = Bitmap::new(100, 100);
    // Fill src with some pattern
    let mut src = src;
    for y in 0..100 {
        for x in 0..100 {
            if (x + y) % 2 == 0 {
                src.set_pixel(x, y, 1);
            }
        }
    }

    c.bench_function("draw_symbol_100x100", |b| {
        b.iter(|| {
            bitmap_utils::draw_symbol_at_position(
                black_box(&mut dst),
                black_box(&src),
                black_box(500),
                black_box(500),
                black_box(0), // OR
            )
        })
    });
}

criterion_group!(benches, bench_bitmap_new, bench_bitmap_get_pixel, bench_bitmap_set_pixel, bench_draw_symbol);
criterion_main!(benches);
