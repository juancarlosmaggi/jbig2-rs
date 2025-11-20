pub mod arithmetic;
pub mod bitmap;
pub mod bitmap_utils;
pub mod contexts;
pub mod core_utils;
pub mod decode;
pub mod decoder;
pub mod error;
pub mod huffman;
pub mod image;
pub mod reader;
pub mod segment;
pub mod validation;
pub mod visitor;
pub use error::Jbig2Error;
pub use image::{Jbig2Document, Jbig2Image};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jbig2_document_creation() {
        let doc = Jbig2Document::new();
        assert_eq!(doc.page_count(), 0);
    }

    #[test]
    fn test_jbig2_header_validation() {
        // Invalid header (too short)
        let invalid_data = b"\x00\x00\x00\x00";
        assert!(Jbig2Document::parse(invalid_data).is_err());

        // Invalid header (wrong magic)
        let invalid_data2 = b"\x00\x4a\x42\x32\x0d\x0a\x1a\x0a\x00\x00\x00\x00";
        assert!(Jbig2Document::parse(invalid_data2).is_err());
    }

    #[test]
    fn test_bitmap_operations() {
        let mut bitmap = crate::bitmap::Bitmap::new(10, 10);
        bitmap.set_pixel(5, 5, 1);
        assert_eq!(bitmap.get_pixel(5, 5), 1);
        assert_eq!(bitmap.get_pixel(0, 0), 0);

        // Test bounds checking
        assert_eq!(bitmap.get_pixel(15, 15), 0); // Out of bounds
        bitmap.set_pixel(15, 15, 1); // Should not panic
    }

    #[test]
    fn test_huffman_tables() {
        // Test standard table retrieval
        let table1 = crate::huffman::get_standard_table(1);
        assert!(table1.is_ok());

        let table_invalid = crate::huffman::get_standard_table(999);
        assert!(table_invalid.is_err());
    }

    #[test]
    fn test_segment_header_parsing() {
        // Minimal valid segment header data
        let data = vec![
            0x00, 0x00, 0x00, 0x01, // segment number
            0x00, // flags (type 0)
            0x00, // referred flags
            0x00, 0x00, 0x00, 0x00, // page association
            0x00, 0x00, 0x00, 0x00, // length
        ];
        let header = crate::segment::read_segment_header(&data, 0, false);
        assert!(header.is_ok());
        let header = header.unwrap();
        assert_eq!(header.segment_type, 0);
        assert_eq!(header.number, 1);
    }

    #[test]
    fn test_draw_symbol_at_position() {
        let mut bitmap = crate::bitmap::Bitmap::new(10, 10);
        let mut symbol = crate::bitmap::Bitmap::new(2, 2);
        symbol.set_pixel(0, 0, 1);
        symbol.set_pixel(1, 1, 1);
        crate::bitmap_utils::draw_symbol_at_position(&mut bitmap, &symbol, 1, 1, 0);
        assert_eq!(bitmap.get_pixel(1, 1), 1);
        assert_eq!(bitmap.get_pixel(2, 2), 1);
        assert_eq!(bitmap.get_pixel(0, 0), 0);
    }
}

