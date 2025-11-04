#[cfg(test)]
mod tests {
    use jbig2_rs::image::{Jbig2Document, Jbig2Image};
    use std::fs;

    fn load_file(path: &str) -> Vec<u8> {
        fs::read(path).expect(&format!("Failed to read file: {}", path))
    }

    fn compare_images(decoded: &[u8], expected_png_path: &str) {
        let expected_img = image::open(expected_png_path)
            .expect(&format!("Failed to load PNG: {}", expected_png_path))
            .to_rgb8();
        let expected_data = expected_img.as_raw();

        // Convert RGB to grayscale (take R channel since JBIG2 is monochrome)
        let mut expected_gray = Vec::with_capacity(expected_data.len() / 3);
        for chunk in expected_data.chunks(3) {
            expected_gray.push(chunk[0]); // Use red channel as grayscale
        }

        assert_eq!(decoded.len(), expected_gray.len(),
            "Image dimensions don't match: decoded {} bytes, expected {} bytes",
            decoded.len(), expected_gray.len());

        // Compare pixel by pixel
        let mut differences = 0;
        for (i, (&decoded_pixel, &expected_pixel)) in decoded.iter().zip(expected_gray.iter()).enumerate() {
            if decoded_pixel != expected_pixel {
                differences += 1;
                if differences <= 10 { // Only print first few differences
                    println!("Pixel {} differs: decoded={}, expected={}", i, decoded_pixel, expected_pixel);
                }
            }
        }

        if differences > 0 {
            panic!("Images differ in {} pixels out of {}", differences, decoded.len());
        }
    }

    #[test]
    fn test_weight_no_jbig2() {
        let jbig2_data = load_file("tests/resources/weight_no_jbig2.jb2");
        let doc = Jbig2Document::parse(&jbig2_data).expect("Failed to parse JBIG2");
        assert!(doc.page_count() > 0, "Document should have at least one page");
        let page = doc.get_page(0).unwrap();
        let decoded = page.to_image_data();
        compare_images(&decoded, "tests/resources/weight_no_jbig2.png");
    }

    #[test]
    fn test_weight_t085_w025() {
        let jbig2_data = load_file("tests/resources/weight_t085_w025.jb2");
        let doc = Jbig2Document::parse(&jbig2_data).expect("Failed to parse JBIG2");
        assert!(doc.page_count() > 0, "Document should have at least one page");
        let page = doc.get_page(0).unwrap();
        let decoded = page.to_image_data();
        compare_images(&decoded, "tests/resources/weight_t085_w025.png");
    }

    #[test]
    fn test_weight_t085_w050() {
        let jbig2_data = load_file("tests/resources/weight_t085_w050.jb2");
        let doc = Jbig2Document::parse(&jbig2_data).expect("Failed to parse JBIG2");
        assert!(doc.page_count() > 0, "Document should have at least one page");
        let page = doc.get_page(0).unwrap();
        let decoded = page.to_image_data();
        compare_images(&decoded, "tests/resources/weight_t085_w050.png");
    }

    #[test]
    fn test_weight_t085_w075() {
        let jbig2_data = load_file("tests/resources/weight_t085_w075.jb2");
        let doc = Jbig2Document::parse(&jbig2_data).expect("Failed to parse JBIG2");
        assert!(doc.page_count() > 0, "Document should have at least one page");
        let page = doc.get_page(0).unwrap();
        let decoded = page.to_image_data();
        compare_images(&decoded, "tests/resources/weight_t085_w075.png");
    }

    #[test]
    fn test_parse_invalid_data() {
        // Test with invalid data
        let data = vec![0u8; 10];
        let result = Jbig2Image::parse(&data);
        assert!(result.is_err()); // Should fail with invalid data
    }
}
