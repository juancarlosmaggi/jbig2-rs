//! Bitmap storage and operations for 1-bit images.
//!
//! The [`Bitmap`] type owns packed pixel data and provides safe accessors,
//! bounds checks, and composition helpers used by region decoding.

/// Packed 1bpp bitmap with row-aligned storage.
///
/// The bitmap data is stored as a packed byte vector, with 8 pixels per byte.
/// The stride (bytes per row) is automatically calculated based on the width.
///
/// # Fields
///
/// - `data` - Raw packed bitmap data
/// - `width` - Width in pixels
/// - `height` - Height in pixels
/// - `stride` - Number of bytes per row
#[derive(Clone)]
pub struct Bitmap {
    pub data: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub stride: usize, // bytes per row
}

impl Bitmap {
    /// Create a new bitmap initialized to all zeros.
    ///
    /// # Arguments
    ///
    /// * `width` - Width in pixels
    /// * `height` - Height in pixels
    ///
    pub fn new(width: usize, height: usize) -> Self {
        // Use checked arithmetic to avoid overflow in stride and buffer sizing.
        let stride = width
            .checked_add(7)
            .expect("width too large for stride calculation")
            >> 3;

        let buffer_size = stride.checked_mul(height).expect("buffer size overflow");

        let data = vec![0; buffer_size];

        Bitmap {
            data,
            width,
            height,
            stride,
        }
    }

    /// Return the pixel value at `(x, y)`, or 0 for out-of-bounds reads.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    ///
    /// # Returns
    ///
    /// - `1` if the pixel is set (black/foreground)
    /// - `0` if the pixel is clear (white/background) or out of bounds
    pub fn get_pixel(&self, x: usize, y: usize) -> u8 {
        if y >= self.height || x >= self.width {
            return 0;
        }
        let byte_index = y * self.stride + (x >> 3);
        if byte_index >= self.data.len() {
            return 0;
        }
        let bit_index = 7 - (x & 7);
        (self.data[byte_index] >> bit_index) & 1
    }

    /// Return the pixel value at `(x, y)` without bounds checks.
    ///
    /// Caller must ensure `x < width` and `y < height`.
    #[inline(always)]
    pub fn get_pixel_unchecked(&self, x: usize, y: usize) -> u8 {
        debug_assert!(x < self.width && y < self.height);
        let byte_index = y * self.stride + (x >> 3);
        let bit_index = 7 - (x & 7);
        unsafe { (*self.data.get_unchecked(byte_index) >> bit_index) & 1 }
    }

    /// Set the pixel at `(x, y)`; out-of-bounds writes are ignored.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    /// * `value` - Pixel value (non-zero for set/black, 0 for clear/white)
    pub fn set_pixel(&mut self, x: usize, y: usize, value: u8) {
        if y >= self.height || x >= self.width {
            return; // Silently ignore out-of-bounds writes
        }
        let byte_index = y * self.stride + (x >> 3);
        if byte_index >= self.data.len() {
            return;
        }
        let bit_index = 7 - (x & 7);
        if value != 0 {
            self.data[byte_index] |= 1 << bit_index;
        } else {
            self.data[byte_index] &= !(1 << bit_index);
        }
    }

    /// Set the pixel at `(x, y)` without bounds checks.
    ///
    /// Caller must ensure `x < width` and `y < height`.
    #[inline(always)]
    pub fn set_pixel_unchecked(&mut self, x: usize, y: usize, value: u8) {
        debug_assert!(x < self.width && y < self.height);
        let byte_index = y * self.stride + (x >> 3);
        let bit_index = 7 - (x & 7);
        unsafe {
            let byte = self.data.get_unchecked_mut(byte_index);
            if value != 0 {
                *byte |= 1 << bit_index;
            } else {
                *byte &= !(1 << bit_index);
            }
        }
    }

    /// Return the byte offset for the start of the given row.
    /// Returns None if y is out of bounds.
    #[inline(always)]
    pub fn get_row_start_index(&self, y: usize) -> Option<usize> {
        if y >= self.height {
            None
        } else {
            Some(y * self.stride)
        }
    }

