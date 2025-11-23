//! Arithmetic Decoder Module
//!
//! This module implements the MQ arithmetic decoder used in JBIG2, as specified in
//! ITU-T T.88 Annex E (with decoder modifications as per clause 6 and jbig2dec reference behavior).
//!
//! NOTE: We initialise A = 0x10000 (instead of the spec’s 0x8000 for generic contexts)
//! because the same decoder is used for both generic (GB) and refinement (GR) contexts.
//! All reference implementations do this; the probability estimation tables adapt correctly.

use crate::arithmetic_tables::QE_TABLE;

/// MQ Arithmetic Decoder implementation.
///
/// C register is represented as chigh (bits 31–16) and clow (bits 15–0).
/// A is the interval register (effectively 16-bit value).
/// CT is the bit counter (number of bits until next BYTEIN).
pub struct ArithmeticDecoder {
    data: Vec<u8>,
    offset: usize,
    data_end: usize,
    next_word: u32,
    next_word_bytes: usize,
    chigh: u32,
    clow: u32,
    ct: i32,
    a: u32,
}

impl ArithmeticDecoder {
    /// Creates a new arithmetic decoder instance.
    ///
    /// Initialises A = 0x10000, C = 0, then consumes the first two bytes
    /// (with stuffing handling) and sets CT = 12.
    pub fn new(data: &[u8]) -> Self {
        let mut decoder = ArithmeticDecoder {
            data: data.to_vec(),
            offset: 0,
            data_end: data.len(),
            next_word: 0,
            next_word_bytes: 0,
            chigh: 0,
            clow: 0,
            ct: 11, // will be adjusted to 12 after two BYTEIN calls
            a: 0x10000,
        };

        decoder.refill_buffer();
        decoder.byte_in();
        decoder.byte_in();
        decoder.ct = 12;

        decoder
    }

    /// Refill the 32-bit input buffer, padding with 0xFF when data is exhausted
    /// (this matches jbig2dec behavior for end-of-stream handling).
    fn refill_buffer(&mut self) {
        let mut new_word = 0u32;
        let mut bytes_read = 0;

        for _ in 0..4 {
            let byte = if self.offset < self.data_end {
                let b = self.data[self.offset];
                self.offset += 1;
                b as u32
            } else {
                0xFF
            };
            new_word = (new_word << 8) | byte;
            bytes_read += 1;
        }

        self.next_word = new_word;
        self.next_word_bytes = bytes_read;
    }

    /// Input a byte into the C register, handling 0xFF stuffing correctly.
    fn byte_in(&mut self) {
        let b = ((self.next_word >> 24) & 0xFF) as u8;

        // Always consume the current top byte
        self.next_word <<= 8;
        self.next_word_bytes = self.next_word_bytes.saturating_sub(1);
        if self.next_word_bytes == 0 {
            self.refill_buffer();
        }

        let add: u32;

        if b == 0xFF {
            let b1 = ((self.next_word >> 24) & 0xFF) as u8;
            if b1 > 0x8F {
                add = 0xFF00;
                // Marker – do not consume next byte
            } else {
                add = 0xFE00;
                // Stuffed – consume the next (stuffer) byte
                self.next_word <<= 8;
                self.next_word_bytes = self.next_word_bytes.saturating_sub(1);
                if self.next_word_bytes == 0 {
                    self.refill_buffer();
                }
            }
            self.ct = 8;
        } else {
            add = (b as u32) << 8;
            self.ct = 8;
        }

        let c = ((self.chigh as u64) << 16) | self.clow as u64;
        let c = c + add as u64;
        self.chigh = (c >> 16) as u32;
        self.clow = c as u32 & 0xFFFF;
    }

    /// Decodes a single bit using the specified context.
    ///
    /// Follows the DECODE procedure from Annex E Figure E.15 (with MPS/LPS exchange
    /// and renormalization as per Figures E.16–E.18).
    #[inline(always)]
    pub fn read_bit(
        &mut self,
        contexts: &mut [i8],
        pos: usize,
    ) -> Result<u8, crate::error::Jbig2Error> {
        if pos >= contexts.len() {
            return Err(crate::error::Jbig2Error::new("invalid context position"));
        }

        let ctx_val = unsafe { *contexts.get_unchecked(pos) };
        let cx_index = (ctx_val >> 1) as usize;

        if cx_index >= QE_TABLE.len() {
            return Err(crate::error::Jbig2Error::new("invalid context index"));
        }

        let qe_entry = unsafe { QE_TABLE.get_unchecked(cx_index) };
        let qe = qe_entry.qe as u32;
        let mut mps = (ctx_val & 1) as u8;

        // A -= Qe
        self.a = self.a.wrapping_sub(qe);

        let d: u8;
        let new_cx_index: usize;

        if self.chigh < self.a {
            // MPS path
            if (self.a & 0x8000) != 0 {
                // No renormalization needed – fast path
                unsafe {
                    *contexts.get_unchecked_mut(pos) = ((qe_entry.nmps as i8) << 1) | (mps as i8);
                }
                return Ok(mps);
            }
            // Renormalization needed
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
            // LPS path – subtract A from C (A is shifted left 16 bits)
            let c = ((self.chigh as u64) << 16) | self.clow as u64;
            let c = c.wrapping_sub((self.a as u64) << 16);
            self.chigh = (c >> 16) as u32;
            self.clow = c as u32 & 0xFFFF;

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

        // Renormalization (Figure E.18)
        loop {
            if self.ct == 0 {
                self.byte_in();
            }
            self.a <<= 1;
            let carry = self.clow >> 15 & 1;
            self.clow = (self.clow << 1) & 0xFFFF;
            self.chigh = ((self.chigh << 1) | carry) & 0xFFFF;
            self.ct -= 1;
            if (self.a & 0x8000) != 0 {
                break;
            }
        }

        // Update context
        unsafe {
            *contexts.get_unchecked_mut(pos) = ((new_cx_index as i8) << 1) | (mps as i8);
        }

        Ok(d)
    }

    /// Returns the number of bytes consumed from the input stream.
    pub fn get_bytes_read(&self) -> usize {
        self.offset - self.next_word_bytes
    }
}
