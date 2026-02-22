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
}

impl<'a> CCITTFaxDecoder<'a> {
    fn new(reader: Reader<'a>, width: usize, height: usize, end_of_block: bool) -> Self {
        CCITTFaxDecoder {
            reader,
            width,
            height,
            end_of_block,
        }
    }

    fn read_bit(&mut self) -> Result<u8, Jbig2Error> {
        self.reader.read_bit()
    }

    fn set_run(&self, curr_line: &mut [u8], start: usize, end: usize, val: u8) {
        if val == 0 || start >= end {
            return;
        }

        // We only need to set bits to 1 because the line is initialized to 0.
        let mut idx = start;

        // Handle first partial byte
        if (idx & 7) != 0 {
            let byte_idx = idx >> 3;
            if byte_idx >= curr_line.len() {
                return;
            }

            let bits_in_byte = 8 - (idx & 7);
            let bits = bits_in_byte.min(end - idx);

            // Construct mask: 1s for the bits we want to set.
            // High bits are at lower indices (MSB first).
            // Shift 1s to the right position.
            // Example: idx&7=1 (start at bit 6), bits=3.
            // We want 01110000.
            // 0xFF >> (idx & 7) -> 01111111
            // 0xFF << (8 - ((idx&7) + bits)) -> 11110000 (wait)

            // Mask for high part starting at idx&7: 0xFF >> (idx&7)
            // Mask for low part ending at idx&7+bits: !(0xFF >> (idx&7+bits))

            let start_bit = idx & 7;
            let end_bit = start_bit + bits;

            let mask = (0xFF >> start_bit) & (0xFF << (8 - end_bit));

            curr_line[byte_idx] |= mask;
            idx += bits;
        }

        // Handle full bytes
        let start_byte = idx >> 3;
        if start_byte >= curr_line.len() {
            return;
        }

        let num_bytes = (end - idx) >> 3;
        if num_bytes > 0 {
            let end_byte = start_byte + num_bytes;
            let actual_end_byte = end_byte.min(curr_line.len());

            curr_line[start_byte..actual_end_byte].fill(0xFF);

            idx += (actual_end_byte - start_byte) << 3;

            if actual_end_byte == curr_line.len() {
                return;
            }
        }

        // Handle last partial byte
        if idx < end {
            let byte_idx = idx >> 3;
            if byte_idx >= curr_line.len() {
                return;
            }

            let bits = end - idx;
            // start_bit is 0 because we are aligned now (except if we started unaligned and finished in same byte, which is handled by first block)
            // wait, if we had a first partial block, idx is now aligned.

            let mask = 0xFF << (8 - bits);
            curr_line[byte_idx] |= mask;
        }
    }