    /// Return the byte offset for the start of the given row without bounds checks.
    #[inline(always)]
    pub unsafe fn get_row_start_index_unchecked(&self, y: usize) -> usize {
        debug_assert!(y < self.height);
        y * self.stride
    }

    /// Return the pixel value at `(x, row_start_index)` without bounds checks.
    ///
    /// Caller must ensure `x < width` and `row_start_index` is valid for `y < height`.
    #[inline(always)]
    pub unsafe fn get_pixel_at_index_unchecked(&self, row_start_index: usize, x: usize) -> u8 {
        debug_assert!(x < self.width);
        let byte_index = row_start_index + (x >> 3);
        let bit_index = 7 - (x & 7);
        unsafe { (*self.data.get_unchecked(byte_index) >> bit_index) & 1 }
    }

    /// Set the pixel at `(x, row_start_index)` without bounds checks.
    ///
    /// Caller must ensure `x < width` and `row_start_index` is valid for `y < height`.
    #[inline(always)]
    pub unsafe fn set_pixel_at_index_unchecked(
        &mut self,
        row_start_index: usize,
        x: usize,
        value: u8,
    ) {
        debug_assert!(x < self.width);
        let byte_index = row_start_index + (x >> 3);
        let bit_index = 7 - (x & 7);
        unsafe {
            let byte = self.data.get_unchecked_mut(byte_index);
            if value != 0 {
                *byte |= 1 << bit_index;
            } else {
                *byte &= !(1 << bit_index);
            }
        }
    }

    /// Count the number of set pixels across the entire bitmap.
    pub fn count_black_pixels(&self) -> u32 {
        if self.width == 0 || self.height == 0 {
            return 0;
        }
        let full_bytes = self.width / 8;
        let rem_bits = self.width % 8;

        // Fast path: if no intra-row padding, sum all relevant bytes in one pass.
        if rem_bits == 0 && self.stride == full_bytes {
            return self.data[..self.stride * self.height]
                .iter()
                .map(|&b| b.count_ones())
                .sum();
        }

        let mask = if rem_bits == 0 { 0 } else { 0xFFu8 << (8 - rem_bits) };
        self.data[..self.stride * self.height]
            .chunks_exact(self.stride)
            .map(|row| {
                let mut count = row[..full_bytes].iter().map(|&b| b.count_ones()).sum::<u32>();
                if rem_bits != 0 {
                    count += (row[full_bytes] & mask).count_ones();
                }
                count
            })
            .sum()
    }

    /// Return `(min, max, full_rows)` black pixel counts per row.
    pub fn row_black_stats(&self) -> (u32, u32, u32) {
        if self.width == 0 || self.height == 0 {
            return (0, 0, 0);
        }
        let full_bytes = self.width / 8;
        let rem_bits = self.width % 8;
        let mask = if rem_bits == 0 { 0 } else { 0xFFu8 << (8 - rem_bits) };

        let mut min_row = u32::MAX;
        let mut max_row = 0u32;
        let mut full_rows = 0u32;

        for row in self.data[..self.stride * self.height].chunks_exact(self.stride) {
            let row_count = {
                let mut count = row[..full_bytes].iter().map(|&b| b.count_ones()).sum::<u32>();
                if rem_bits != 0 {
                    count += (row[full_bytes] & mask).count_ones();
                }
                count
            };

            if row_count < min_row {
                min_row = row_count;
            }
            if row_count > max_row {
                max_row = row_count;
            }
            if row_count as usize == self.width {
                full_rows = full_rows.saturating_add(1);
            }
        }

        if min_row == u32::MAX {
            min_row = 0;
        }
        (min_row, max_row, full_rows)
    }

