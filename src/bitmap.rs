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
        let bit_index = 7 - (x & 7);
        (self.data[byte_index] >> bit_index) & 1
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, value: u8) {
        let byte_index = y * self.stride + (x >> 3);
        let bit_index = 7 - (x & 7);
        if value != 0 {
            self.data[byte_index] |= 1 << bit_index;
        } else {
            self.data[byte_index] &= !(1 << bit_index);
        }
    }
}