use crate::bitmap::Bitmap;
use crate::common::error::Jbig2Error;
use crate::common::reader::Reader;

/// Read a raw 1bpp bitmap from the bitstream, row by row.
pub fn read_uncompressed_bitmap(
    reader: &mut Reader<'_>,
    width: usize,
    height: usize,
) -> Result<Bitmap, Jbig2Error> {
    let mut bitmap = Bitmap::new(width, height);
    if width == 0 || height == 0 {
        return Ok(bitmap);
    }

    let stride = bitmap.stride;

    // Fast path for byte-aligned reads
    if reader.get_shift() == -1 {
        let total_bytes = match height.checked_mul(stride) {
            Some(v) => v,
            None => return Err(Jbig2Error::new("bitmap size overflow")),
        };

        let start_pos = reader.get_position();
        let end_pos = reader.get_end();

        if start_pos + total_bytes <= end_pos {
            let data = reader.get_data();
            let src_slice = &data[start_pos..start_pos + total_bytes];

            let rem_bits = width % 8;
            let mask = if rem_bits == 0 {
                0xFF
            } else {
                0xFFu8 << (8 - rem_bits)
            };

            // Optimization: if the bitmap data is contiguous (which it is for new bitmaps),
            // we can copy the whole block at once if no masking is needed.
            // But we need to mask the last byte of each row if rem_bits != 0.
            // Since we iterate anyway for masking, row-by-row copy is fine and cache-friendly.

            for y in 0..height {
                let row_offset = y * stride;
                let src_row = &src_slice[row_offset..row_offset + stride];
                let dst_row = &mut bitmap.data[row_offset..row_offset + stride];

                dst_row.copy_from_slice(src_row);

                // Mask the last byte if needed to ensure padding bits are zero
                if rem_bits != 0 {
                    dst_row[stride - 1] &= mask;
                }
            }

            reader.set_position(start_pos + total_bytes);
            return Ok(bitmap);
        }
    }

    for y in 0..height {
        for x in 0..width {
            let pixel = reader.read_bit()?;
            bitmap.set_pixel(x, y, pixel);
        }
        reader.byte_align();
    }
    Ok(bitmap)
}
