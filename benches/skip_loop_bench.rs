use criterion::{Criterion, black_box, criterion_group, criterion_main};
use jbig2_rs::bitmap::core::Bitmap;

fn bench_skip_loop(c: &mut Criterion) {
    let mut group = c.benchmark_group("skip_loop");

    let grid_width = 1000;
    let grid_height = 1000;
    let grid_vector_x = 256;
    let grid_vector_y = 0;
    let grid_offset_x = 0;
    let grid_offset_y = 0;
    let pattern_width = 16;
    let pattern_height = 16;
    let region_width = 10000;
    let region_height = 10000;

    group.bench_function("original", |b| {
        b.iter(|| {
            let mut skip = Bitmap::new(grid_width, grid_height);
            for mg in 0..grid_height {
                let base_x = grid_offset_x + mg as i64 * grid_vector_y;
                let base_y = grid_offset_y + mg as i64 * grid_vector_x;
                let mut x = base_x;
                let mut y = base_y;
                for ng in 0..grid_width {
                    let region_x = x >> 8;
                    let region_y = y >> 8;
                    let outside = region_x + pattern_width <= 0
                        || region_x >= region_width
                        || region_y + pattern_height <= 0
                        || region_y >= region_height;
                    if outside {
                        skip.set_pixel(ng, mg, 1);
                    }
                    x += grid_vector_x;
                    y -= grid_vector_y;
                }
            }
            black_box(skip);
        });
    });

    group.bench_function("fast_range_check_row_start", |b| {
        b.iter(|| {
            let mut skip = Bitmap::new(grid_width, grid_height);
            for mg in 0..grid_height {
                let base_x = grid_offset_x + mg as i64 * grid_vector_y;
                let base_y = grid_offset_y + mg as i64 * grid_vector_x;
                let mut x = base_x;
                let mut y = base_y;
                let row_start = unsafe { skip.get_row_start_index_unchecked(mg) };

                for ng in 0..grid_width {
                    let region_x = x >> 8;
                    let region_y = y >> 8;

                    let inside = region_x > -pattern_width
                        && region_x < region_width
                        && region_y > -pattern_height
                        && region_y < region_height;

                    if !inside {
                        unsafe { skip.set_pixel_at_index_unchecked(row_start, ng, 1) };
                    }
                    x += grid_vector_x;
                    y -= grid_vector_y;
                }
            }
            black_box(skip);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_skip_loop);
criterion_main!(benches);
