use crate::bitmap::Bitmap;
use crate::bitmap_utils;
use crate::segment::PageInfo;

/// Convert a bitmap to bit-packed format (MSB first)
pub(super) fn bitmap_to_bit_packed(bitmap: &Bitmap) -> Vec<u8> {
    let width = bitmap.width;
    let height = bitmap.height;
    let row_size = width.div_ceil(8); // bytes per row
    let mut packed = vec![0u8; row_size * height];
    for y in 0..height {
        for x in 0..width {
            let pixel = bitmap.get_pixel(x, y);
            if pixel != 0 {
                let byte_index = y * row_size + (x / 8);
                let bit_index = 7 - (x % 8); // MSB first
                packed[byte_index] |= 1 << bit_index;
            }
        }
    }
    packed
}

#[derive(Clone)]
pub struct Jbig2Page {
    pub page_info: PageInfo,
    pub bitmap: Bitmap,
    pub bit_packed_data: Vec<u8>,
}

impl Jbig2Page {
    pub fn to_image_data(&self) -> Vec<u8> {
        let width = self.page_info.width as usize;
        let height = self.page_info.height as usize;
        let mut img_data = vec![0u8; width * height];
        let row_size = width.div_ceil(8);
        for y in 0..height {
            for x in 0..width {
                let byte_index = y * row_size + (x / 8);
                let bit_index = 7 - (x % 8);
                let pixel = if (self.bit_packed_data[byte_index] & (1 << bit_index)) != 0 {
                    255
                } else {
                    0
                };
                img_data[y * width + x] = pixel;
            }
        }
        img_data
    }
}

/// Handle page information and initialization
pub(super) fn on_page_information(
    current_page_info: &mut Option<PageInfo>,
    current_bitmap: &mut Option<Bitmap>,
    current_y: &mut usize,
    pages: &mut Vec<Jbig2Page>,
    info: PageInfo,
) {
    // Validate dimensions: only check for zero, not for max size
    // Large dimensions are allowed - allocation will naturally fail if too large
    if info.width == 0 || info.height == 0 {
        return;
    }

    // If we have a previous page, finalize it
    if let (Some(page_info), Some(bitmap)) = (current_page_info.take(), current_bitmap.take()) {
        let bit_packed_data = bitmap_to_bit_packed(&bitmap);
        pages.push(Jbig2Page {
            page_info,
            bitmap,
            bit_packed_data,
        });
    }

    *current_page_info = Some(info.clone());
    *current_y = 0;
    let width = info.width as usize;
    let height = info.height as usize;
    let bitmap = bitmap_utils::create_initialized_bitmap(width, height, info.default_pixel_value);
    *current_bitmap = Some(bitmap);
}

/// Handle end of stripe
pub(super) fn on_end_of_stripe(current_y: &mut usize, height: usize) {
    if std::env::var_os("JBIG2_RS_TRACE_SEGMENTS").is_some() {
        eprintln!("end_of_stripe: current_y={} height={}", *current_y, height);
    }
    *current_y += height;
}

/// Finalize the current page and add it to the pages vector
pub(super) fn finalize_current_page(
    current_page_info: &mut Option<PageInfo>,
    current_bitmap: &mut Option<Bitmap>,
    pages: &mut Vec<Jbig2Page>,
) {
    if let (Some(page_info), Some(bitmap)) = (current_page_info.take(), current_bitmap.take()) {
        let bit_packed_data = bitmap_to_bit_packed(&bitmap);
        pages.push(Jbig2Page {
            page_info,
            bitmap,
            bit_packed_data,
        });
    }
}