    fn decode_2d_line(
        &mut self,
        ref_line: &[u8],
        curr_line: &mut [u8],
        _eofb: &mut bool,
    ) -> Result<(), Jbig2Error> {
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

            let b1 =
                self.find_changing_element_of_color(ref_line, a0, self.width, 1 - current_color);
            let b2 = self.find_changing_element(ref_line, b1 as i32, self.width);

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
                    // Fill from x to b2 with current_color
                    self.set_run(curr_line, x, b2, current_color);
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
                    self.set_run(curr_line, x, end, current_color);
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

                    let end1 = (x + r1).min(self.width);
                    self.set_run(curr_line, x, end1, current_color);
                    x += r1;

                    let end2 = (x + r2).min(self.width);
                    self.set_run(curr_line, x, end2, 1 - current_color);
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
        // Unrolled decision tree for faster decoding
        // V(0): 1
        if self.read_bit()? == 1 {
            return Ok(2);
        }

        // 0...
        // H: 001
        // VL(-1): 010
        // VR(1): 011
        if self.read_bit()? == 1 {
            // 01...
            if self.read_bit()? == 0 {
                return Ok(1); // VL(-1): 010
            } else {
                return Ok(3); // VR(1): 011
            }
        }

        // 00...
        if self.read_bit()? == 1 {
            return Ok(4); // H: 001
        }

        // 000...
        // Pass: 0001
        if self.read_bit()? == 1 {
            return Ok(0); // Pass: 0001
        }

        // 0000...
        // VL(-2): 000010
        // VR(2): 000011
        if self.read_bit()? == 1 {
            // 00001...
            if self.read_bit()? == 0 {
                return Ok(5); // VL(-2): 000010
            } else {
                return Ok(7); // VR(2): 000011
            }
        }

        // 00000...
        // VL(-3): 0000010
        // VR(3): 0000011
        if self.read_bit()? == 1 {
            // 000001...
            if self.read_bit()? == 0 {
                return Ok(6); // VL(-3): 0000010
            } else {
                return Ok(8); // VR(3): 0000011
            }
        }

        // 000000... -> Invalid/Error
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

        // Determine starting position x
        let mut x = if pos < 0 { 0 } else { pos as usize };

        if x >= width {
            return width;
        }

        // Determine the color we are looking for change FROM.
        // If pos < 0, we treat the virtual pixel at -1 as white (0).
        // So we are looking for the first pixel that is NOT 0.
        // If pos >= 0, we look for the first pixel that is NOT line[pos].

        let color_to_match = if pos < 0 {
            0
        } else {
            let byte = line[x >> 3];
            (byte >> (7 - (x & 7))) & 1
        };

        // We want to find the first pixel >= x that is != color_to_match.
        // Or conceptually, we skip pixels == color_to_match.

        // If color_to_match is 0, we search for 1.
        // If color_to_match is 1, we search for 0.

        // If pos >= 0, we advance x by 1 first as per original logic:
        // "x = x.saturating_add(1);"
        if pos >= 0 {
            x = x.saturating_add(1);
        }

        if x >= width {
            return width;
        }

        // Align to byte boundary
        while x < width && (x & 7) != 0 {
            let byte = line[x >> 3];
            let bit = (byte >> (7 - (x & 7))) & 1;
            if bit != color_to_match {
                return x;
            }
            x += 1;
        }

        if x >= width {
            return width;
        }

        // Process full bytes
        // If color_to_match == 0, we look for non-zero byte.
        // If color_to_match == 1, we look for non-0xFF byte.
        let target_byte = if color_to_match == 0 { 0x00 } else { 0xFF };

        let start_byte_idx = x >> 3;
        let limit_byte = (width + 7) >> 3;

        // Optimization: Scan for the first byte that is not target_byte
        if start_byte_idx < limit_byte {
            let slice = &line[start_byte_idx..limit_byte];
            let mut chunks = slice.chunks_exact(8);
            let target_chunk = if target_byte == 0 { 0u64 } else { !0u64 };
            let mut offset = 0;

            for chunk in chunks.by_ref() {
                // Check 8 bytes at once
                let val = u64::from_ne_bytes(chunk.try_into().unwrap());
                if val != target_chunk {
                    // Found mismatch in this chunk
                    for (i, &b) in chunk.iter().enumerate() {
                        if b != target_byte {
                            let byte_idx = start_byte_idx + offset + i;
                            let check_byte = if color_to_match == 0 { b } else { !b };
                            let bit_offset = check_byte.leading_zeros() as usize;
                            let result_x = (byte_idx << 3) + bit_offset;
                            return result_x.min(width);
                        }
                    }
                }
                offset += 8;
            }

            // Check remaining bytes
            for (i, &b) in chunks.remainder().iter().enumerate() {
                if b != target_byte {
                    let byte_idx = start_byte_idx + offset + i;
                    let check_byte = if color_to_match == 0 { b } else { !b };
                    let bit_offset = check_byte.leading_zeros() as usize;
                    let result_x = (byte_idx << 3) + bit_offset;
                    return result_x.min(width);
                }
            }
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

        // Check if the pixel at x is already `color`.
        // If x < width, line[x] is the new color (which is different from prev color).
        // If line[x] == color, we are done.
        // If line[x] != color, we need to find the next change.

        if x < width {
            let byte = line[x >> 3];
            let pixel_color = (byte >> (7 - (x & 7))) & 1;
            if pixel_color != color {
                x = self.find_changing_element(line, x as i32, width);
            }
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

        // Zero line for the first row reference
        let stride = bitmap.stride;
        let zero_line = vec![0u8; stride];

        for y in 0..self.height {
            if eofb {
                break;
            }

            // Determine ref_line and curr_line slices
            let (ref_slice, curr_slice) = if y == 0 {
                let curr = &mut bitmap.data[0..stride];
                (zero_line.as_slice(), curr)
            } else {
                let (prev, curr) = bitmap.data.split_at_mut(y * stride);
                let ref_row = &prev[(y - 1) * stride..];
                let curr_row = &mut curr[..stride];
                (ref_row as &[u8], curr_row)
            };

            self.decode_2d_line(ref_slice, curr_slice, &mut eofb)?;
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
