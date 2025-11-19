use crate::bitmap::Bitmap;
use crate::error::Jbig2Error;
use crate::reader::Reader;
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
    fn decode_2d_line(&mut self) -> Result<(), Jbig2Error> {
        let mut a0 = -1i32; // Current position in reference line
        let mut x = 0; // Current position in current line
        let mut current_color = 0; // Start with White run

        while x < self.width {
            let b1 = self.find_changing_element_of_color(&self.ref_line, (a0 + 1) as usize, self.width, 1 - current_color);
            let b2 = self.find_changing_element(&self.ref_line, b1, self.width);
            // Read mode code
            let mode = self.read_mode_code()?;
            eprintln!("DEBUG: MMR x={} a0={} mode={} color={} b1={} b2={}", x, a0, mode, current_color, b1, b2);
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
                    let mut a1;
                    if current_color == 0 {
                        // White, then Black
                        let run1 = self.decode_run_length(true)? as usize;
                        let run2 = self.decode_run_length(false)? as usize;
                        eprintln!("DEBUG: MMR Horizontal White={} Black={}", run1, run2);
                        
                        // Write White Run
                        for i in 0..run1 {
                            if x + i < self.width {
                                self.curr_line[x + i] = 0;
                            }
                        }
                        x += run1;
                        a1 = x;
                        
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
                        eprintln!("DEBUG: MMR Horizontal Black={} White={}", run1, run2);
                        
                        // Write Black Run
                        for i in 0..run1 {
                            if x + i < self.width {
                                self.curr_line[x + i] = 1;
                            }
                        }
                        x += run1;
                        a1 = x;
                        
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
                    eprintln!("DEBUG: MMR EOFB encountered");
                    return Ok(());
                }
                _ => return Err(Jbig2Error::new("invalid MMR mode")),
            }
        }
        Ok(())
    }
    fn read_mode_code(&mut self) -> Result<u8, Jbig2Error> {
        // Read up to 7 bits for mode codes
        let mut code = 0u32;
        for length in 1..=7 {
            let bit = self.read_bit()? as u32;
            code = (code << 1) | bit;
            // eprintln!("DEBUG: read_mode_code length={} code={:b}", length, code);
            match (code, length) {
                (0b0001, 4) => return Ok(0),    // Pass
                (0b010, 3) => return Ok(1),     // VL(1)
                (0b1, 1) => return Ok(2),       // V(0)
                (0b011, 3) => return Ok(3),     // VR(1)
                (0b001, 3) => return Ok(4),     // Horizontal
                (0b000010, 6) => return Ok(5),  // VL(2)
                (0b0000010, 7) => return Ok(6), // VL(3)
                (0b000011, 6) => return Ok(7),  // VR(2)
                (0b0000011, 7) => return Ok(8), // VR(3)
                _ => {
                    // Check for EOFB: 0x001001 (24 bits)
                    // We have read 'length' bits so far.
                    // If we have read 24 bits and it matches, return EOFB code (e.g., 9)
                    if length == 24 && code == 0x001001 {
                        return Ok(9); // EOFB
                    }
                } 
            }
        }
        // Continue reading up to 24 bits for EOFB if needed
        for length in 8..=24 {
             let bit = self.read_bit()? as u32;
             code = (code << 1) | bit;
             if length == 24 && code == 0x001001 {
                 return Ok(9); // EOFB
             }
        }
        eprintln!("DEBUG: Invalid MMR mode code: {:b} (len=24)", code);
        Err(Jbig2Error::new("invalid MMR mode code"))
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
    fn find_changing_element_of_color(&self, line: &[u8], start: usize, end: usize, color: u8) -> usize {
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
            if line[i] == color {
                if i == 0 || line[i-1] != color {
                    return i;
                }
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
                Self::get_white_run(code, length)
            } else {
                Self::get_black_run(code, length)
            };
            if run >= 0 {
                return Ok(run);
            }
            // Check for make-up codes
            let makeup = if white {
                Self::get_white_makeup(code, length)
            } else {
                Self::get_black_makeup(code, length)
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
                        Self::get_white_run(code, length)
                    } else {
                        Self::get_black_run(code, length)
                    };
                    if term >= 0 {
                        total += term;
                        return Ok(total);
                    }
                }
                eprintln!("DEBUG: Invalid terminating code after makeup: {:b} (len={}) white={}", code, length, white);
                return Err(Jbig2Error::new("invalid terminating code after makeup"));
            }
        }
        eprintln!("DEBUG: Run length code too long: {:b} (len={}) white={}", code, length, white);
        Err(Jbig2Error::new("run length code too long"))
    }
    // White run length terminating codes
    #[allow(unreachable_patterns)]
    fn get_white_run(code: u32, length: usize) -> i32 {
        match (code, length) {
            (0b00110101, 8) => 0,
            (0b000111, 6) => 1,
            (0b0111, 4) => 2,
            (0b1000, 4) => 3,
            (0b1011, 4) => 4,
            (0b1100, 4) => 5,
            (0b1110, 4) => 6,
            (0b1111, 4) => 7,
            (0b10011, 5) => 8,
            (0b10100, 5) => 9,
            (0b00111, 5) => 10,
            (0b01000, 5) => 11,
            (0b001000, 6) => 12,
            (0b000011, 6) => 13,
            (0b110100, 6) => 14,
            (0b110101, 6) => 15,
            (0b1011010, 7) => 16,
            (0b1011011, 7) => 17,
            (0b1011100, 7) => 18,
            (0b1011101, 7) => 19,
            (0b01001100, 8) => 20,
            (0b01001101, 8) => 21,
            (0b01001110, 8) => 22,
            (0b01001111, 8) => 23,
            (0b01010000, 8) => 24,
            (0b01010001, 8) => 25,
            (0b01010010, 8) => 26,
            (0b01010011, 8) => 27,
            (0b01010100, 8) => 28,
            (0b01010101, 8) => 29,
            (0b001101100, 9) => 30,
            (0b001101101, 9) => 31,
            (0b001101110, 9) => 32,
            (0b001101111, 9) => 33,
            (0b001110000, 9) => 34,
            (0b001110001, 9) => 35,
            (0b001110010, 9) => 36,
            (0b001110011, 9) => 37,
            (0b001110100, 9) => 38,
            (0b001110101, 9) => 39,
            (0b001110110, 9) => 40,
            (0b001110111, 9) => 41,
            (0b001111000, 9) => 42,
            (0b001111001, 9) => 43,
            (0b001111010, 9) => 44,
            (0b001111011, 9) => 45,
            (0b001111100, 9) => 46,
            (0b001111101, 9) => 47,
            (0b001111110, 9) => 48,
            (0b001111111, 9) => 49,
            (0b0001101100, 10) => 50,
            (0b0001101101, 10) => 51,
            (0b0001101110, 10) => 52,
            (0b0001101111, 10) => 53,
            (0b0001110000, 10) => 54,
            (0b0001110001, 10) => 55,
            (0b0001110010, 10) => 56,
            (0b0001110011, 10) => 57,
            (0b0001110100, 10) => 58,
            (0b0001110101, 10) => 59,
            (0b0001110110, 10) => 60,
            (0b0001110111, 10) => 61,
            (0b0001111000, 10) => 62,
            (0b0001111001, 10) => 63,
            _ => -1,
        }
    }
    // Black run length terminating codes
    fn get_black_run(code: u32, length: usize) -> i32 {
        match (code, length) {
            (0b0000110111, 10) => 0,
            (0b010, 3) => 1,
            (0b11, 2) => 2,
            (0b10, 2) => 3,
            (0b011, 3) => 4,
            (0b0011, 4) => 5,
            (0b0010, 4) => 6,
            (0b00011, 5) => 7,
            (0b000101, 6) => 8,
            (0b000100, 6) => 9,
            (0b0000100, 7) => 10,
            (0b0000101, 7) => 11,
            (0b0000111, 7) => 12,
            (0b00000100, 8) => 13,
            (0b00000101, 8) => 14,
            (0b00000110, 8) => 15,
            (0b00000111, 8) => 16,
            (0b00001000, 8) => 17,
            (0b00001001, 8) => 18,
            (0b00001010, 8) => 19,
            (0b00001011, 8) => 20,
            (0b00001100, 8) => 21,
            (0b00001101, 8) => 22,
            (0b00001110, 8) => 23,
            (0b00001111, 8) => 24,
            (0b00010000, 8) => 25,
            (0b00010001, 8) => 26,
            (0b00010010, 8) => 27,
            (0b00010011, 8) => 28,
            (0b00010100, 8) => 29,
            (0b00010101, 8) => 30,
            (0b00010110, 8) => 31,
            (0b00010111, 8) => 32,
            (0b00011000, 8) => 33,
            (0b00011001, 8) => 34,
            (0b00011010, 8) => 35,
            (0b00011011, 8) => 36,
            (0b00011100, 8) => 37,
            (0b00011101, 8) => 38,
            (0b00011110, 8) => 39,
            (0b00011111, 8) => 40,
            (0b00100000, 8) => 41,
            (0b00100001, 8) => 42,
            (0b00100010, 8) => 43,
            (0b00100011, 8) => 44,
            (0b00100100, 8) => 45,
            (0b00100101, 8) => 46,
            (0b00100110, 8) => 47,
            (0b00100111, 8) => 48,
            (0b00101000, 8) => 49,
            (0b00101001, 8) => 50,
            (0b00101010, 8) => 51,
            (0b00101011, 8) => 52,
            (0b00101100, 8) => 53,
            (0b00101101, 8) => 54,
            (0b00101110, 8) => 55,
            (0b00101111, 8) => 56,
            (0b00110000, 8) => 57,
            (0b00110001, 8) => 58,
            (0b00110010, 8) => 59,
            (0b00110011, 8) => 60,
            (0b00110100, 8) => 61,
            (0b00110101, 8) => 62,
            (0b00110110, 8) => 63,
            _ => -1,
        }
    }
    // White make-up codes
    fn get_white_makeup(code: u32, length: usize) -> i32 {
        match (code, length) {
            (0b11011, 5) => 64,
            (0b10010, 5) => 128,
            (0b010111, 6) => 192,
            (0b0110111, 7) => 256,
            (0b00110110, 8) => 320,
            (0b00110111, 8) => 384,
            (0b011001100, 9) => 448,
            (0b011001101, 9) => 512,
            (0b011001110, 9) => 576,
            (0b011001111, 9) => 640,
            (0b00011001100, 11) => 704,
            (0b00011001101, 11) => 768,
            (0b00011001110, 11) => 832,
            (0b00011001111, 11) => 896,
            (0b000100110000, 12) => 960,
            (0b000100110001, 12) => 1024,
            (0b000100110010, 12) => 1088,
            (0b000100110011, 12) => 1152,
            (0b000100110100, 12) => 1216,
            (0b000100110101, 12) => 1280,
            (0b000100110110, 12) => 1344,
            (0b000100110111, 12) => 1408,
            (0b0000110011000, 13) => 1472,
            (0b0000110011001, 13) => 1536,
            (0b0000110011010, 13) => 1600,
            (0b0000110011011, 13) => 1664,
            (0b0000110011100, 13) => 1728,
            (0b0000110011101, 13) => 1792,
            (0b0000110011110, 13) => 1856,
            (0b0000110011111, 13) => 1920,
            (0b0000011011000, 13) => 1984,
            (0b0000011011001, 13) => 2048,
            (0b0000011011010, 13) => 2112,
            (0b0000011011011, 13) => 2176,
            (0b0000011011100, 13) => 2240,
            (0b0000011011101, 13) => 2304,
            (0b0000011011110, 13) => 2368,
            (0b0000011011111, 13) => 2432,
            (0b00000011011000, 14) => 2496,
            (0b00000011011001, 14) => 2560,
            _ => -1,
        }
    }
    // Black make-up codes
    fn get_black_makeup(code: u32, length: usize) -> i32 {
        match (code, length) {
            (0b0000001111, 10) => 64,
            (0b000011001000, 12) => 128,
            (0b000011001001, 12) => 192,
            (0b000001011011, 12) => 256,
            (0b000000110011, 12) => 320,
            (0b000000110100, 12) => 384,
            (0b000000110101, 12) => 448,
            (0b0000001101100, 13) => 512,
            (0b0000001101101, 13) => 576,
            (0b0000001001010, 13) => 640,
            (0b0000001001011, 13) => 704,
            (0b0000001001100, 13) => 768,
            (0b0000001001101, 13) => 832,
            (0b0000001110010, 13) => 896,
            (0b0000001110011, 13) => 960,
            (0b0000001110100, 13) => 1024,
            (0b0000001110101, 13) => 1088,
            (0b0000001110110, 13) => 1152,
            (0b0000001110111, 13) => 1216,
            (0b0000001010010, 13) => 1280,
            (0b0000001010011, 13) => 1344,
            (0b0000001010100, 13) => 1408,
            (0b0000001010101, 13) => 1472,
            (0b0000001011010, 13) => 1536,
            (0b0000001011011, 13) => 1600,
            (0b0000001100100, 13) => 1664,
            (0b0000001100101, 13) => 1728,
            (0b0000001100110, 13) => 1792,
            (0b0000001100111, 13) => 1856,
            (0b0000001011000, 13) => 1920,
            (0b0000001011001, 13) => 1984,
            (0b0000001011100, 13) => 2112,
            (0b0000001011101, 13) => 2240,
            (0b0000001011110, 13) => 2304,
            (0b0000001011111, 13) => 2368,
            (0b0000001101000, 13) => 2432,
            (0b0000001101001, 13) => 2496,
            (0b0000001101010, 13) => 2560,
            _ => -1,
        }
    }
    fn decode(&mut self) -> Result<Bitmap, Jbig2Error> {
        let mut bitmap = Bitmap::new(self.width, self.height);
        for y in 0..self.height {
            // Decode 2D line
            self.decode_2d_line()?;
            // Copy current line to bitmap
            for x in 0..self.width {
                bitmap.set_pixel(x, y, self.curr_line[x]);
            }
            // Swap reference and current lines
            std::mem::swap(&mut self.ref_line, &mut self.curr_line);
            self.curr_line.fill(0);
        }
        // Handle EOFB if required
        if self.end_of_block {
            // Look for EOFB pattern: two consecutive EOL codes (000000000001)
            let mut eol_count = 0;
            while eol_count < 2 {
                let mut code = 0u16;
                for _ in 0..12 {
                    let bit = self.read_bit()? as u16;
                    code = (code << 1) | bit;
                }
                if code & 1 == 1 {
                    eol_count += 1;
                } else {
                    eol_count = 0;
                }
            }
        }
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
