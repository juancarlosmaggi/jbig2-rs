use crate::bitmap::Bitmap;
use crate::error::Jbig2Error;
use crate::reader::Reader;

/// Read a raw 1bpp bitmap from the bitstream, row by row.
pub fn read_uncompressed_bitmap(
    reader: &mut Reader,
    width: usize,
    height: usize,
) -> Result<Bitmap, Jbig2Error> {
    let mut bitmap = Bitmap::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let pixel = reader.read_bit()?;
            bitmap.set_pixel(x, y, pixel);
        }
        reader.byte_align();
    }
    Ok(bitmap)
}