    /// Combine another bitmap at `(x, y)` using the operator code.
    ///
    /// This uses a byte-oriented loop for aligned runs.
    ///
    /// # Arguments
    ///
    /// * `other` - The source bitmap to combine
    /// * `x` - X coordinate in this bitmap where the source should be placed
    /// * `y` - Y coordinate in this bitmap where the source should be placed
    /// * `operator` - Combination operator (0=OR, 1=AND, 2=XOR, 3=XNOR, 4=REPLACE)
    pub fn combine(&mut self, other: &Bitmap, x: isize, y: isize, operator: u8) {
        if operator == 0 {
            self.combine_or(other, x, y);
            return;
        }
        // Clip to destination bounds before iterating.
        let start_y = y.max(0) as usize;
        let end_y = (y + other.height as isize).min(self.height as isize).max(0) as usize;

        if start_y >= end_y {
            return;
        }

        let start_x = x.max(0) as usize;
        let end_x = (x + other.width as isize).min(self.width as isize).max(0) as usize;

        if start_x >= end_x {
            return;
        }

        // Track corresponding source offsets for each clipped row.
        let src_start_y = (start_y as isize - y) as usize;
        let src_start_x = (start_x as isize - x) as usize;
        let _width = end_x - start_x;

        if operator == 0 && (start_x & 7) == 0 && (src_start_x & 7) == 0 {
            let width = end_x - start_x;
            let full_bytes = width >> 3;
            let rem_bits = width & 7;
            let row_bytes = full_bytes + usize::from(rem_bits != 0);
            let src_byte_offset = src_start_x >> 3;
            let dst_byte_offset = start_x >> 3;

            for i in 0..(end_y - start_y) {
                let dst_y = start_y + i;
                let src_y = src_start_y + i;

                let dst_row_start = dst_y * self.stride + dst_byte_offset;
                let src_row_start = src_y * other.stride + src_byte_offset;

                let dst_row = &mut self.data[dst_row_start..dst_row_start + row_bytes];
                let src_row = &other.data[src_row_start..src_row_start + row_bytes];

                or_bytes_unaligned(&mut dst_row[..full_bytes], &src_row[..full_bytes]);
                if rem_bits != 0 {
                    let mask = 0xFFu8 << (8 - rem_bits);
                    dst_row[full_bytes] |= src_row[full_bytes] & mask;
                }
            }
            return;
        }

        // Process the row in chunks to minimize per-pixel work.
        for i in 0..(end_y - start_y) {
            let dst_y = start_y + i;
            let src_y = src_start_y + i;

            let dst_row_start = dst_y * self.stride;
            let src_row_start = src_y * other.stride;

            let mut current_x = start_x;
            let mut current_src_x = src_start_x;
            let src_row_end = src_row_start + other.stride;

            while current_x < end_x {
                let dst_byte_idx = dst_row_start + (current_x >> 3);
                let bits_left_in_byte = 8 - (current_x & 7);
                let bits_to_process = bits_left_in_byte.min(end_x - current_x);

                // Align the source bits to the destination byte boundary.
                let src_byte_idx = src_row_start + (current_src_x >> 3);
                let src_byte = other.data[src_byte_idx];
                let next_byte = if src_byte_idx + 1 < src_row_end {
                    other.data[src_byte_idx + 1]
                } else {
                    0
                };
                let src_word = ((src_byte as u16) << 8) | next_byte as u16;
                let src_bit_offset = current_src_x & 7;
                let mut src_aligned = ((src_word << src_bit_offset) >> 8) as u8;
                let dst_bit_offset = current_x & 7;
                if dst_bit_offset != 0 {
                    src_aligned >>= dst_bit_offset;
                }

                // Mask off only the destination bits covered by this chunk.
                let mask_high = 0xFFu8 >> (current_x & 7);
                let shift_low = (current_x & 7) + bits_to_process;
                let mask_low = if shift_low >= 8 {
                    0xFF
                } else {
                    !(0xFFu8 >> shift_low)
                };
                let mask = mask_high & mask_low;

                let dst_byte = self.data[dst_byte_idx];
                let mut new_byte = dst_byte;

                match operator {
                    0 => new_byte |= src_aligned & mask, // OR
                    1 => new_byte = (dst_byte & src_aligned & mask) | (dst_byte & !mask), // AND within mask, preserve outside
                    2 => new_byte ^= src_aligned & mask,                                  // XOR
                    3 => {
                        // XNOR.
                        let xor = dst_byte ^ src_aligned;
                        new_byte = (new_byte & !mask) | (!xor & mask);
                    }
                    4 => {
                        // REPLACE.
                        new_byte = (new_byte & !mask) | (src_aligned & mask);
                    }
                    _ => {}
                }

                self.data[dst_byte_idx] = new_byte;

                current_x += bits_to_process;
                current_src_x += bits_to_process;
            }
        }
    }

