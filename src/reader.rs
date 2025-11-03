pub struct Reader {
    data: Vec<u8>,
    end: usize,
    position: usize,
    shift: i32,
    current_byte: u8,
}

impl Reader {
    pub fn new(data: Vec<u8>, start: usize, end: usize) -> Self {
        Reader {
            data,
            end,
            position: start,
            shift: -1,
            current_byte: 0,
        }
    }

    pub fn read_bit(&mut self) -> Result<u8, crate::error::Jbig2Error> {
        if self.shift < 0 {
            if self.position >= self.end {
                return Err(crate::error::Jbig2Error::new("end of data while reading bit"));
            }
            self.current_byte = self.data[self.position];
            self.position += 1;
            self.shift = 7;
        }
        let bit = (self.current_byte >> self.shift as u32) & 1;
        self.shift -= 1;
        Ok(bit)
    }

    pub fn read_bits(&mut self, num_bits: u32) -> Result<u32, crate::error::Jbig2Error> {
        let mut result = 0;
        for i in (0..num_bits).rev() {
            result |= (self.read_bit()? as u32) << i;
        }
        Ok(result)
    }

    pub fn byte_align(&mut self) {
        self.shift = -1;
    }

    pub fn read_byte(&mut self) -> i32 {
        if self.position >= self.end {
            return -1;
        }
        let byte = self.data[self.position];
        self.position += 1;
        byte as i32
    }
}