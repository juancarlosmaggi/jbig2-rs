use crate::bitmap::Bitmap;

/// Allocate a bitmap filled with `default_value` (0 for white, non-zero for black).
pub fn create_initialized_bitmap(width: usize, height: usize, default_value: u8) -> Bitmap {
    let stride = (width + 7) >> 3;
    let data = if default_value != 0 {
        vec![0xff; stride * height]
    } else {
        vec![0; stride * height]
    };
    Bitmap {
        data,
        width,
        height,
        stride,
    }
}

/// Apply a composition operator to a single destination/source pixel pair.
pub fn apply_combination_operator(dst_pixel: u8, src_pixel: u8, operator: u8) -> u8 {
    match operator {
        0 => dst_pixel | src_pixel,
        1 => dst_pixel & src_pixel,
        2 => dst_pixel ^ src_pixel,
        3 => !(dst_pixel ^ src_pixel) & 1,
        4 => src_pixel,
        _ => dst_pixel,
    }
}

/// Draw a symbol bitmap into the destination at the given offset.
pub fn draw_symbol_at_position(
    bitmap: &mut Bitmap,
    symbol: &Bitmap,
    offset_x: i32,
    offset_y: i32,
    combination_operator: u8,
) {
    bitmap.combine(
        symbol,
        offset_x as isize,
        offset_y as isize,
        combination_operator,
    );
}
