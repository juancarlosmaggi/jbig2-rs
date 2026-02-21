use criterion::{Criterion, black_box, criterion_group, criterion_main};
use jbig2_rs::bitmap::Bitmap;
use jbig2_rs::bitmap::utils as bitmap_utils;

/// Benchmark bitmap allocation.
fn bench_bitmap_new(c: &mut Criterion) {
    c.bench_function("bitmap_new_1000x1000", |b| {
        b.iter(|| Bitmap::new(black_box(1000), black_box(1000)))
    });
}

/// Benchmark repeated pixel reads.
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

/// Benchmark repeated pixel writes.
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

/// Benchmark compositing a symbol bitmap.
fn bench_draw_symbol(c: &mut Criterion) {
    let mut dst = Bitmap::new(2000, 2000);
    let src = Bitmap::new(100, 100);
    // Fill src with a simple checker pattern.
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
                black_box(0),
            )
        })
    });
}

/// Benchmark counting black pixels.
fn bench_count_black_pixels(c: &mut Criterion) {
    let mut bitmap = Bitmap::new(2000, 2000);
    // Fill with some data
    for i in 0..bitmap.data.len() {
        bitmap.data[i] = (i % 255) as u8;
    }

    c.bench_function("count_black_pixels_2000x2000", |b| {
        b.iter(|| black_box(bitmap.count_black_pixels()))
    });

    let mut bitmap_padded = Bitmap::new(1999, 1999);
    for i in 0..bitmap_padded.data.len() {
        bitmap_padded.data[i] = (i % 255) as u8;
    }
    c.bench_function("count_black_pixels_1999x1999", |b| {
        b.iter(|| black_box(bitmap_padded.count_black_pixels()))
    });
}

criterion_group!(
    benches,
    bench_bitmap_new,
    bench_bitmap_get_pixel,
    bench_bitmap_set_pixel,
    bench_draw_symbol,
    bench_count_black_pixels
);
criterion_main!(benches);
