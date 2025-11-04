#[cfg(test)]
mod tests {
    use jbig2_rs::image::{Jbig2Document, Jbig2Image};
    use std::fs;

    fn load_file(path: &str) -> Vec<u8> {
        fs::read(path).expect(&format!("Failed to read file: {}", path))
    }

    #[test]
    fn test_minimal_valid() {
        let jbig2_data = load_file("tests/resources/minimal_valid.jb2");
        let doc = Jbig2Document::parse(&jbig2_data).expect("Failed to parse JBIG2");
        assert_eq!(doc.page_count(), 1);
        // Note: to_image_data may fail due to large dimensions in test file
    }

    #[test]
    fn test_halftone_region() {
        let jbig2_data = load_file("tests/resources/halftone_region.jb2");
        let doc = Jbig2Document::parse(&jbig2_data).expect("Failed to parse JBIG2");
        assert!(doc.page_count() > 0);
    }

    #[test]
    fn test_symbol_dictionary() {
        let jbig2_data = load_file("tests/resources/symbol_dictionary.jb2");
        let doc = Jbig2Document::parse(&jbig2_data).expect("Failed to parse JBIG2");
        assert!(doc.page_count() > 0);
    }

    #[test]
    fn test_text_region() {
        let jbig2_data = load_file("tests/resources/text_region.jb2");
        let doc = Jbig2Document::parse(&jbig2_data).expect("Failed to parse JBIG2");
        assert!(doc.page_count() > 0);
    }

    #[test]
    fn test_parse_invalid_data() {
        // Test with invalid data
        let data = vec![0u8; 10];
        let result = Jbig2Image::parse(&data);
        assert!(result.is_err()); // Should fail with invalid data
    }
}
