use crate::bitmap::Bitmap;
use crate::common::error::Jbig2Error;
use crate::document::core::Jbig2Document;
use crate::document::info::PageInfo;

#[derive(Clone)]
pub struct Jbig2Page {
    pub page_info: PageInfo,
    pub bitmap: Bitmap,
    pub bit_packed_data: Vec<u8>,
}

impl Jbig2Page {
    /// Return the number of packed bytes in each bitmap row.
    pub fn stride(&self) -> usize {
        self.page_info.width.div_ceil(8) as usize
    }

    /// Return packed 1bpp bitmap bytes, MSB-first within each byte.
    ///
    /// A bit value of `1` represents black/foreground and `0` represents
    /// white/background. Rows are padded to byte boundaries using zero bits.
    pub fn packed_bitmap(&self) -> &[u8] {
        &self.bit_packed_data
    }

    /// Expand the packed bitmap into 8-bit grayscale pixels.
    pub fn to_grayscale_image_data(&self) -> Vec<u8> {
        let width = self.page_info.width as usize;
        let height = self.page_info.height as usize;
        let mut img_data = vec![0u8; width * height];
        let row_size = width.div_ceil(8);
        for y in 0..height {
            for x in 0..width {
                let byte_index = y * row_size + (x / 8);
                let bit_index = 7 - (x % 8);
                // Map packed bits into grayscale pixels.
                let pixel = if (self.bit_packed_data[byte_index] & (1 << bit_index)) != 0 {
                    0
                } else {
                    255
                };
                img_data[y * width + x] = pixel;
            }
        }
        img_data
    }

    /// Expand the packed bitmap into 8-bit grayscale pixels.
    ///
    /// This legacy helper returns one byte per pixel (`0` for black, `255` for
    /// white). Use [`Self::packed_bitmap`] for embedders that need packed 1bpp
    /// bytes.
    pub fn to_image_data(&self) -> Vec<u8> {
        self.to_grayscale_image_data()
    }
}

/// Type alias for a single JBIG2 page (backward compatibility).
///
/// Prefer [`Jbig2Document`] and [`Jbig2Document::get_page`] in new code.
pub type Jbig2Image = Jbig2Page;

impl Jbig2Image {
    /// Parse JBIG2 data and return the first page's grayscale image data.
    ///
    /// This helper assumes a single-page document.
    ///
    /// # Returns
    ///
    /// 8-bit grayscale data as a vector of bytes, one byte per pixel.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use jbig2_rs::Jbig2Image;
    ///
    /// let data = std::fs::read("page.jb2").unwrap();
    /// let image_data = Jbig2Image::parse(&data)?;
    /// # Ok::<(), jbig2_rs::Jbig2Error>(())
    /// ```
    pub fn parse(data: &[u8]) -> Result<Vec<u8>, Jbig2Error> {
        let doc = Jbig2Document::parse(data)?;
        if let Some(page) = doc.get_page(0) {
            Ok(page.to_image_data())
        } else {
            Err(Jbig2Error::new("no pages in document"))
        }
    }
}
