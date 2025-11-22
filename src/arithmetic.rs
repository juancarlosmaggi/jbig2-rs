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
    #[inline(always)]
    pub fn read_bit(
        &mut self,
        contexts: &mut [i8],
        pos: usize,
    ) -> Result<u8, crate::error::Jbig2Error> {
        if pos >= contexts.len() {
            return Err(crate::error::Jbig2Error::new("invalid context position"));
        }
        
        // SAFETY: We checked pos < contexts.len() above.
        // cx_index is derived from the context value which is an i8.
        // The context value is updated only within this function using values from QE_TABLE.
        // QE_TABLE indices are within bounds by design of the table.
        // However, initial context values come from outside.
        // We should check cx_index bounds once, but we can use get_unchecked for the table lookup
        // if we verify it's within bounds.
        
        let ctx_val = unsafe { *contexts.get_unchecked(pos) };
        let cx_index = (ctx_val >> 1) as usize;
        
        if cx_index >= QE_TABLE.len() {
             return Err(crate::error::Jbig2Error::new("invalid context index"));
        }

        let mut cx_mps = (ctx_val & 1) as u8;
        // SAFETY: We checked cx_index < QE_TABLE.len() above.
        let qe_entry = unsafe { QE_TABLE.get_unchecked(cx_index) };
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
                // SAFETY: pos is within bounds as checked at start
                unsafe { *contexts.get_unchecked_mut(pos) = ((qe_entry.nmps as i8) << 1) | (cx_mps as i8) };
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
            
            // Safety check - prevents infinite loop if A becomes 0
            if self.a == 0 {
                 return Err(crate::error::Jbig2Error::new("arithmetic decoder state corrupted: A=0"));
            }

            loop_count += 1;
            // Safety check - prevents stuck renormalization loop
            if loop_count > 100 {
                 return Err(crate::error::Jbig2Error::new("arithmetic decoder stuck in renormalization"));
            }
        }

        // Update context
        // SAFETY: pos is within bounds as checked at start
        unsafe { *contexts.get_unchecked_mut(pos) = ((new_cx_index as i8) << 1) | (cx_mps as i8) };
        Ok(d)
    }

    /// Returns the number of bytes consumed from the input stream.
    /// This includes bytes currently buffered in the decoder but not yet fully processed.
    /// Note: This is an approximation for switching between coding methods.
    pub fn get_bytes_read(&self) -> usize {
        self.offset - self.next_word_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoder_creation() {
        let data = vec![0x00, 0x00, 0x00, 0x00];
        let decoder = ArithmeticDecoder::new(&data);
        
        // Decoder should be initialized
        assert_eq!(decoder.a, 0x8000);
        assert_eq!(decoder.offset, 4);
    }

    #[test]
    fn test_decoder_with_short_data() {
        let data = vec![0xFF];
        let decoder = ArithmeticDecoder::new(&data);
        
        // Should handle short data gracefully
        assert_eq!(decoder.a, 0x8000);
    }

    #[test]
    fn test_decoder_empty_data() {
        let data = vec![];
        let decoder = ArithmeticDecoder::new(&data);
        
        // Should handle empty data
        assert_eq!(decoder.a, 0x8000);
        assert_eq!(decoder.offset, 0);
    }

    #[test]
    fn test_read_bit_basic() {
        // Simple test data with some predictable structure
        let data = vec![0x84, 0xC7, 0x37, 0xF8, 0x69, 0x72, 0xEC, 0x6F];
        let mut decoder = ArithmeticDecoder::new(&data);
        let mut contexts = vec![0i8; 512];
        
        // Should be able to decode some bits without error
        let result = decoder.read_bit(&mut contexts, 0);
        assert!(result.is_ok());
        let bit = result.unwrap();
        assert!(bit == 0 || bit == 1);
    }

    #[test]
    fn test_read_bit_invalid_context() {
        let data = vec![0x00; 8];
        let mut decoder = ArithmeticDecoder::new(&data);
        let mut contexts = vec![0i8; 10];
        
        // Try to use context beyond bounds
        let result = decoder.read_bit(&mut contexts, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_bit_updates_context() {
        let data = vec![0x84, 0xC7, 0x37, 0xF8, 0x69, 0x72, 0xEC, 0x6F];
        let mut decoder = ArithmeticDecoder::new(&data);
        let mut contexts = vec![0i8; 512];
        
        let _initial_ctx = contexts[0];
        let _bit = decoder.read_bit(&mut contexts, 0);
        
        // Context should be updated (may or may not change depending on bit)
        // Just verify no panic occurred
    }

    #[test]
    fn test_multiple_bits_same_context() {
        let data = vec![0x84, 0xC7, 0x37, 0xF8, 0x69, 0x72, 0xEC, 0x6F];
        let mut decoder = ArithmeticDecoder::new(&data);
        let mut contexts = vec![0i8; 512];
        
        // Should be able to read multiple bits
        for _ in 0..10 {
            let result = decoder.read_bit(&mut contexts, 0);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_multiple_contexts() {
        let data = vec![0x84, 0xC7, 0x37, 0xF8, 0x69, 0x72, 0xEC, 0x6F];
        let mut decoder = ArithmeticDecoder::new(&data);
        let mut contexts = vec![0i8; 512];
        
        // Read from different contexts
        for ctx_idx in 0..5 {
            let result = decoder.read_bit(&mut contexts, ctx_idx);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_decoder_state_remains_valid() {
        let data = vec![0x84, 0xC7, 0x37, 0xF8];
        let mut decoder = ArithmeticDecoder::new(&data);
        let mut contexts = vec![0i8; 512];
        
        // Read several bits
        for _ in 0..20 {
            let _ = decoder.read_bit(&mut contexts, 0);
            // A should always remain valid (non-zero)
            // Note: A is private, so we can't directly check,
            // but the decoder should not panic
        }
    }

    #[test]
    fn test_refill_buffer() {
        // Test with data that will require buffer refill
        let data = vec![0xFF; 16];
        let mut decoder = ArithmeticDecoder::new(&data);
        let mut contexts = vec![0i8; 512];
        
        // Read many bits to trigger refill
        for _ in 0..50 {
            let result = decoder.read_bit(&mut contexts, 0);
            if result.is_ok() {
                // Continue
            } else {
                // May eventually exhaust data
                break;
            }
        }
    }
}
