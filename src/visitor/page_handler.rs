use crate::bitmap::Bitmap;
use crate::bitmap_utils;
use crate::segment::PageInfo;

/// Convert a bitmap to bit-packed rows (MSB first).
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
    /// Expand the packed bitmap into 8-bit grayscale pixels.
    pub fn to_image_data(&self) -> Vec<u8> {
        let width = self.page_info.width as usize;
        let height = self.page_info.height as usize;
        let mut img_data = vec![0u8; width * height];
        let row_size = width.div_ceil(8);
        for y in 0..height {
            for x in 0..width {
                let byte_index = y * row_size + (x / 8);
                let bit_index = 7 - (x % 8);
                // Map packed bits into grayscale pixels.
                let pixel = if (self.bit_packed_data[byte_index] & (1 << bit_index)) != 0 {
                    0
                } else {
                    255
                };
                img_data[y * width + x] = pixel;
            }
        }
        img_data
    }
}

fn finalize_page(page_info: &mut PageInfo, bitmap: &mut Bitmap, current_y: usize) {
    if page_info.height_unknown {
        let final_height = if current_y > 0 {
            current_y
        } else {
            bitmap.height
        };
        let final_height = final_height.max(1);
        bitmap_utils::resize_bitmap_height(bitmap, final_height, page_info.default_pixel_value);
        page_info.height = final_height as u32;
    }
}

/// Initialize a new page, finalizing any existing page state.
pub(super) fn on_page_information(
    current_page_info: &mut Option<PageInfo>,
    current_bitmap: &mut Option<Bitmap>,
    current_y: &mut usize,
    pages: &mut Vec<Jbig2Page>,
    info: PageInfo,
) {
    // Skip empty pages; allocation handles oversized dimensions.
    if info.width == 0 || info.height == 0 {
        return;
    }

    // Finalize the previous page, if any.
    if let (Some(mut page_info), Some(mut bitmap)) =
        (current_page_info.take(), current_bitmap.take())
    {
        finalize_page(&mut page_info, &mut bitmap, *current_y);
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

/// Advance the current vertical stripe offset.
pub(super) fn on_end_of_stripe(
    current_page_info: &mut Option<PageInfo>,
    current_bitmap: &mut Option<Bitmap>,
    current_y: &mut usize,
    end_row: usize,
) {
    if std::env::var_os("JBIG2_RS_TRACE_SEGMENTS").is_some() {
        eprintln!("end_of_stripe: current_y={} end_row={}", *current_y, end_row);
    }
    let next_row = end_row.saturating_add(1);
    *current_y = next_row;
    if let (Some(page_info), Some(bitmap)) =
        (current_page_info.as_mut(), current_bitmap.as_mut())
    {
        if page_info.height_unknown {
            let stripe = page_info.stripe_size as usize;
            let mut next_height = next_row.max(1);
            if stripe > 0 {
                next_height = next_height.saturating_add(stripe);
            }
            bitmap_utils::resize_bitmap_height(bitmap, next_height, page_info.default_pixel_value);
            page_info.height = bitmap.height as u32;
        }
    }
}

/// Finalize the current page and store it in the page list.
pub(super) fn finalize_current_page(
    current_page_info: &mut Option<PageInfo>,
    current_bitmap: &mut Option<Bitmap>,
    current_y: usize,
    pages: &mut Vec<Jbig2Page>,
) {
    if let (Some(mut page_info), Some(mut bitmap)) =
        (current_page_info.take(), current_bitmap.take())
    {
        finalize_page(&mut page_info, &mut bitmap, current_y);
        let bit_packed_data = bitmap_to_bit_packed(&bitmap);
        pages.push(Jbig2Page {
            page_info,
            bitmap,
            bit_packed_data,
        });
    }
}
