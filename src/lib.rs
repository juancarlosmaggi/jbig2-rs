pub mod error;
pub mod core_utils;
pub mod reader;
pub mod huffman;
pub mod bitmap;
pub mod decoder;
pub mod segment;
pub mod image;

pub use error::Jbig2Error;
pub use image::Jbig2Image;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jbig2_image_creation() {
        let image = Jbig2Image::new();
        assert_eq!(image.width, 0);
        assert_eq!(image.height, 0);
    }
}
