use super::mmr_tables::{get_black_makeup, get_black_run, get_white_makeup, get_white_run};
use crate::bitmap::Bitmap;
use crate::common::error::Jbig2Error;
use crate::common::reader::Reader;

// CCITT Group 4 (MMR) decoder implementation with robustness checks.
#[derive(Clone)]
struct CCITTFaxDecoder<'a> {
    reader: Reader<'a>,
    width: usize,
    height: usize,
    end_of_block: bool,
    ref_line: Vec<u8>,
    curr_line: Vec<u8>,
}

impl<'a> CCITTFaxDecoder<'a> {
    fn new(reader: Reader<'a>, width: usize, height: usize, end_of_block: bool) -> Self {
        CCITTFaxDecoder {
            reader,
            width,
            height,
            end_of_block,
            ref_line: vec![0; width],
            curr_line: vec![0; width],
        }
    }

    fn read_bit(&mut self) -> Result<u8, Jbig2Error> {
        self.reader.read_bit()
    }

    fn decode_2d_line(&mut self, _eofb: &mut bool) -> Result<(), Jbig2Error> {
        if self.width == 0 {
            return Ok(());
        }

        let mut a0: i32 = -1;
        let mut x: usize = 0;
        let mut current_color: u8 = 0; // always start with white

        let mut loop_guard = 0;
        while x < self.width {
            loop_guard += 1;
            if loop_guard > self.width * 3 + 1000 {
                return Err(Jbig2Error::new("Infinite loop in MMR line decoding"));
            }

            let b1 = self.find_changing_element_of_color(
                &self.ref_line,
                a0,
                self.width,
                1 - current_color,
            );
            let b2 = self.find_changing_element(&self.ref_line, b1 as i32, self.width);

            let mode = match self.read_mode_code() {
                Ok(m) => m,
                Err(_) => {
                    // Invalid code or end-of-data; finish the line with white.
                    break;
                }
            };

            match mode {
                0 => {
                    // Pass mode
                    for i in x..b2 {
                        self.curr_line[i] = current_color;
                    }
                    x = b2;
                    a0 = x as i32;
                    // color unchanged
                }
                1 | 2 | 3 | 5 | 6 | 7 | 8 => {
                    let offset = match mode {
                        1 => -1, // VL(-1)
                        2 => 0,  // V(0)
                        3 => 1,  // VR(1)
                        5 => -2, // VL(-2)
                        6 => -3, // VL(-3)
                        7 => 2,  // VR(2)
                        8 => 3,  // VR(3)
                        _ => 0,  // unreachable
                    };

                    let mut a1 = (b1 as i32) + offset;
                    if a1 < 0 {
                        a1 = 0;
                    }

                    let end = (a1 as usize).min(self.width);
                    for i in x..end {
                        self.curr_line[i] = current_color;
                    }
                    x = end;
                    a0 = a1;
                    current_color = 1 - current_color;
                }
                4 => {
                    // Horizontal mode
                    let white_first = current_color == 0;
                    let r1 = match self.decode_run_length(white_first) {
                        Ok(r) => r as usize,
                        Err(_) => {
                            break; // finish with white
                        }
                    };
                    let r2 = match self.decode_run_length(!white_first) {
                        Ok(r) => r as usize,
                        Err(_) => {
                            break;
                        }
                    };

                    for i in 0..r1 {
                        if x + i < self.width {
                            self.curr_line[x + i] = current_color;
                        }
                    }
                    x += r1;

                    for i in 0..r2 {
                        if x + i < self.width {
                            self.curr_line[x + i] = 1 - current_color;
                        }
                    }
                    x += r2;

                    a0 = x as i32;
                    // color flips twice → unchanged
                }
                _ => return Err(Jbig2Error::new("invalid MMR mode code")),
            }
        }

        // Remaining pixels after early termination stay white.
        Ok(())
    }

    fn read_mode_code(&mut self) -> Result<u8, Jbig2Error> {
        let mut code: u32 = 0;
        let mut length: usize = 0;

        // First try normal modes (1–7 bits).
        for _ in 0..7 {
            let bit = self.read_bit()? as u32;
            code = (code << 1) | bit;
            length += 1;

            if let Some(mode) = match (code, length) {
                (0b1, 1) => Some(2),       // V(0)
                (0b001, 3) => Some(4),     // H
                (0b010, 3) => Some(1),     // VL(-1)
                (0b011, 3) => Some(3),     // VR(+1)
                (0b0001, 4) => Some(0),    // Pass
                (0b000010, 6) => Some(5),  // VL(-2)
                (0b000011, 6) => Some(7),  // VR(+2)
                (0b0000010, 7) => Some(6), // VL(-3)
                (0b0000011, 7) => Some(8), // VR(+3)
                _ => None,
            } {
                return Ok(mode);
            }
        }

        Err(Jbig2Error::new("no valid MMR mode code"))
    }

