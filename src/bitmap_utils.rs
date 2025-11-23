use crate::bitmap::Bitmap;
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
pub fn apply_combination_operator(dst_pixel: u8, src_pixel: u8, operator: u8) -> u8 {
    match operator {
        0 => dst_pixel | src_pixel,        // OR
        1 => dst_pixel & src_pixel,        // AND
        2 => dst_pixel ^ src_pixel,        // XOR
        3 => !(dst_pixel ^ src_pixel) & 1, // XNOR
        4 => src_pixel,                    // REPLACE
        _ => dst_pixel,                    // undefined: no-op
    }
}
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
