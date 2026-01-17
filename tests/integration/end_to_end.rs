use crate::common::load_test_file;
use jbig2_rs::image::{Jbig2Document, Jbig2Image};

#[test]
fn test_minimal_valid() {
    let jbig2_data = load_test_file("minimal_valid.jb2");
    let doc = Jbig2Document::parse(&jbig2_data).expect("Failed to parse JBIG2");
    assert_eq!(doc.page_count(), 1);
}

#[test]
fn test_halftone_region() {
    let jbig2_data = load_test_file("halftone_region.jb2");
    let doc = Jbig2Document::parse(&jbig2_data).expect("Failed to parse JBIG2");
    assert!(doc.page_count() > 0);
}

#[test]
fn test_symbol_dictionary() {
    let jbig2_data = load_test_file("symbol_dictionary.jb2");
    let doc = Jbig2Document::parse(&jbig2_data).expect("Failed to parse JBIG2");
    assert!(doc.page_count() > 0);
}

#[test]
fn test_text_region() {
    let jbig2_data = load_test_file("text_region.jb2");
    let doc = Jbig2Document::parse(&jbig2_data).expect("Failed to parse JBIG2");
    assert!(doc.page_count() > 0);
}

#[test]
fn test_parse_invalid_data() {
    let data = vec![0u8; 10];
    let result = Jbig2Image::parse(&data);
    assert!(result.is_err());
}
