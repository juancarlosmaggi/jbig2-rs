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
    pub data: Vec<u8>,
    pub bp: usize,
    pub data_end: usize,
    pub chigh: u32,
    pub clow: u32,
    pub ct: i8,
    pub a: u16,
}
impl ArithmeticDecoder {
    pub fn new(data: &[u8]) -> Self {
        let mut decoder = ArithmeticDecoder {
            data: data.to_vec(),
            bp: 1, // Start at 1 because data[0] is already read into chigh
            data_end: data.len(),
            chigh: data[0] as u32,
            clow: 0,
            ct: 0,
            a: 0x8000,
        };
        decoder.byte_in();
        decoder.chigh = ((decoder.chigh << 7) & 0xffff) | ((decoder.clow >> 9) & 0x7f);
        decoder.clow = (decoder.clow << 7) & 0xffff;
        decoder.ct = 0;
        decoder.a = 0x8000;
        decoder
    }
    fn byte_in(&mut self) {
        if self.bp >= self.data_end {
            // Out of data - set ct to prevent underflow, add 0xFF00 to maintain state
            self.clow = self.clow.wrapping_add(0xff00u32);
            self.ct = 8;
            return;
        }
        let b0 = self.data[self.bp];
        self.bp += 1;
        if b0 == 0xff {
            if self.bp >= self.data_end {
                self.clow = self.clow.wrapping_add(0xff00u32);
                self.ct = 8;
                return;
            }
            let b1 = self.data[self.bp];
            if b1 > 0x8f {
                self.clow = self.clow.wrapping_add(0xff00u32);
                self.ct = 8;
                return;
            }
            self.bp += 1;
            self.clow = self.clow.wrapping_add((b1 as u32) << 8);
            self.ct = 8;
        } else {
            self.clow = self.clow.wrapping_add((b0 as u32) << 8);
            self.ct = 8;
        }
        if self.clow > 0xffff {
            self.chigh = self.chigh.wrapping_add(self.clow >> 16);
            self.clow &= 0xffff;
        }
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
        let d;
        let new_cx_index;
        let mut a = self.a.wrapping_sub(qe_icx);
        if self.chigh < qe_icx as u32 {
            // exchangeLps
            if a < qe_icx {
                a = qe_icx;
                d = cx_mps;
                new_cx_index = qe_entry.nmps as usize;
            } else {
                a = qe_icx;
                d = 1 ^ cx_mps;
                if qe_entry.switch_flag == 1 {
                    cx_mps = d;
                }
                new_cx_index = qe_entry.nlps as usize;
            }
        } else {
            self.chigh -= qe_icx as u32;
            if (a & 0x8000) != 0 {
                self.a = a;
                return Ok(cx_mps);
            }
            // exchangeMps
            if a < qe_icx {
                d = 1 ^ cx_mps;
                if qe_entry.switch_flag == 1 {
                    cx_mps = d;
                }
                new_cx_index = qe_entry.nlps as usize;
            } else {
                d = cx_mps;
                new_cx_index = qe_entry.nmps as usize;
            }
        }
        // renormD - Figure E.18 from JBIG2 spec
        loop {
            // CRITICAL: Check ct BEFORE decrementing (matches jbig2dec)
            if self.ct == 0 {
                self.byte_in();  // Sets ct = 8
            }
            a <<= 1;
            self.chigh = ((self.chigh << 1) & 0xffff) | ((self.clow >> 15) & 1);
            self.clow = (self.clow << 1) & 0xffff;
            self.ct -= 1;  // Safe: ct was >= 1
            if (a & 0x8000) != 0 {
                break;
            }
        }
        self.a = a;
        contexts[pos] = ((new_cx_index as i8) << 1) | (cx_mps as i8);
        Ok(d)
    }
}
