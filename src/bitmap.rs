//! Bitmap Module
//!
//! This module provides the [`Bitmap`] struct for storing and manipulating 1-bit monochrome images.
//! It handles memory management, pixel access, and coordinate bounds checking.

/// Represents a 2D bitmap image with 1 bit per pixel.
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
    /// Creates a new bitmap with the specified dimensions.
    ///
    /// Initializes all pixels to 0 (white/background).
    ///
    /// # Arguments
    ///
    /// * `width` - Width in pixels
    /// * `height` - Height in pixels
    ///
    /// # Panics
    ///
    /// Panics if dimensions are unreasonably large (> 200,000,000) or if memory allocation fails.
    pub fn new(width: usize, height: usize) -> Self {
        // Sanity check dimensions before any arithmetic
        if width > 200_000_000 || height > 200_000_000 {
            panic!("Bitmap dimensions unreasonable: {}x{} (likely decode error)", width, height);
        }
        
        // Use checked arithmetic to prevent overflow
        let stride = width.checked_add(7)
            .expect("width too large for stride calculation")
            >> 3;
        
        let buffer_size = stride.checked_mul(height)
            .expect("buffer size overflow");
        
        let data = vec![0; buffer_size];
        
        Bitmap {
            data,
            width,
            height,
            stride,
        }
    }

    /// Gets the pixel value at the specified coordinates.
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

    /// Sets the pixel value at the specified coordinates.
    ///
    /// Silently ignores out-of-bounds coordinates.
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

    /// Combines another bitmap into this one at the specified coordinates using the given operator.
    ///
    /// This method is optimized to use byte-level operations where possible.
    ///
    /// # Arguments
    ///
    /// * `other` - The source bitmap to combine
    /// * `x` - X coordinate in this bitmap where the source should be placed
    /// * `y` - Y coordinate in this bitmap where the source should be placed
    /// * `operator` - Combination operator (0=OR, 1=AND, 2=XOR, 3=XNOR, 4=REPLACE)
    pub fn combine(&mut self, other: &Bitmap, x: isize, y: isize, operator: u8) {
        // Clip to bounds
        let start_y = y.max(0) as usize;
        let end_y = (y + other.height as isize).min(self.height as isize) as usize;
        
        if start_y >= end_y {
            return;
        }

        let start_x = x.max(0) as usize;
        let end_x = (x + other.width as isize).min(self.width as isize) as usize;

        if start_x >= end_x {
            return;
        }

        // Calculate source offsets
        let src_start_y = (start_y as isize - y) as usize;
        let src_start_x = (start_x as isize - x) as usize;
        let _width = end_x - start_x;

        // Optimization: Process byte-by-byte
        for i in 0..(end_y - start_y) {
            let dst_y = start_y + i;
            let src_y = src_start_y + i;
            
            let dst_row_start = dst_y * self.stride;
            let src_row_start = src_y * other.stride;

            // Bit offsets
            let dst_bit_offset = start_x & 7;
            let src_bit_offset = src_start_x & 7;
            
            // Shift amount to align src to dst
            // + means src needs to be shifted right (or dst left)
            // We want to align MSB (bit 7).
            // If dst_bit_offset = 0 (MSB), src_bit_offset = 1.
            // src byte: .X......
            // dst byte: X.......
            // We need to shift src LEFT by 1.
            let shift = src_bit_offset as i8 - dst_bit_offset as i8;

            let mut current_x = start_x;
            let mut current_src_x = src_start_x;
            
            while current_x < end_x {
                let dst_byte_idx = dst_row_start + (current_x >> 3);
                let bits_left_in_byte = 8 - (current_x & 7);
                let bits_to_process = bits_left_in_byte.min(end_x - current_x);
                
                // Construct source byte aligned to dest
                let src_byte_idx = src_row_start + (current_src_x >> 3);
                let mut src_byte = other.data[src_byte_idx];
                
                // Handle shift
                if shift > 0 {
                    // src is "later" in the byte, need to shift LEFT to match dst
                    // e.g. src=1, dst=0. shift=1. src=0x40. dst=0x80. 0x40 << 1 = 0x80.
                    src_byte <<= shift;
                    // Need bits from next byte?
                    // If we are processing N bits, and src_bit_offset + N > 8, we need next byte.
                    // Actually, easier: just grab next byte if needed.
                    if (current_src_x & 7) + bits_to_process > 8 && src_byte_idx + 1 < other.data.len() {
                        let next_byte = other.data[src_byte_idx + 1];
                        src_byte |= next_byte >> (8 - shift);
                    }
                } else if shift < 0 {
                    // src is "earlier", need to shift RIGHT
                    // e.g. src=0, dst=1. shift=-1. src=0x80. dst=0x40. 0x80 >> 1 = 0x40.
                    src_byte >>= -shift;
                     if (current_src_x & 7) + bits_to_process > 8 && src_byte_idx + 1 < other.data.len() {
                         let next_byte = other.data[src_byte_idx + 1];
                         src_byte |= next_byte << (8 + shift);
                    }
                }
                
                // Create mask for the bits we are processing
                // e.g. bits_to_process=3, dst_bit_offset=0 -> 11100000
                // e.g. bits_to_process=3, dst_bit_offset=2 -> 00111000
                let mask_high = 0xFFu8 >> (current_x & 7);
                let shift_low = (current_x & 7) + bits_to_process;
                let mask_low = if shift_low == 8 { 0 } else { !(0xFFu8 >> shift_low) };
                let mask = mask_high & mask_low;

                let dst_byte = self.data[dst_byte_idx];
                let mut new_byte = dst_byte;

                match operator {
                    0 => new_byte |= src_byte & mask,        // OR
                    1 => new_byte &= src_byte | !mask,     // AND (preserve bits outside mask)
                    2 => new_byte ^= src_byte & mask,        // XOR
                    3 => { // XNOR
                         let xor = dst_byte ^ src_byte;
                         new_byte = (new_byte & !mask) | (!xor & mask);
                    },
                    4 => { // REPLACE
                        new_byte = (new_byte & !mask) | (src_byte & mask);
                    },
                    _ => {}
                }

                self.data[dst_byte_idx] = new_byte;

                current_x += bits_to_process;
                current_src_x += bits_to_process;
            }
        }
    }
}
