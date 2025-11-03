#[derive(Clone)]
pub struct Bitmap {
    pub data: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub stride: usize, // bytes per row
}

impl Bitmap {
    pub fn new(width: usize, height: usize) -> Self {
        let stride = (width + 7) >> 3;
        let data = vec![0; stride * height];
        Bitmap { data, width, height, stride }
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

    /// Convert the bit-packed bitmap to an 8-bit grayscale image
    /// Returns a Vec<u8> where 0 = black (255 in image terms), 1 = white (0 in image terms)
    /// This matches the JS implementation's bit unpacking logic
    pub fn to_grayscale_image(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.width * self.height);
        for y in 0..self.height {
            let mut mask = 128u8;
            let mut buffer = 0u8;
            let mut buffer_index = 0;
            for x in 0..self.width {
                if mask == 0 || buffer_index == 0 {
                    mask = 128;
                    let byte_index = y * self.stride + (x >> 3);
                    buffer = if byte_index < self.data.len() { self.data[byte_index] } else { 0 };
                    buffer_index = 8;
                }
                let pixel = if (buffer & mask) != 0 { 0 } else { 255 }; // 0 = black, 255 = white
                result.push(pixel);
                mask >>= 1;
                buffer_index -= 1;
            }
        }
        result
    }
}