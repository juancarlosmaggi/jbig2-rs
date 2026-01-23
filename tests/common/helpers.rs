//! Helper functions for test assertions and common operations.

use jbig2_rs::bitmap::Bitmap;
use jbig2_rs::common::reader::Reader;
use std::fs;

/// Load a test fixture from `tests/resources`.
pub fn load_test_file(filename: &str) -> Vec<u8> {
    let path = format!("tests/resources/{}", filename);
    fs::read(&path).unwrap_or_else(|_| panic!("Failed to read test file: {}", path))
}

/// Assert that two bitmaps are identical in size and pixels.
#[allow(dead_code)]
pub fn assert_bitmap_equals(actual: &Bitmap, expected: &Bitmap) {
    assert_eq!(actual.width, expected.width, "Bitmap widths differ");
    assert_eq!(actual.height, expected.height, "Bitmap heights differ");

    for y in 0..actual.height {
        for x in 0..actual.width {
            let actual_pixel = actual.get_pixel(x, y);
            let expected_pixel = expected.get_pixel(x, y);
            assert_eq!(
                actual_pixel, expected_pixel,
                "Pixel mismatch at ({}, {}): expected {}, got {}",
                x, y, expected_pixel, actual_pixel
            );
        }
    }
}

/// Assert that all pixels in the given ranges match the expected value.
pub fn assert_pixel_range(
    bitmap: &Bitmap,
    x_range: (usize, usize),
    y_range: (usize, usize),
    expected_value: u8,
) {
    let (x_start, x_end) = x_range;
    let (y_start, y_end) = y_range;

    for y in y_start..=y_end.min(bitmap.height.saturating_sub(1)) {
        for x in x_start..=x_end.min(bitmap.width.saturating_sub(1)) {
            let pixel = bitmap.get_pixel(x, y);
            assert_eq!(
                pixel, expected_value,
                "Pixel at ({}, {}) should be {}, got {}",
                x, y, expected_value, pixel
            );
        }
    }
}

/// Create a Reader over the full test buffer.
pub fn create_test_reader(data: Vec<u8>) -> Reader<'static> {
    let len = data.len();
    Reader::new(data, 0, len)
}

/// Print a hex dump of data for debugging.
pub fn hex_dump(data: &[u8], max_bytes: usize) {
    let bytes_to_show = data.len().min(max_bytes);
    print!("Hex dump ({} bytes): ", bytes_to_show);

    for (i, byte) in data.iter().take(bytes_to_show).enumerate() {
        if i > 0 && i % 16 == 0 {
            println!();
            print!("  ");
        }
        print!("{:02x} ", byte);
    }

    if data.len() > max_bytes {
        print!("... ({} more bytes)", data.len() - max_bytes);
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_reader() {
        let data = vec![1, 2, 3, 4, 5];
        let _reader = create_test_reader(data);
    }

    #[test]
    fn test_assert_pixel_range() {
        let bitmap = Bitmap::new(10, 10);
        assert_pixel_range(&bitmap, (0, 9), (0, 9), 0);
    }

    #[test]
    fn test_hex_dump_doesnt_panic() {
        let data = vec![0xaa, 0xbb, 0xcc, 0xdd];
        hex_dump(&data, 10);
        hex_dump(&data, 2);
    }
}
