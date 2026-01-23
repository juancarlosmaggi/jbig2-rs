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
    /// Expand the packed bitmap into 8-bit grayscale pixels.
    pub fn to_image_data(&self) -> Vec<u8> {
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
}

/// Type alias for a single JBIG2 page (backward compatibility).
///
/// Prefer [`Jbig2Document`] and [`Jbig2Document::get_page`] in new code.
pub type Jbig2Image = Jbig2Page;

impl Jbig2Image {
    /// Parse JBIG2 data and return the first page's image data.
    ///
    /// This helper assumes a single-page document.
    ///
    /// # Returns
    ///
    /// Raw bitmap data as a vector of bytes (1 bit per pixel, packed).
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