    pub(crate) fn combine_or(&mut self, other: &Bitmap, x: isize, y: isize) {
        // Clip to destination bounds before iterating.
        let start_y = y.max(0) as usize;
        let end_y = (y + other.height as isize).min(self.height as isize).max(0) as usize;

        if start_y >= end_y {
            return;
        }

        let start_x = x.max(0) as usize;
        let end_x = (x + other.width as isize).min(self.width as isize).max(0) as usize;

        if start_x >= end_x {
            return;
        }

        // Track corresponding source offsets for each clipped row.
        let src_start_y = (start_y as isize - y) as usize;
        let src_start_x = (start_x as isize - x) as usize;
        let width = end_x - start_x;

        if (start_x & 7) == 0 && (src_start_x & 7) == 0 {
            let full_bytes = width >> 3;
            let rem_bits = width & 7;
            let row_bytes = full_bytes + usize::from(rem_bits != 0);
            let src_byte_offset = src_start_x >> 3;
            let dst_byte_offset = start_x >> 3;

            for i in 0..(end_y - start_y) {
                let dst_y = start_y + i;
                let src_y = src_start_y + i;

                let dst_row_start = dst_y * self.stride + dst_byte_offset;
                let src_row_start = src_y * other.stride + src_byte_offset;

                let dst_row = &mut self.data[dst_row_start..dst_row_start + row_bytes];
                let src_row = &other.data[src_row_start..src_row_start + row_bytes];

                or_bytes_unaligned(&mut dst_row[..full_bytes], &src_row[..full_bytes]);
                if rem_bits != 0 {
                    let mask = 0xFFu8 << (8 - rem_bits);
                    dst_row[full_bytes] |= src_row[full_bytes] & mask;
                }
            }
            return;
        }

        let dst_bit_offset = start_x & 7;
        for i in 0..(end_y - start_y) {
            let dst_y = start_y + i;
            let src_y = src_start_y + i;

            let dst_row_start = dst_y * self.stride;
            let src_row_start = src_y * other.stride;
            let src_row_end = src_row_start + other.stride;

            let mut dst_x = start_x;
            let mut src_x = src_start_x;
            let mut remaining = width;

            if dst_bit_offset != 0 {
                let bits = remaining.min(8 - dst_bit_offset);
                let dst_byte_idx = dst_row_start + (dst_x >> 3);
                let src_byte_idx = src_row_start + (src_x >> 3);
                let src_byte = other.data[src_byte_idx];
                let next_byte = if src_byte_idx + 1 < src_row_end {
                    other.data[src_byte_idx + 1]
                } else {
                    0
                };
                let src_word = ((src_byte as u16) << 8) | next_byte as u16;
                let mut src_aligned = ((src_word << (src_x & 7)) >> 8) as u8;
                src_aligned >>= dst_bit_offset;
                let mask_high = 0xFFu8 >> dst_bit_offset;
                let shift_low = dst_bit_offset + bits;
                let mask_low = if shift_low >= 8 {
                    0xFF
                } else {
                    !(0xFFu8 >> shift_low)
                };
                let mask = mask_high & mask_low;
                self.data[dst_byte_idx] |= src_aligned & mask;
                dst_x += bits;
                src_x += bits;
                remaining -= bits;
            }

            if remaining >= 8 {
                let full_bytes = remaining >> 3;
                let dst_byte_idx = dst_row_start + (dst_x >> 3);
                let src_byte_idx = src_row_start + (src_x >> 3);
                let src_bit_offset = src_x & 7;
                for j in 0..full_bytes {
                    let src_idx = src_byte_idx + j;
                    let src_byte = other.data[src_idx];
                    let next_byte = if src_idx + 1 < src_row_end {
                        other.data[src_idx + 1]
                    } else {
                        0
                    };
                    let src_word = ((src_byte as u16) << 8) | next_byte as u16;
                    let src_aligned = ((src_word << src_bit_offset) >> 8) as u8;
                    self.data[dst_byte_idx + j] |= src_aligned;
                }
                let bits = full_bytes << 3;
                dst_x += bits;
                src_x += bits;
                remaining -= bits;
            }

            if remaining > 0 {
                let dst_byte_idx = dst_row_start + (dst_x >> 3);
                let src_byte_idx = src_row_start + (src_x >> 3);
                let src_bit_offset = src_x & 7;
                let src_byte = other.data[src_byte_idx];
                let next_byte = if src_byte_idx + 1 < src_row_end {
                    other.data[src_byte_idx + 1]
                } else {
                    0
                };
                let src_word = ((src_byte as u16) << 8) | next_byte as u16;
                let src_aligned = ((src_word << src_bit_offset) >> 8) as u8;
                let mask = 0xFFu8 << (8 - remaining);
                self.data[dst_byte_idx] |= src_aligned & mask;
            }
        }
    }
}

