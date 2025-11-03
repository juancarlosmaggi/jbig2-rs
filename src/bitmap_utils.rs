use crate::bitmap::Bitmap;

pub fn create_initialized_bitmap(width: usize, height: usize, default_value: u8) -> Bitmap {
    let mut bitmap = Bitmap::new(width, height);
    if default_value != 0 {
        for y in 0..height {
            for x in 0..width {
                bitmap.set_pixel(x, y, 1);
            }
        }
    }
    bitmap
}

pub fn apply_combination_operator(dst_pixel: u8, src_pixel: u8, operator: usize) -> u8 {
    match operator {
        0 => dst_pixel | src_pixel, // OR
        2 => dst_pixel ^ src_pixel, // XOR
        _ => dst_pixel,             // undefined: no-op
    }
}

pub fn apply_page_combination_operator(dst_pixel: u8, src_pixel: u8, operator: u8) -> u8 {
    match operator {
        0 => dst_pixel | src_pixel, // OR
        1 => dst_pixel & src_pixel, // AND
        2 => dst_pixel ^ src_pixel, // XOR
        3 => !(dst_pixel ^ src_pixel) & 1, // XNOR
        4 => src_pixel,             // REPLACE
        _ => dst_pixel,             // undefined: no-op
    }
}

pub fn draw_symbol_at_position(
    bitmap: &mut Bitmap,
    symbol: &Bitmap,
    offset_x: i32,
    offset_y: i32,
    transposed: bool,
    combination_operator: usize,
) {
    for y in 0..symbol.height {
        let row = offset_y + y as i32;
        if row < 0 || row >= bitmap.height as i32 {
            continue;
        }
        for x in 0..symbol.width {
            let col = offset_x + x as i32;
            if col >= 0 && col < bitmap.width as i32 {
                let (src_x, src_y) = if transposed { (x, y) } else { (y, x) };
                let src_pixel = symbol.get_pixel(src_x, src_y);
                let dst_pixel = bitmap.get_pixel(col as usize, row as usize);
                let new_pixel = apply_combination_operator(dst_pixel, src_pixel, combination_operator);
                bitmap.set_pixel(col as usize, row as usize, new_pixel);
            }
        }
    }
}