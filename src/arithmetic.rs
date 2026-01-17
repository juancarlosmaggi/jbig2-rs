//! Arithmetic decoder used by region and symbol decoders.

use crate::arithmetic_tables::QE_TABLE;
use crate::error::Jbig2Error;

/// MQ arithmetic decoder with internal interval and bit counter state.
pub struct ArithmeticDecoder {
    data: Vec<u8>,
    offset: usize,
    next_word: u32,
    next_word_bytes: usize,
    c: u32,
    ct: i32,
    a: u32,
}

impl ArithmeticDecoder {
    /// Create a new decoder initialized from the provided byte stream.
    pub fn new(data: &[u8]) -> Self {
        let mut decoder = ArithmeticDecoder {
            data: data.to_vec(),
            offset: 0,
            next_word: 0,
            next_word_bytes: 0,
            c: 0,
            ct: 0,
            a: 0x8000,
        };

        let (word, bytes) = decoder.get_next_word(0);
        decoder.next_word = word;
        decoder.next_word_bytes = bytes;
        if bytes == 0 {
            return decoder;
        }
        decoder.offset = bytes;

        // Initialize the C register from the first input byte.
        decoder.c = (!(decoder.next_word >> 8)) & 0xFF0000;

        decoder.byte_in();

        // Normalize after the initial byte-in.
        decoder.c <<= 7;
        decoder.ct -= 7;
        decoder.a = 0x8000;

        decoder
    }

    fn get_next_word(&self, offset: usize) -> (u32, usize) {
        if offset >= self.data.len() {
            return (0, 0);
        }

        let mut val = 0u32;
        let mut ret = 0usize;

        if offset < self.data.len() {
            val |= (self.data[offset] as u32) << 24;
            ret += 1;
        }
        if offset + 1 < self.data.len() {
            val |= (self.data[offset + 1] as u32) << 16;
            ret += 1;
        }
        if offset + 2 < self.data.len() {
            val |= (self.data[offset + 2] as u32) << 8;
            ret += 1;
        }
        if offset + 3 < self.data.len() {
            val |= self.data[offset + 3] as u32;
            ret += 1;
        }

        (val, ret)
    }

    /// Pull the next byte into the C register with marker/stuffing handling.
    fn byte_in(&mut self) {
        let b = ((self.next_word >> 24) & 0xFF) as u8;

        if b == 0xFF {
            if self.next_word_bytes <= 1 {
                let (word, bytes) = self.get_next_word(self.offset);
                if bytes == 0 {
                    self.next_word = 0xFF900000;
                    self.next_word_bytes = 2;
                    self.c = self.c.wrapping_add(0xFF00);
                    self.ct = 8;
                    return;
                }
                self.next_word = word;
                self.next_word_bytes = bytes;
                self.offset += bytes;

                let b1 = ((self.next_word >> 24) & 0xFF) as u8;
                if b1 > 0x8F {
                    self.ct = 8;
                    self.next_word = 0xFF000000 | (self.next_word >> 8);
                    self.next_word_bytes = 2;
                    if self.offset > 0 {
                        self.offset -= 1;
                    }
                } else {
                    self.c = self
                        .c
                        .wrapping_add(0xFE00u32.wrapping_sub((b1 as u32) << 9));
                    self.ct = 7;
                }
            } else {
                let b1 = ((self.next_word >> 16) & 0xFF) as u8;
                if b1 > 0x8F {
                    self.ct = 8;
                } else {
                    self.next_word_bytes -= 1;
                    self.next_word <<= 8;
                    self.c = self
                        .c
                        .wrapping_add(0xFE00u32.wrapping_sub((b1 as u32) << 9));
                    self.ct = 7;
                }
            }
        } else {
            self.next_word <<= 8;
            self.next_word_bytes = self.next_word_bytes.saturating_sub(1);

            if self.next_word_bytes == 0 {
                let (word, bytes) = self.get_next_word(self.offset);
                if bytes == 0 {
                    self.next_word = 0xFF900000;
                    self.next_word_bytes = 2;
                    self.c = self.c.wrapping_add(0xFF00);
                    self.ct = 8;
                    return;
                }
                self.next_word = word;
                self.next_word_bytes = bytes;
                self.offset += bytes;
            }

            let b = ((self.next_word >> 24) & 0xFF) as u8;
            self.c = self
                .c
                .wrapping_add(0xFF00u32.wrapping_sub((b as u32) << 8));
            self.ct = 8;
        }
    }

    /// Decode a single bit using the specified context state.
    #[inline(always)]
    pub fn read_bit(
        &mut self,
        contexts: &mut [i8],
        pos: usize,
    ) -> Result<u8, Jbig2Error> {
        if pos >= contexts.len() {
            return Err(Jbig2Error::new("invalid context position"));
        }

        let ctx_val = unsafe { *contexts.get_unchecked(pos) };
        let cx_index = (ctx_val >> 1) as usize;

        if cx_index >= QE_TABLE.len() {
            return Err(Jbig2Error::new("invalid context index"));
        }

        let qe_entry = unsafe { QE_TABLE.get_unchecked(cx_index) };
        let qe = qe_entry.qe as u32;
        let mut mps = (ctx_val & 1) as u8;

        // Shrink the interval by Qe.
        self.a = self.a.wrapping_sub(qe);

        let d: u8;
        let new_cx_index: usize;

        if (self.c >> 16) < self.a {
            // MPS path.
            if (self.a & 0x8000) != 0 {
                // No renormalization needed; keep context unchanged.
                return Ok(mps);
            }
            if self.a < qe {
                d = 1 ^ mps;
                if qe_entry.switch_flag == 1 {
                    mps = d;
                }
                new_cx_index = qe_entry.nlps as usize;
            } else {
                d = mps;
                new_cx_index = qe_entry.nmps as usize;
            }
        } else {
            // LPS path; subtract A from C (A is shifted left 16 bits).
            self.c = self.c.wrapping_sub(self.a << 16);

            if self.a < qe {
                self.a = qe;
                d = mps;
                new_cx_index = qe_entry.nmps as usize;
            } else {
                self.a = qe;
                d = 1 ^ mps;
                if qe_entry.switch_flag == 1 {
                    mps = d;
                }
                new_cx_index = qe_entry.nlps as usize;
            }
        }

        // Renormalize the interval and code registers.
        loop {
            if self.ct == 0 {
                self.byte_in();
            }
            self.a <<= 1;
            self.c <<= 1;
            self.ct -= 1;
            if (self.a & 0x8000) != 0 {
                break;
            }
        }

        // Update the context state for the next symbol.
        unsafe {
            *contexts.get_unchecked_mut(pos) = ((new_cx_index as i8) << 1) | (mps as i8);
        }

        Ok(d)
    }

    /// Return the number of bytes consumed from the input stream.
    pub fn get_bytes_read(&self) -> usize {
        self.offset.saturating_sub(self.next_word_bytes)
    }
}