fn or_bytes_unaligned(dst: &mut [u8], src: &[u8]) {
    let len = dst.len().min(src.len());
    let mut idx = 0usize;
    unsafe {
        while idx + 8 <= len {
            let dst_ptr = dst.as_mut_ptr().add(idx) as *mut u64;
            let src_ptr = src.as_ptr().add(idx) as *const u64;
            let dst_val = std::ptr::read_unaligned(dst_ptr);
            let src_val = std::ptr::read_unaligned(src_ptr);
            std::ptr::write_unaligned(dst_ptr, dst_val | src_val);
            idx += 8;
        }
    }
    while idx < len {
        dst[idx] |= src[idx];
        idx += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn combine_naive(dst: &mut Bitmap, other: &Bitmap, x: isize, y: isize, operator: u8) {
        let start_y = y.max(0) as usize;
        let end_y = (y + other.height as isize).min(dst.height as isize).max(0) as usize;
        if start_y >= end_y {
            return;
        }

        let start_x = x.max(0) as usize;
        let end_x = (x + other.width as isize).min(dst.width as isize).max(0) as usize;
        if start_x >= end_x {
            return;
        }

        for dst_y in start_y..end_y {
            let src_y = (dst_y as isize - y) as usize;
            for dst_x in start_x..end_x {
                let src_x = (dst_x as isize - x) as usize;
                let src = other.get_pixel(src_x, src_y);
                let dst_val = dst.get_pixel(dst_x, dst_y);
                let value = match operator {
                    0 => dst_val | src,
                    1 => dst_val & src,
                    2 => dst_val ^ src,
                    3 => (dst_val ^ src) ^ 1,
                    4 => src,
                    _ => dst_val | src,
                };
                dst.set_pixel(dst_x, dst_y, value);
            }
        }
    }

    #[test]
    fn test_bitmap_creation() {
        let bitmap = Bitmap::new(10, 10);
        assert_eq!(bitmap.width, 10);
        assert_eq!(bitmap.height, 10);
        assert_eq!(bitmap.stride, 2);
    }

    #[test]
    fn test_bitmap_get_pixel_default() {
        let bitmap = Bitmap::new(8, 8);
        // The buffer initializes to all zeros.
        for y in 0..8 {
            for x in 0..8 {
                assert_eq!(bitmap.get_pixel(x, y), 0);
            }
        }
    }

    #[test]
    fn test_bitmap_set_get_pixel() {
        let mut bitmap = Bitmap::new(8, 8);

        bitmap.set_pixel(0, 0, 1);
        assert_eq!(bitmap.get_pixel(0, 0), 1);

        bitmap.set_pixel(7, 7, 1);
        assert_eq!(bitmap.get_pixel(7, 7), 1);

        bitmap.set_pixel(3, 3, 1);
        assert_eq!(bitmap.get_pixel(3, 3), 1);

        // Clearing a pixel should flip it back to zero.
        bitmap.set_pixel(3, 3, 0);
        assert_eq!(bitmap.get_pixel(3, 3), 0);
    }

    #[test]
    fn test_bitmap_out_of_bounds() {
        let mut bitmap = Bitmap::new(5, 5);

        // Out-of-range reads return zero.
        assert_eq!(bitmap.get_pixel(10, 10), 0);
        assert_eq!(bitmap.get_pixel(5, 5), 0);

        // Out-of-range writes are ignored.
        bitmap.set_pixel(10, 10, 1);
        bitmap.set_pixel(5, 5, 1);
    }

    #[test]
    fn test_bitmap_stride_calculation() {
        // Width 1 maps to a single byte row.
        let bm1 = Bitmap::new(1, 1);
        assert_eq!(bm1.stride, 1);

        // Width 8 still fits in one byte.
        let bm8 = Bitmap::new(8, 1);
        assert_eq!(bm8.stride, 1);

        // Width 9 rounds up to two bytes.
        let bm9 = Bitmap::new(9, 1);
        assert_eq!(bm9.stride, 2);

        // Width 16 stays at two bytes.
        let bm16 = Bitmap::new(16, 1);
        assert_eq!(bm16.stride, 2);
    }

    #[test]
    fn test_bitmap_combine_or() {
        let mut bm1 = Bitmap::new(8, 8);
        let mut bm2 = Bitmap::new(4, 4);

        // Seed bm1 with a couple of set pixels.
        bm1.set_pixel(0, 0, 1);
        bm1.set_pixel(1, 1, 1);

        // Seed bm2 with a different pattern.
        bm2.set_pixel(0, 0, 1);
        bm2.set_pixel(2, 2, 1);

        // OR should merge both patterns.
        bm1.combine(&bm2, 0, 0, 0); // operator 0 = OR

        // Both source and destination pixels should remain set.
        assert_eq!(bm1.get_pixel(0, 0), 1);
        assert_eq!(bm1.get_pixel(1, 1), 1);
        assert_eq!(bm1.get_pixel(2, 2), 1);
    }

    #[test]
    fn test_bitmap_combine_matches_naive_unaligned() {
        let mut dst_opt = Bitmap::new(23, 11);
        let mut dst_naive = dst_opt.clone();
        let mut src = Bitmap::new(11, 7);

        for y in 0..src.height {
            for x in 0..src.width {
                if (x + y) % 3 == 0 || (x * 2 + y) % 5 == 0 {
                    src.set_pixel(x, y, 1);
                }
            }
        }

        dst_opt.combine(&src, 3, 2, 0);
        combine_naive(&mut dst_naive, &src, 3, 2, 0);
        assert_eq!(dst_opt.data, dst_naive.data);

        let mut dst_opt = Bitmap::new(19, 9);
        let mut dst_naive = dst_opt.clone();
        dst_opt.combine(&src, -2, 1, 2);
        combine_naive(&mut dst_naive, &src, -2, 1, 2);
        assert_eq!(dst_opt.data, dst_naive.data);
    }

    #[test]
    fn test_bitmap_combine_and() {
        let mut bm1 = Bitmap::new(8, 8);
        let mut bm2 = Bitmap::new(8, 8);

        // Fill bm1 with ones to exercise AND behavior.
        for y in 0..8 {
            for x in 0..8 {
                bm1.set_pixel(x, y, 1);
            }
        }

        // Fill bm2 with ones as well.
        for y in 0..8 {
            for x in 0..8 {
                bm2.set_pixel(x, y, 1);
            }
        }

        // AND with all-ones should keep all pixels set.
        bm1.combine(&bm2, 0, 0, 1); // operator 1 = AND

        // Spot-check a few pixels for correctness.
        assert_eq!(bm1.get_pixel(0, 0), 1);
        assert_eq!(bm1.get_pixel(4, 4), 1);
        assert_eq!(bm1.get_pixel(7, 7), 1);
    }

    #[test]
    fn test_bitmap_combine_and_with_zeros() {
        let mut bm1 = Bitmap::new(8, 8);
        let bm2 = Bitmap::new(8, 8);

        // Fill bm1 with ones to ensure AND clears them.
        for y in 0..8 {
            for x in 0..8 {
                bm1.set_pixel(x, y, 1);
            }
        }

        // Verify bm2 starts at all zeros.
        for y in 0..8 {
            for x in 0..8 {
                assert_eq!(bm2.get_pixel(x, y), 0, "bm2({},{}) should be 0", x, y);
            }
        }

        // AND with zero should clear the destination.
        bm1.combine(&bm2, 0, 0, 1);

        // Check each pixel for zero after the combine.
        for y in 0..8 {
            for x in 0..8 {
                assert_eq!(
                    bm1.get_pixel(x, y),
                    0,
                    "After AND, bm1({},{}) should be 0",
                    x,
                    y
                );
            }
        }
    }

    #[test]
    fn test_bitmap_combine_replace() {
        let mut bm1 = Bitmap::new(8, 8);
        let bm2 = Bitmap::new(4, 4);

        // Fill bm1 with ones to make the replace visible.
        for y in 0..8 {
            for x in 0..8 {
                bm1.set_pixel(x, y, 1);
            }
        }

        // Combine with REPLACE
        bm1.combine(&bm2, 0, 0, 4); // operator 4 = REPLACE

        // The replaced region should now be zeros.
        for y in 0..4 {
            for x in 0..4 {
                assert_eq!(bm1.get_pixel(x, y), 0);
            }
        }

        // Outside the region should remain unchanged.
        assert_eq!(bm1.get_pixel(5, 5), 1);
    }

    #[test]
    fn test_bitmap_combine_offset() {
        let mut bm1 = Bitmap::new(10, 10);
        let mut bm2 = Bitmap::new(3, 3);

        // Set a single pixel in bm2 to test offsets.
        bm2.set_pixel(1, 1, 1);

        // Combine with an offset to move the source pixel.
        bm1.combine(&bm2, 2, 2, 0); // OR

        // The source pixel should land at the translated coordinate.
        assert_eq!(bm1.get_pixel(3, 3), 1);
        assert_eq!(bm1.get_pixel(1, 1), 0);
    }

    #[test]
    fn test_bitmap_combine_negative_offset() {
        let mut bm1 = Bitmap::new(10, 10);
        let mut bm2 = Bitmap::new(4, 4);

        // Fill bm2 so clipping behavior is visible.
        for y in 0..4 {
            for x in 0..4 {
                bm2.set_pixel(x, y, 1);
            }
        }

        // Combine with a negative offset so the source is clipped.
        bm1.combine(&bm2, -2, -2, 0);

        // Pixels that map into bm1 should be set.
        assert_eq!(bm1.get_pixel(0, 0), 1);
        assert_eq!(bm1.get_pixel(1, 1), 1);
    }

    #[test]
    fn test_bitmap_zero_dimensions() {
        let bm_zero_width = Bitmap::new(0, 10);
        assert_eq!(bm_zero_width.width, 0);
        assert_eq!(bm_zero_width.height, 10);

        let bm_zero_height = Bitmap::new(10, 0);
        assert_eq!(bm_zero_height.width, 10);
        assert_eq!(bm_zero_height.height, 0);
    }

    #[test]
    fn test_bitmap_clone() {
        let mut bm1 = Bitmap::new(5, 5);
        bm1.set_pixel(2, 2, 1);

        let bm2 = bm1.clone();
        assert_eq!(bm2.get_pixel(2, 2), 1);
        assert_eq!(bm2.width, 5);
        assert_eq!(bm2.height, 5);
    }

    // #[test]
    // #[should_panic(expected = "Bitmap dimensions unreasonable")]
    // fn test_bitmap_unreasonable_dimensions() {
    //     Bitmap::new(300_000_000, 1);
    // }
}
