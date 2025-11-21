//! Arithmetic Decoder Module
//!
//! This module implements the MQ arithmetic decoder used in JBIG2, as specified in
//! ITU-T T.88 and T.82. It provides context-based adaptive binary arithmetic decoding.
//!
//! # Overview
//!
//! The arithmetic decoder maintains an internal state (A, C, CT) and updates probability
//! estimates (contexts) based on decoded bits. It handles:
//!
//! - Byte stream consumption (with 0xFF stuffing handling)
//! - Probability estimation using the QE table
//! - Renormalization of the A and C registers
//! - Conditional exchange of MPS (More Probable Symbol) and LPS (Less Probable Symbol)

// Import QE table from separate module
use crate::arithmetic_tables::QE_TABLE;

/// MQ Arithmetic Decoder implementation.
///
/// Maintains the state of the arithmetic decoding process, including the current
/// interval (A), code register (C), and bit counter (CT).
pub struct ArithmeticDecoder {
    data: Vec<u8>,
    offset: usize, // Position in data array
    data_end: usize,
    next_word: u32,         // Buffer containing up to 4 bytes
    next_word_bytes: usize, // Number of valid bytes in next_word
    chigh: u32,
    clow: u32,
    ct: i32,
    a: u32,
}

impl ArithmeticDecoder {
    /// Creates a new arithmetic decoder instance.
    ///
    /// Initializes the decoder state (A, C, CT) and pre-fills the buffer from the input data.
    ///
    /// # Arguments
    ///
    /// * `data` - The byte slice containing the arithmetic coded data stream
    pub fn new(data: &[u8]) -> Self {
        let mut decoder = ArithmeticDecoder {
            data: data.to_vec(),
            offset: 0,
            data_end: data.len(),
            next_word: 0,
            next_word_bytes: 0,
            chigh: 0,
            clow: 0,
            ct: 0,
            a: 0,
        };

        // Read first 4 bytes into buffer (BIG-ENDIAN)
        if data.len() >= 4 {
            decoder.next_word = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
            decoder.next_word_bytes = 4;
            decoder.offset = 4;
        } else {
            // Handle short data
            let mut bytes = [0u8; 4];
            for (i, &b) in data.iter().enumerate() {
                bytes[i] = b;
            }
            decoder.next_word = u32::from_be_bytes(bytes);
            decoder.next_word_bytes = data.len();
            decoder.offset = data.len();
        }

        // Initialize C: C = (~(next_word >> 8)) & 0xFF0000
        let c = (!(decoder.next_word >> 8)) & 0xFF0000;
        decoder.chigh = (c >> 16) & 0xFFFF;
        decoder.clow = c & 0xFFFF;

        // Call byte_in (operates on buffer!)
        decoder.byte_in();

        // Finalize: C <<= 7, CT -= 7, A = 0x8000
        let c = (decoder.chigh << 16) | decoder.clow;
        let c = c << 7;
        decoder.chigh = (c >> 16) & 0xFFFF;
        decoder.clow = c & 0xFFFF;
        decoder.ct -= 7;
        decoder.a = 0x8000;

        decoder
    }

    fn byte_in(&mut self) {
        // CRITICAL: This operates on the buffered next_word, NOT on data[offset]!

        // Get current top byte from buffer
        let b_check = ((self.next_word >> 24) & 0xFF) as u8;

        if b_check == 0xFF {
            // Special 0xFF handling
            // Shift buffer
            self.next_word <<= 8;
            self.next_word_bytes = self.next_word_bytes.saturating_sub(1);

            // Refill buffer if needed
            if self.next_word_bytes == 0 {
                self.refill_buffer();
            }

            // Get next byte from buffer
            let b1 = ((self.next_word >> 24) & 0xFF) as u8;

            if b1 > 0x8F {
                // Marker byte stuffing
                self.clow = self.clow.wrapping_add(0xFF00);
                self.ct = 8;
                return;
            }

            // Normal 0xFF processing
            self.clow = self.clow.wrapping_add(0xFF00 | (b1 as u32));
            self.ct = 7;
        } else {
            // Normal byte
            self.next_word <<= 8;
            self.next_word_bytes = self.next_word_bytes.saturating_sub(1);

            // Refill buffer if exhausted
            if self.next_word_bytes == 0 {
                self.refill_buffer();
            }

            // Get NEW top byte from buffer (after shift!)
            let b = ((self.next_word >> 24) & 0xFF) as u8;

            // Update C
            let full_c = (self.chigh << 16) | self.clow;
            let full_c = full_c.wrapping_add(0xFF00 - ((b as u32) << 8));
            self.chigh = full_c >> 16 & 0xFFFF;
            self.clow = full_c & 0xFFFF;
            self.ct = 8;
        }

        // Handle overflow from clow to chigh
        if self.clow > 0xFFFF {
            self.chigh = self.chigh.wrapping_add(self.clow >> 16);
            self.clow &= 0xFFFF;
        }
    }

