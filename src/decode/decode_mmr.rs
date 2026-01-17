use super::mmr_tables::{get_black_makeup, get_black_run, get_white_makeup, get_white_run};
use crate::bitmap::Bitmap;
use crate::error::Jbig2Error;
use crate::reader::Reader;

// CCITT Group 4 (MMR) decoder implementation with robustness checks.
#[derive(Clone)]
struct CCITTFaxDecoder {
    reader: Reader,
    width: usize,
    height: usize,
    end_of_block: bool,
    ref_line: Vec<u8>,
    curr_line: Vec<u8>,
    trace: bool,
    invalid_modes: u32,
    invalid_runs: u32,
}

impl CCITTFaxDecoder {
    fn new(reader: Reader, width: usize, height: usize, end_of_block: bool, trace: bool) -> Self {
        CCITTFaxDecoder {
            reader,
            width,
            height,
            end_of_block,
            ref_line: vec![0; width],
            curr_line: vec![0; width],
            trace,
            invalid_modes: 0,
            invalid_runs: 0,
        }
    }

    fn read_bit(&mut self) -> Result<u8, Jbig2Error> {
        self.reader.read_bit()
    }

    fn decode_2d_line(&mut self, eofb: &mut bool) -> Result<(), Jbig2Error> {
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
                    if self.trace {
                        self.invalid_modes = self.invalid_modes.saturating_add(1);
                    }
                    // Invalid code or end-of-data; finish the line with white.
                    break;
                }
            };

            if mode == 9 {
                *eofb = true;
                break;
            }

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
                            if self.trace {
                                self.invalid_runs = self.invalid_runs.saturating_add(1);
                            }
                            break; // finish with white
                        }
                    };
                    let r2 = match self.decode_run_length(!white_first) {
                        Ok(r) => r as usize,
                        Err(_) => {
                            if self.trace {
                                self.invalid_runs = self.invalid_runs.saturating_add(1);
                            }
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

        // If end_of_block, look for the EOFB marker without consuming bits unless matched.
        if self.end_of_block {
            // Save full reader state.
            let saved_pos = self.reader.get_position();
            let saved_shift = self.reader.get_shift();
            let saved_current_byte = self.reader.get_current_byte();

            let mut full_code = code;
            let mut full_len = length;
            let mut matched = false;

            for _ in length..24 {
                match self.read_bit() {
                    Ok(bit) => {
                        full_code = (full_code << 1) | (bit as u32);
                        full_len += 1;
                        if full_len == 24 && full_code == 0x001001 {
                            matched = true;
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }

            if matched {
                return Ok(9);
            } else {
                // Restore state if the marker was not matched.
                self.reader.set_position(saved_pos);
                self.reader.set_shift(saved_shift);
                self.reader.set_current_byte(saved_current_byte);
            }
        }
        Err(Jbig2Error::new("no valid MMR mode code"))
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

            for x in 0..self.width {
                bitmap.set_pixel(x, y, self.curr_line[x]);
            }

            std::mem::swap(&mut self.ref_line, &mut self.curr_line);
            self.curr_line.fill(0);

            y += 1;
        }

        Ok(bitmap)
    }
}

/// Decode an MMR-encoded bitmap from the reader.
pub fn decode_mmr_bitmap(
    input: &mut Reader,
    width: usize,
    height: usize,
    end_of_block: bool,
) -> Result<Bitmap, Jbig2Error> {
    if width == 0 || height == 0 {
        return Ok(Bitmap::new(width, height));
    }

    let data_clone = input.get_data().to_vec();
    let pos = input.get_position();
    let end = input.get_end();
    let trace = std::env::var_os("JBIG2_RS_TRACE_MMR").is_some();

    let reader = Reader::new(data_clone, pos, end);
    let mut decoder = CCITTFaxDecoder::new(reader, width, height, end_of_block, trace);

    let bitmap = decoder.decode()?;

    input.set_position(decoder.reader.get_position());
    if decoder.trace && (decoder.invalid_modes > 0 || decoder.invalid_runs > 0) {
        let bytes = end.saturating_sub(pos);
        eprintln!(
            "mmr: width={} height={} bytes={} invalid_modes={} invalid_runs={}",
            width,
            height,
            bytes,
            decoder.invalid_modes,
            decoder.invalid_runs
        );
    }

    Ok(bitmap)
}
