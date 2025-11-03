pub mod error;
pub mod core_utils;
pub mod reader;
pub mod huffman;
pub mod bitmap;
pub mod decoder;
pub mod arithmetic;
pub mod contexts;
pub mod segment;
pub mod visitor;
pub mod image;
pub mod decode_generic;
pub mod decode_text;
pub mod decode_symbol;
pub mod decode_pattern;
pub mod decode_halftone;
pub mod decode_refinement;
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