    fn refill_buffer(&mut self) {
        // Read up to 4 bytes from data into next_word
        let mut bytes_read = 0;
        let mut new_word = 0u32;

        for _i in 0..4 {
            if self.offset < self.data_end {
                new_word = (new_word << 8) | (self.data[self.offset] as u32);
                self.offset += 1;
                bytes_read += 1;
            } else {
                new_word <<= 8; // Pad with zeros
            }
        }

        self.next_word = new_word;
        self.next_word_bytes = bytes_read;
    }

    /// Decodes a single bit using the specified context.
    ///
    /// This is the core decoding function. It uses the probability estimate associated
    /// with the given context to decode the next bit and updates the context state.
    ///
    /// # Arguments
    ///
    /// * `contexts` - Mutable slice of context states (indices into QE table)
    /// * `pos` - Index of the context to use for this bit
    ///
    /// # Returns
    ///
    /// - `Ok(u8)` - The decoded bit (0 or 1)
    /// - `Err(Jbig2Error)` - If context index is invalid
    pub fn read_bit(
        &mut self,
        contexts: &mut [i8],
        pos: usize,
    ) -> Result<u8, crate::error::Jbig2Error> {
        if pos >= contexts.len() {
            return Err(crate::error::Jbig2Error::new("invalid context position"));
        }
        let cx_index = (contexts[pos] >> 1) as usize;
        if cx_index >= QE_TABLE.len() {
            return Err(crate::error::Jbig2Error::new("invalid context index"));
        }
        let mut cx_mps = (contexts[pos] & 1) as u8;
        let qe_entry = &QE_TABLE[cx_index];
        let qe_icx = qe_entry.qe;
        let d: u8;
        let new_cx_index: usize;

        // Figure F.2: Subtract Qe from A
        self.a = self.a.wrapping_sub(qe_icx as u32);

        // Figure F.2: Compare C (top 16 bits) to updated A
        if self.chigh < self.a {
            // MPS path (C < A)
            if (self.a & 0x8000) == 0 {
                // Need renormalization
                // MPS_EXCHANGE (Figure E.16)
                if self.a < qe_icx as u32 {
                    d = 1 ^ cx_mps; // Return LPS
                    if qe_entry.switch_flag == 1 {
                        cx_mps = d;
                    }
                    new_cx_index = qe_entry.nlps as usize;
                } else {
                    d = cx_mps; // Return MPS
                    new_cx_index = qe_entry.nmps as usize;
                }
                // renormD will follow below
            } else {
                // Don't need renormalization - fast path
                // Update context and return MPS immediately
                contexts[pos] = ((qe_entry.nmps as i8) << 1) | (cx_mps as i8);
                return Ok(cx_mps);
            }
        } else {
            // LPS path (C >= A)
            // Subtract A from C FIRST
            let c_full = (self.chigh << 16) | self.clow;
            let c_full = c_full.wrapping_sub(self.a << 16);
            self.chigh = (c_full >> 16) & 0xFFFF;
            self.clow = c_full & 0xFFFF;

            // LPS_EXCHANGE (Figure E.17)
            if self.a < qe_icx as u32 {
                self.a = qe_icx as u32;
                d = cx_mps; // Return MPS (even though we're in LPS path!)
                new_cx_index = qe_entry.nmps as usize;
            } else {
                self.a = qe_icx as u32;
                d = 1 ^ cx_mps; // Return LPS
                if qe_entry.switch_flag == 1 {
                    cx_mps = d;
                }
                new_cx_index = qe_entry.nlps as usize;
            }
            // renormD will follow below
        }

        // renormD (Figure E.18) - only reached if renormalization needed
        let mut loop_count = 0;
        loop {
            if self.ct == 0 {
                self.byte_in();
            }
            self.a <<= 1;
            self.chigh = ((self.chigh << 1) & 0xffff) | ((self.clow >> 15) & 1);
            self.clow = (self.clow << 1) & 0xffff;
            self.ct -= 1;
            if (self.a & 0x8000) != 0 {
                break;
            }
            
            // TODO: Safety check - prevents infinite loop if A becomes 0
            // Consider returning proper error instead of force-fixing state
            if self.a == 0 {
                 eprintln!("CRITICAL ERROR: ArithmeticDecoder A became 0. Breaking infinite loop.");
                 self.a = 0x8000; // Force valid state
                 break;
            }

            loop_count += 1;
            // TODO: Safety check - prevents stuck renormalization loop
            // Consider returning proper error instead of force-fixing state
            if loop_count > 100 {
                 eprintln!("CRITICAL ERROR: ArithmeticDecoder renormD stuck. A={:x}", self.a);
                 self.a = 0x8000;
                 break;
            }
        }

        // Update context
        contexts[pos] = ((new_cx_index as i8) << 1) | (cx_mps as i8);
        Ok(d)
    }
}
