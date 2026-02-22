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
        let data = data.into();
        let end = end.min(data.len());
        Reader {
            data,
            end,
            position: start,
            shift: -1,
            current_byte: 0,
        }
    }

    /// Read the next bit, advancing the internal position.
    #[inline]
    pub fn read_bit(&mut self) -> Result<u8, crate::common::error::Jbig2Error> {
        if self.shift < 0 {
            if self.position >= self.end {
                return Err(crate::common::error::Jbig2Error::new(
                    "end of data while reading bit",
                ));
            }
            // SAFETY: self.end is clamped to self.data.len() in new() and set_limit().
            // Thus, position < end implies position < data.len().
            self.current_byte = unsafe { *self.data.get_unchecked(self.position) };
            self.position += 1;
            self.shift = 7;
        }
        let bit = (self.current_byte >> self.shift as u32) & 1;
        self.shift -= 1;
        Ok(bit)
    }

    /// Read a multi-bit value MSB-first from the stream.
    #[inline]
    pub fn read_bits(
        &mut self,
        mut num_bits: u32,
    ) -> Result<u32, crate::common::error::Jbig2Error> {
        if num_bits == 0 {
            return Ok(0);
        }

        // Ensure we have a valid current byte if needed
        if self.shift < 0 {
            if self.position >= self.end {
                return Err(crate::common::error::Jbig2Error::new(
                    "end of data while reading bits",
                ));
            }
            self.current_byte = self.data[self.position];
            self.position += 1;
            self.shift = 7;
        }

        let available = (self.shift + 1) as u32;

        if num_bits <= available {
            // All bits are in the current byte
            let shift_after = self.shift - num_bits as i32;
            let result = (self.current_byte as u32 >> (shift_after + 1)) & ((1 << num_bits) - 1);
            self.shift = shift_after;
            return Ok(result);
        }

        // Take all available bits from current byte
        let mut result = (self.current_byte as u32) & ((1 << available) - 1);
        num_bits -= available;
        self.shift = -1; // Current byte exhausted

        // Read full bytes
        while num_bits >= 8 {
            if self.position >= self.end {
                return Err(crate::common::error::Jbig2Error::new(
                    "end of data while reading bits",
                ));
            }
            // SAFETY: position < end <= data.len()
            let byte = unsafe { *self.data.get_unchecked(self.position) };
            self.position += 1;
            result = (result << 8) | (byte as u32);
            num_bits -= 8;
        }

        // Read remaining bits from a new byte
        if num_bits > 0 {
            if self.position >= self.end {
                return Err(crate::common::error::Jbig2Error::new(
                    "end of data while reading bits",
                ));
            }
            // SAFETY: position < end <= data.len()
            self.current_byte = unsafe { *self.data.get_unchecked(self.position) };
            self.position += 1;
            // Take top `num_bits`
            let shift_after = 7 - num_bits as i32;
            let chunk = self.current_byte as u32 >> (shift_after + 1);
            result = (result << num_bits) | chunk;
            self.shift = shift_after;
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
        // SAFETY: position < end <= data.len()
        let b = unsafe { *self.data.get_unchecked(self.position) };
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