    fn consume_eofb_marker(&mut self) -> bool {
        let saved_pos = self.reader.get_position();
        let saved_shift = self.reader.get_shift();
        let saved_current_byte = self.reader.get_current_byte();

        let mut code = 0u32;
        for _ in 0..24 {
            match self.read_bit() {
                Ok(bit) => {
                    code = (code << 1) | (bit as u32);
                }
                Err(_) => {
                    code = 0;
                    break;
                }
            }
        }

        if code == 0x001001 {
            true
        } else {
            self.reader.set_position(saved_pos);
            self.reader.set_shift(saved_shift);
            self.reader.set_current_byte(saved_current_byte);
            false
        }
    }

    fn find_changing_element(&self, line: &[u8], pos: i32, width: usize) -> usize {
        if width == 0 {
            return 0;
        }
        if pos < 0 {
            let mut x = 0usize;
            while x < width {
                if line[x] != 0 {
                    return x;
                }
                x += 1;
            }
            return width;
        }
        let mut x = pos as usize;
        if x >= width {
            return width;
        }
        let color = line[x];
        x = x.saturating_add(1);
        while x < width {
            if line[x] != color {
                return x;
            }
            x += 1;
        }
        width
    }

    fn find_changing_element_of_color(
        &self,
        line: &[u8],
        pos: i32,
        width: usize,
        color: u8,
    ) -> usize {
        let mut x = self.find_changing_element(line, pos, width);
        if x < width && line[x] != color {
            x = self.find_changing_element(line, x as i32, width);
        }
        x
    }

    fn decode_run_length(&mut self, white: bool) -> Result<i32, Jbig2Error> {
        let mut total = 0i32;

        loop {
            let mut code = 0u32;
            let mut clen = 0usize;
            let mut found_makeup = false;

            // Read up to 14 bits to cover the longest code.
            while clen < 14 {
                let bit = self.read_bit()? as u32;
                code = (code << 1) | bit;
                clen += 1;

                // Check for a terminating code.
                let term = if white {
                    get_white_run(code, clen)
                } else {
                    get_black_run(code, clen)
                };
                if term >= 0 {
                    total += term;
                    return Ok(total);
                }

                // Check for a make-up code.
                let makeup = if white {
                    get_white_makeup(code, clen)
                } else {
                    get_black_makeup(code, clen)
                };
                if makeup >= 0 {
                    total += makeup;
                    found_makeup = true;
                    break;
                }
            }

            // If we found a makeup code, continue to read the next code.
            if found_makeup {
                continue;
            }

            // Fell off the end without a match.
            return Err(Jbig2Error::new("invalid run-length code"));
        }
    }

    fn decode(&mut self) -> Result<Bitmap, Jbig2Error> {
        let mut bitmap = Bitmap::new(self.width, self.height);
        let mut eofb = false;
        let mut y = 0usize;

        while y < self.height && !eofb {
            self.decode_2d_line(&mut eofb)?;

            let row_start = y * bitmap.stride;
            let row_slice = &mut bitmap.data[row_start..row_start + bitmap.stride];

            for (i, chunk) in self.curr_line.chunks(8).enumerate() {
                let mut byte = 0u8;
                for (bit_idx, &pixel) in chunk.iter().enumerate() {
                    if pixel != 0 {
                        byte |= 1 << (7 - bit_idx);
                    }
                }
                row_slice[i] = byte;
            }

            std::mem::swap(&mut self.ref_line, &mut self.curr_line);
            self.curr_line.fill(0);

            y += 1;
        }
        if self.end_of_block {
            self.consume_eofb_marker();
        }

        Ok(bitmap)
    }
}

/// Decode an MMR-encoded bitmap from the reader.
pub fn decode_mmr_bitmap(
    input: &mut Reader<'_>,
    width: usize,
    height: usize,
    end_of_block: bool,
) -> Result<Bitmap, Jbig2Error> {
    if width == 0 || height == 0 {
        return Ok(Bitmap::new(width, height));
    }

    let pos = input.get_position();
    let end = input.get_end();
    let data = input.get_data();

    let (bitmap, new_pos) = {
        let reader = Reader::new(data, pos, end);
        let mut decoder = CCITTFaxDecoder::new(reader, width, height, end_of_block);
        let bitmap = decoder.decode()?;
        let new_pos = decoder.reader.get_position();
        (bitmap, new_pos)
    };

    input.set_position(new_pos);

    Ok(bitmap)
}
