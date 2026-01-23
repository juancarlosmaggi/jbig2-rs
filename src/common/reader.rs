use std::borrow::Cow;

/// Bit/byte reader with a movable window over a data buffer.
#[derive(Clone)]
pub struct Reader<'a> {
    data: Cow<'a, [u8]>,
    end: usize,
    position: usize,
    shift: i32,
    current_byte: u8,
}

impl<'a> Reader<'a> {
    /// Create a reader over `data[start..end]`.
    pub fn new<D>(data: D, start: usize, end: usize) -> Self
    where
        D: Into<Cow<'a, [u8]>>,
    {
        Reader {
            data: data.into(),
            end,
            position: start,
            shift: -1,
            current_byte: 0,
        }
    }

    /// Read the next bit, advancing the internal position.
    pub fn read_bit(&mut self) -> Result<u8, crate::common::error::Jbig2Error> {
        if self.shift < 0 {
            if self.position >= self.end {
                return Err(crate::common::error::Jbig2Error::new(
                    "end of data while reading bit",
                ));
            }
            self.current_byte = self.data[self.position];
            self.position += 1;
            self.shift = 7;
        }
        let bit = (self.current_byte >> self.shift as u32) & 1;
        self.shift -= 1;
        Ok(bit)
    }

    /// Read a multi-bit value MSB-first from the stream.
    pub fn read_bits(&mut self, num_bits: u32) -> Result<u32, crate::common::error::Jbig2Error> {
        let mut result = 0;
        for i in (0..num_bits).rev() {
            result |= (self.read_bit()? as u32) << i;
        }
        Ok(result)
    }

    /// Align the reader to the next byte boundary.
    pub fn byte_align(&mut self) {
        self.shift = -1;
    }

    /// Return the underlying buffer.
    pub fn get_data(&self) -> &[u8] {
        self.data.as_ref()
    }

    /// Return the current byte position.
    pub fn get_position(&self) -> usize {
        self.position
    }

    /// Set the current byte position.
    pub fn set_position(&mut self, pos: usize) {
        self.position = pos;
    }

    /// Set the current bit shift within the buffered byte.
    pub fn set_shift(&mut self, shift: i32) {
        self.shift = shift;
    }

    /// Return the current bit shift within the buffered byte.
    pub fn get_shift(&self) -> i32 {
        self.shift
    }

    /// Return the last buffered byte.
    pub fn get_current_byte(&self) -> u8 {
        self.current_byte
    }

    /// Set the buffered byte without advancing the position.
    pub fn set_current_byte(&mut self, byte: u8) {
        self.current_byte = byte;
    }

    /// Return the byte limit for this reader.
    pub fn get_end(&self) -> usize {
        self.end
    }

    /// Read the next byte and advance the position.
    pub fn read_byte(&mut self) -> Option<u8> {
        if self.position >= self.end {
            return None;
        }
        let b = self.data[self.position];
        self.position += 1;
        Some(b)
    }

    /// Advance the position by `amount` bytes.
    pub fn skip(&mut self, amount: usize) {
        self.position += amount;
    }

    /// Limit reads to `limit` bytes from the current position.
    pub fn set_limit(&mut self, limit: usize) {
        self.end = self.position + limit;
        if self.end > self.data.len() {
            self.end = self.data.len();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reader_creation() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let reader = Reader::new(data.clone(), 0, 4);
        assert_eq!(reader.get_position(), 0);
        assert_eq!(reader.get_end(), 4);
    }

    #[test]
    fn test_read_byte() {
        let data = vec![0xAB, 0xCD, 0xEF];
        let mut reader = Reader::new(data, 0, 3);

        assert_eq!(reader.read_byte(), Some(0xAB));
        assert_eq!(reader.read_byte(), Some(0xCD));
        assert_eq!(reader.read_byte(), Some(0xEF));
        assert_eq!(reader.read_byte(), None);
    }

    #[test]
    fn test_read_bit() {
        let data = vec![0b10110100];
        let mut reader = Reader::new(data, 0, 1);

        assert_eq!(reader.read_bit().unwrap(), 1);
        assert_eq!(reader.read_bit().unwrap(), 0);
        assert_eq!(reader.read_bit().unwrap(), 1);
        assert_eq!(reader.read_bit().unwrap(), 1);
        assert_eq!(reader.read_bit().unwrap(), 0);
        assert_eq!(reader.read_bit().unwrap(), 1);
        assert_eq!(reader.read_bit().unwrap(), 0);
        assert_eq!(reader.read_bit().unwrap(), 0);

        assert!(reader.read_bit().is_err());
    }

    #[test]
    fn test_read_bits_multiple() {
        let data = vec![0xFF, 0x00];
        let mut reader = Reader::new(data, 0, 2);

        assert_eq!(reader.read_bits(4).unwrap(), 0b1111);
        assert_eq!(reader.read_bits(4).unwrap(), 0b1111);
        assert_eq!(reader.read_bits(8).unwrap(), 0b00000000);
    }

    #[test]
    fn test_byte_align() {
        let data = vec![0b10110100, 0b11001100];
        let mut reader = Reader::new(data, 0, 2);

        assert_eq!(reader.read_bit().unwrap(), 1);
        assert_eq!(reader.read_bit().unwrap(), 0);
        assert_eq!(reader.read_bit().unwrap(), 1);

        reader.byte_align();

        assert_eq!(reader.read_bit().unwrap(), 1);
        assert_eq!(reader.read_bit().unwrap(), 1);
    }

    #[test]
    fn test_position_management() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        let mut reader = Reader::new(data, 0, 5);

        assert_eq!(reader.get_position(), 0);

        reader.read_byte();
        assert_eq!(reader.get_position(), 1);

        reader.set_position(3);
        assert_eq!(reader.get_position(), 3);
        assert_eq!(reader.read_byte(), Some(0x04));
    }

    #[test]
    fn test_skip() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        let mut reader = Reader::new(data, 0, 5);

        reader.skip(2);
        assert_eq!(reader.get_position(), 2);
        assert_eq!(reader.read_byte(), Some(0x03));
    }

    #[test]
    fn test_set_limit() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        let mut reader = Reader::new(data, 0, 5);

        reader.set_limit(2);
        assert_eq!(reader.get_end(), 2);

        assert_eq!(reader.read_byte(), Some(0x01));
        assert_eq!(reader.read_byte(), Some(0x02));
        assert_eq!(reader.read_byte(), None);
    }

    #[test]
    fn test_read_bits_eof() {
        let data = vec![0xFF];
        let mut reader = Reader::new(data, 0, 1);

        let result = reader.read_bits(16);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_reader() {
        let data = vec![];
        let mut reader = Reader::new(data, 0, 0);

        assert_eq!(reader.read_byte(), None);
        assert!(reader.read_bit().is_err());
    }
}
