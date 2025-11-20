#[derive(Clone)]
struct QeEntry {
    qe: u16,
    nmps: u8,
    nlps: u8,
    switch_flag: u8,
}
const QE_TABLE: [QeEntry; 47] = [
    QeEntry {
        qe: 0x5601,
        nmps: 1,
        nlps: 1,
        switch_flag: 1,
    },
    QeEntry {
        qe: 0x3401,
        nmps: 2,
        nlps: 6,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x1801,
        nmps: 3,
        nlps: 9,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x0ac1,
        nmps: 4,
        nlps: 12,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x0521,
        nmps: 5,
        nlps: 29,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x0221,
        nmps: 38,
        nlps: 33,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x5601,
        nmps: 7,
        nlps: 6,
        switch_flag: 1,
    },
    QeEntry {
        qe: 0x5401,
        nmps: 8,
        nlps: 14,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x4801,
        nmps: 9,
        nlps: 14,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x3801,
        nmps: 10,
        nlps: 14,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x3001,
        nmps: 11,
        nlps: 17,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x2401,
        nmps: 12,
        nlps: 18,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x1c01,
        nmps: 13,
        nlps: 20,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x1601,
        nmps: 29,
        nlps: 21,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x5601,
        nmps: 15,
        nlps: 14,
        switch_flag: 1,
    },
    QeEntry {
        qe: 0x5401,
        nmps: 16,
        nlps: 14,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x5101,
        nmps: 17,
        nlps: 15,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x4801,
        nmps: 18,
        nlps: 16,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x3801,
        nmps: 19,
        nlps: 17,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x3401,
        nmps: 20,
        nlps: 18,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x3001,
        nmps: 21,
        nlps: 19,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x2801,
        nmps: 22,
        nlps: 19,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x2401,
        nmps: 23,
        nlps: 20,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x2201,
        nmps: 24,
        nlps: 21,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x1c01,
        nmps: 25,
        nlps: 22,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x1801,
        nmps: 26,
        nlps: 23,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x1601,
        nmps: 27,
        nlps: 24,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x1401,
        nmps: 28,
        nlps: 25,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x1201,
        nmps: 29,
        nlps: 26,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x1101,
        nmps: 30,
        nlps: 27,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x0ac1,
        nmps: 31,
        nlps: 28,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x09c1,
        nmps: 32,
        nlps: 29,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x08a1,
        nmps: 33,
        nlps: 30,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x0521,
        nmps: 34,
        nlps: 31,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x0441,
        nmps: 35,
        nlps: 32,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x02a1,
        nmps: 36,
        nlps: 33,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x0221,
        nmps: 37,
        nlps: 34,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x0141,
        nmps: 38,
        nlps: 35,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x0111,
        nmps: 39,
        nlps: 36,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x0085,
        nmps: 40,
        nlps: 37,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x0049,
        nmps: 41,
        nlps: 38,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x0025,
        nmps: 42,
        nlps: 39,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x0015,
        nmps: 43,
        nlps: 40,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x0009,
        nmps: 44,
        nlps: 41,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x0005,
        nmps: 45,
        nlps: 42,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x0001,
        nmps: 45,
        nlps: 43,
        switch_flag: 0,
    },
    QeEntry {
        qe: 0x5601,
        nmps: 46,
        nlps: 46,
        switch_flag: 0,
    },
];
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

        // CRITICAL: Follow jbig2dec initialization exactly
        // 1. Read first 4 bytes into buffer (BIG-ENDIAN)
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

        eprintln!("=== ARITHMETIC INIT DEBUG ===");
        eprintln!(
            "First 4 bytes in buffer: {:02X} {:02X} {:02X} {:02X}",
            (decoder.next_word >> 24) & 0xFF,
            (decoder.next_word >> 16) & 0xFF,
            (decoder.next_word >> 8) & 0xFF,
            decoder.next_word & 0xFF
        );
        eprintln!(
            "After step 1: next_word = 0x{:08X}, next_word_bytes = {}, offset = {}",
            decoder.next_word, decoder.next_word_bytes, decoder.offset
        );

        // 2. Initialize C: C = (~(next_word >> 8)) & 0xFF0000
        let c = (!(decoder.next_word >> 8)) & 0xFF0000;
        eprintln!("After step 2: C = 0x{:08X}", c);
        decoder.chigh = (c >> 16) & 0xFFFF;
        decoder.clow = c & 0xFFFF;

        // 3. Call byte_in (operates on buffer!)
        decoder.byte_in();
        let c_after = (decoder.chigh << 16) | decoder.clow;
        eprintln!(
            "After step 3 (bytein): C = 0x{:08X}, CT = {}",
            c_after, decoder.ct
        );

        // 4. Finalize: C <<= 7, CT -= 7, A = 0x8000
        let c = (decoder.chigh << 16) | decoder.clow;
        let c = c << 7;
        decoder.chigh = (c >> 16) & 0xFFFF;
        decoder.clow = c & 0xFFFF;
        decoder.ct -= 7;
        decoder.a = 0x8000;

        let c_final = (decoder.chigh << 16) | decoder.clow;
        eprintln!(
            "After step 4 (finalize): A=0x{:04X}, C=0x{:08X}, CT={}",
            decoder.a, c_final, decoder.ct
        );
        eprintln!("============================");

        decoder
    }

    fn byte_in(&mut self) {
        // CRITICAL: This operates on the buffered next_word, NOT on data[offset]!
        // Based on jbig2_arith.c:92-183

        // Line 92: Get current top byte from buffer
        let b_check = ((self.next_word >> 24) & 0xFF) as u8;

        if b_check == 0xFF {
            // Special 0xFF handling (jbig2_arith.c:93-149)
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
            // Normal byte (jbig2_arith.c:154-183)
            // Line 154: Shift buffer left
            self.next_word <<= 8;
            self.next_word_bytes = self.next_word_bytes.saturating_sub(1);

            // Refill buffer if exhausted
            if self.next_word_bytes == 0 {
                self.refill_buffer();
            }

            // Line 177: Get NEW top byte from buffer (after shift!)
            let b = ((self.next_word >> 24) & 0xFF) as u8;

            // Line 178: Update C
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
        }

        // Update context
        contexts[pos] = ((new_cx_index as i8) << 1) | (cx_mps as i8);
        Ok(d)
    }
}
