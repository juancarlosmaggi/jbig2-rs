use crate::bitmap::Bitmap;
use crate::error::Jbig2Error;
use crate::reader::Reader;
use super::mmr_tables::{get_white_run, get_black_run, get_white_makeup, get_black_makeup};

// CCITT Group 4 (MMR) decoder implementation
// Based on ITU-T T.6 specification
#[derive(Clone)]
struct CCITTFaxDecoder {
    reader: Reader,
    width: usize,
    height: usize,
    end_of_block: bool,
    // Reference line for 2D coding
    ref_line: Vec<u8>,
    // Current line being decoded
    curr_line: Vec<u8>,
}
impl CCITTFaxDecoder {
    fn new(reader: Reader, width: usize, height: usize, end_of_block: bool) -> Self {
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
    // Decode a 2D MMR line
    fn decode_2d_line(&mut self, eofb: &mut bool) -> Result<(), Jbig2Error> {
        // Handle width=0 case immediately to prevent infinite loop
        if self.width == 0 {
            return Ok(());
        }

        let mut a0 = -1i32; // Current position in reference line
        let mut x = 0; // Current position in current line
        let mut current_color = 0; // Start with White run

        let mut loop_count = 0;
        loop {
            loop_count += 1;
            if loop_count > self.width * 2 + 1000 {
                 return Err(Jbig2Error::new("Infinite loop detected in decode_2d_line"));
            }
            // Check line completion BEFORE reading mode code
            if a0 != -1 && (a0 as usize) >= self.width {
                break;
            }

            let b1 = self.find_changing_element_of_color(
                &self.ref_line,
                (a0 + 1) as usize,
                self.width,
                1 - current_color,
            );
            let b2 = self.find_changing_element(&self.ref_line, b1, self.width);

            // Try to read mode code
            // If no valid mode matches, it means we've hit end of valid data (padding/garbage)
            let mode = match self.read_mode_code() {
                Ok(m) => m,
                Err(_) => {
                    // No valid mode code = end of line data (not an error!)
                    // This happens when we hit padding bits after valid MMR data
                    break;
                }
            };

            match mode {
                0 => {
                    // Pass mode
                    // Fill with current_color from x to b2
                    for i in x..b2 {
                        self.curr_line[i] = current_color;
                    }
                    x = b2;
                    a0 = x as i32; // Implicitly updated to new a0 (which is b2)
                    // Color does not change
                }
                1 => {
                    // Vertical mode (-1)
                    x = self.write_run(x, b1 as i32, -1, current_color);
                    a0 = x as i32;
                    current_color = 1 - current_color;
                }
                2 => {
                    // Vertical mode (0)
                    x = self.write_run(x, b1 as i32, 0, current_color);
                    a0 = x as i32;
                    current_color = 1 - current_color;
                }
                3 => {
                    // Vertical mode (+1)
                    x = self.write_run(x, b1 as i32, 1, current_color);
                    a0 = x as i32;
                    current_color = 1 - current_color;
                }
                4 => {
                    // Horizontal mode
                    if current_color == 0 {
                        // White, then Black
                        let run1 = self.decode_run_length(true)? as usize;
                        let run2 = self.decode_run_length(false)? as usize;

                        // Write White Run
                        for i in 0..run1 {
                            if x + i < self.width {
                                self.curr_line[x + i] = 0;
                            }
                        }
                        x += run1;

                        // Write Black Run
                        for i in 0..run2 {
                            if x + i < self.width {
                                self.curr_line[x + i] = 1;
                            }
                        }
                        x += run2;
                    } else {
                        // Black, then White
                        let run1 = self.decode_run_length(false)? as usize;
                        let run2 = self.decode_run_length(true)? as usize;

                        // Write Black Run
                        for i in 0..run1 {
                            if x + i < self.width {
                                self.curr_line[x + i] = 1;
                            }
                        }
                        x += run1;

                        // Write White Run
                        for i in 0..run2 {
                            if x + i < self.width {
                                self.curr_line[x + i] = 0;
                            }
                        }
                        x += run2;
                    }
                    a0 = x as i32;
                    // Color does not change (flipped twice)
                }
                5 => {
                    // Vertical mode (-2)
                    x = self.write_run(x, b1 as i32, -2, current_color);
                    a0 = x as i32;
                    current_color = 1 - current_color;
                }
                6 => {
                    // Vertical mode (-3)
                    x = self.write_run(x, b1 as i32, -3, current_color);
                    a0 = x as i32;
                    current_color = 1 - current_color;
                }
                7 => {
                    // Vertical mode (+2)
                    x = self.write_run(x, b1 as i32, 2, current_color);
                    a0 = x as i32;
                    current_color = 1 - current_color;
                }
                8 => {
                    // Vertical mode (+3)
                    x = self.write_run(x, b1 as i32, 3, current_color);
                    a0 = x as i32;
                    current_color = 1 - current_color;
                }
                9 => {
                    // EOFB
                    *eofb = true;
                    break;
                }
                _ => return Err(Jbig2Error::new("invalid MMR mode")),
            }
        }
        Ok(())
    }
    fn read_mode_code(&mut self) -> Result<u8, Jbig2Error> {
        let mut code = 0u32;

        for length in 1..=7 {
            let bit = self.read_bit()? as u32;
            code = (code << 1) | bit;
            match (code, length) {
                (0b1, 1) => return Ok(2),       // V(0)
                (0b001, 3) => return Ok(4),     // Horizontal
                (0b010, 3) => return Ok(1),     // VL(1)
                (0b011, 3) => return Ok(3),     // VR(1)
                (0b0001, 4) => return Ok(0),    // Pass
                (0b000010, 6) => return Ok(5),  // VL(2)
                (0b000011, 6) => return Ok(7),  // VR(2)
                (0b0000010, 7) => return Ok(6), // VL(3)
                (0b0000011, 7) => return Ok(8), // VR(3)
                _ => {}
            }
        }

        // Only check for EOFB if end_of_block is true
        if !self.end_of_block {
            return Err(Jbig2Error::new("no mode code match"));
        }

        // Only check for EOFB if end_of_block is true
        // EOFB is 24 bits (0x001001)
        // From jbig2_mmr.c:1191: EOFB = 0x001001 (24 bits)
        for length in 8..=24 {
            let bit = self.read_bit()? as u32;
            code = (code << 1) | bit;
            if length == 24 && code == 0x001001 {
                return Ok(9); // EOFB
            }
        }
        Err(Jbig2Error::new("no mode code match"))
    }
    fn find_changing_element(&self, line: &[u8], start: usize, end: usize) -> usize {
        let mut i = start;
        while i < end {
            if i == 0 || line[i] != line[i - 1] {
                return i;
            }
            i += 1;
        }
        end
    }
    fn find_changing_element_of_color(
        &self,
        line: &[u8],
        start: usize,
        end: usize,
        color: u8,
    ) -> usize {
        let mut i = start;
        // First, skip pixels that are NOT the target color (if we are currently on !color)
        // But wait, the definition is "first pixel of color !c".
        // If we are at `start`, and line[start] == color, we are good?
        // No, changing element means the *transition* to that color.
        // But jbig2dec says: "It searches the reference line starting from a0 for the first pixel of color !c".
        // This implies it's looking for the *start* of a run of color !c.

        // If line[i] is already color, then i is the start?
        // No, a changing element is defined as an element whose color is different from the previous element.
        // "b1 is the first changing element on the reference line to the right of a0 and of color opposite to a0"
        // Actually, "opposite to current color".

        // Let's implement a simple search:
        // Find the first `i >= start` such that `line[i] == color` AND (`i==0` OR `line[i-1] != color`).
        // But since we are scanning from left to right, we just need to find the first `i` where `line[i] == color`?
        // Wait, if we are inside a run of `color`, the first changing element is the *end* of this run (start of next).
        // If we are inside a run of `!color`, the first changing element is the *start* of the next run (which is `color`).

        // jbig2dec implementation:
        // while (x < w) { if (line[x] == color && (x==0 || line[x-1] != color)) return x; x++; }

        while i < end {
            if line[i] == color && (i == 0 || line[i - 1] != color) {
                return i;
            }
            i += 1;
        }
        end
    }
    fn write_run(&mut self, x: usize, b1: i32, offset: i32, color: u8) -> usize {
        // a1 = b1 + offset
        let a1 = (b1 + offset) as usize;

        // Write pixels
        let end = a1.min(self.width);
        if end > x {
            for i in x..end {
                self.curr_line[i] = color;
            }
        }
        end
    }
    // Decode run length for horizontal mode
    fn decode_run_length(&mut self, white: bool) -> Result<i32, Jbig2Error> {
        let mut code = 0u32;
        let mut length = 0;
        // Read up to 13 bits for run length codes
        for _ in 0..13 {
            let bit = self.read_bit()? as u32;
            code = (code << 1) | bit;
            length += 1;
            // Check for terminating codes
            let run = if white {
                get_white_run(code, length)
            } else {
                get_black_run(code, length)
            };
            if run >= 0 {
                return Ok(run);
            }
            // Check for make-up codes
            let makeup = if white {
                get_white_makeup(code, length)
            } else {
                get_black_makeup(code, length)
            };
            if makeup >= 0 {
                // Make-up code found, add to total and continue
                let mut total = makeup;
                // Reset for next code
                code = 0;
                length = 0;
                // Read terminating code
                for _ in 0..13 {
                    let bit = self.read_bit()? as u32;
                    code = (code << 1) | bit;
                    length += 1;
                    let term = if white {
                        get_white_run(code, length)
                    } else {
                        get_black_run(code, length)
                    };
                    if term >= 0 {
                        total += term;
                        return Ok(total);
                    }
                }
                return Err(Jbig2Error::new("invalid terminating code after makeup"));
            }
        }
        Err(Jbig2Error::new("run length code too long"))
    }
    fn decode(&mut self) -> Result<Bitmap, Jbig2Error> {
        let mut bitmap = Bitmap::new(self.width, self.height);
        let mut eofb = false;
        let mut y = 0;

        while y < self.height && !eofb {
            self.decode_2d_line(&mut eofb)?;

            // Copy current line to bitmap
            for x in 0..self.width {
                bitmap.set_pixel(x, y, self.curr_line[x]);
            }
            // Swap reference and current lines
            std::mem::swap(&mut self.ref_line, &mut self.curr_line);
            self.curr_line.fill(0);

            y += 1;
        }

        // If EOFB was encountered before reaching height, pad remaining lines with zeros
        // (Lines are already initialized to 0 in Bitmap::new)

        Ok(bitmap)
    }
}
pub fn decode_mmr_bitmap(
    input: &mut Reader,
    width: usize,
    height: usize,
    end_of_block: bool,
) -> Result<Bitmap, Jbig2Error> {
    if width == 0 || height == 0 {
        return Ok(Bitmap::new(width, height));
    }
    let reader_clone = Reader::new(
        input.get_data().to_vec(),
        input.get_position(),
        input.get_end(),
    );
    let mut decoder = CCITTFaxDecoder::new(reader_clone, width, height, end_of_block);
    let result = decoder.decode()?;
    input.set_position(decoder.reader.get_position());
    Ok(result)
}
