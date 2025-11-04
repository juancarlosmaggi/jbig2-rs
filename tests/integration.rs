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
        assert!(
            doc.page_count() > 0,
            "Document should have at least one page"
        );
        let page = doc.get_page(0).unwrap();
        let _decoded = page.to_image_data(); // Just check it decodes without error
    }

    #[test]
    fn test_parse_invalid_data() {
        // Test with invalid data
        let data = vec![0u8; 10];
        let result = Jbig2Image::parse(&data);
        assert!(result.is_err()); // Should fail with invalid data
    }
}
