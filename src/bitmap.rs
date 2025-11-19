#[derive(Clone)]
pub struct Bitmap {
    pub data: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub stride: usize, // bytes per row
}

impl Bitmap {
    pub fn new(width: usize, height: usize) -> Self {
        // Sanity check dimensions before any arithmetic
        if width > 100_000 || height > 100_000 {
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
