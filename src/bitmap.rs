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
}
