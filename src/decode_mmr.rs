use crate::bitmap::Bitmap;
use crate::error::Jbig2Error;
use crate::reader::Reader;

// CCITT Group 4 (MMR) decoder implementation
// Based on ITU-T T.6 specification

#[derive(Clone)]
struct MMRDecoder {
    reader: Reader,
    width: usize,
    height: usize,
    _black_is_1: bool,
    _end_of_block: bool,
}

impl MMRDecoder {
    fn new(reader: Reader, width: usize, height: usize, black_is_1: bool, end_of_block: bool) -> Self {
        MMRDecoder {
            reader,
            width,
            height,
            _black_is_1: black_is_1,
            _end_of_block: end_of_block,
        }
    }

    // Read a bit from the stream
    fn read_bit(&mut self) -> Result<u8, Jbig2Error> {
        self.reader.read_bit()
    }

    // Decode a run length using the 1D Huffman codes
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
                return Err(Jbig2Error::new("invalid terminating code after makeup"));
            }
        }
        Err(Jbig2Error::new("run length code too long"))
    }

    // White run length terminating codes (simplified - only common ones)
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
            // Add more as needed...
            _ => -1,
        }
    }

    // Black run length terminating codes (simplified)
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
            // Add more as needed...
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
            _ => -1,
        }
    }

    // Decode MMR bitmap
    fn decode(&mut self) -> Result<Bitmap, Jbig2Error> {
        let mut bitmap = Bitmap::new(self.width, self.height);
        let mut reference_line = vec![0u8; self.width];
        let mut current_line = vec![0u8; self.width];

        for y in 0..self.height {
            let mut x = 0;
            let _a0 = -1; // Reference position
            let mut color = 0; // 0 = white, 1 = black

            // For MMR, we need to handle 2D coding
            // This is a simplified implementation - full MMR would need proper 2D mode detection

            // For now, fall back to basic 1D decoding per line
            while x < self.width {
                let run_length = self.decode_run_length(color == 0)?;
                for i in 0..run_length as usize {
                    if x + i < self.width {
                        current_line[x + i] = color;
                    }
                }
                x += run_length as usize;
                color = 1 - color; // Switch color
            }

            // Copy current line to bitmap
            for (i, &pixel) in current_line.iter().enumerate() {
                bitmap.set_pixel(i, y, pixel);
            }

            // Update reference line
            reference_line.copy_from_slice(&current_line);
            current_line.fill(0);
        }

        Ok(bitmap)
    }
}

pub fn decode_mmr_bitmap(input: &mut Reader, width: usize, height: usize, end_of_block: bool) -> Result<Bitmap, Jbig2Error> {
    let reader_clone = Reader::new(input.get_data().to_vec(), input.get_position(), input.get_end());
    let mut decoder = MMRDecoder::new(reader_clone, width, height, true, end_of_block);
    let result = decoder.decode()?;
    input.set_position(decoder.reader.get_position());
    Ok(result)
}