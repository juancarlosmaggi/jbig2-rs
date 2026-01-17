//! Test fixtures and data generators for creating test data.

use jbig2_rs::bitmap::Bitmap;

/// Create an empty bitmap with the given dimensions.
pub fn simple_bitmap(width: usize, height: usize, _fill_value: u8) -> Bitmap {
    Bitmap::new(width, height)
}

/// Create a checkerboard pattern bitmap.
pub fn checkerboard_bitmap(width: usize, height: usize) -> Bitmap {
    let mut bitmap = Bitmap::new(width, height);

    for y in 0..height {
        for x in 0..width {
            if (x + y) % 2 == 0 {
                bitmap.set_pixel(x, y, 1);
            }
        }
    }

    bitmap
}

/// Create a bitmap with a border of set pixels.
pub fn bordered_bitmap(width: usize, height: usize, border_width: usize) -> Bitmap {
    let mut bitmap = Bitmap::new(width, height);

    for y in 0..height {
        for x in 0..width {
            if x < border_width
                || x >= width - border_width
                || y < border_width
                || y >= height - border_width
            {
                bitmap.set_pixel(x, y, 1);
            }
        }
    }

    bitmap
}

/// Create a minimal JBIG2 file header byte sequence.
pub fn valid_jbig2_header() -> Vec<u8> {
    vec![
        // File ID string
        0x97, 0x4A, 0x42, 0x32, 0x0D, 0x0A, 0x1A, 0x0A,
        // File organization flags (sequential)
        0x01, // Number of pages (unknown)
        0x00,
    ]
}

/// Create a basic segment header byte sequence.
pub fn segment_header(segment_number: u32, segment_type: u8, data_length: u32) -> Vec<u8> {
    let mut header = Vec::new();

    // Segment number (4 bytes)
    header.extend_from_slice(&segment_number.to_be_bytes());

    // Flags byte (no referred segments)
    header.push(0x00);

    // Segment type
    header.push(segment_type);

    // Page association (page 0)
    header.push(0x00);

    // Data length (4 bytes)
    header.extend_from_slice(&data_length.to_be_bytes());

    header
}

/// Create a bitmap that transitions from 0 to 1 at the given threshold.
pub fn gradient_bitmap(width: usize, height: usize, threshold: usize) -> Bitmap {
    let mut bitmap = Bitmap::new(width, height);

    for y in 0..height {
        for x in 0..width {
            if x >= threshold {
                bitmap.set_pixel(x, y, 1);
            }
        }
    }

    bitmap
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_bitmap() {
        let bitmap = simple_bitmap(10, 10, 1);
        assert_eq!(bitmap.width, 10);
        assert_eq!(bitmap.height, 10);
        // simple_bitmap creates an empty bitmap.
        assert_eq!(bitmap.get_pixel(0, 0), 0);
        assert_eq!(bitmap.get_pixel(9, 9), 0);
    }

    #[test]
    fn test_checkerboard_bitmap() {
        let bitmap = checkerboard_bitmap(4, 4);
        assert_eq!(bitmap.get_pixel(0, 0), 1); // (0+0) % 2 == 0
        assert_eq!(bitmap.get_pixel(1, 0), 0); // (1+0) % 2 == 1
        assert_eq!(bitmap.get_pixel(0, 1), 0); // (0+1) % 2 == 1
        assert_eq!(bitmap.get_pixel(1, 1), 1); // (1+1) % 2 == 0
    }

    #[test]
    fn test_bordered_bitmap() {
        let bitmap = bordered_bitmap(10, 10, 2);
        // Corners should be in the border.
        assert_eq!(bitmap.get_pixel(0, 0), 1);
        assert_eq!(bitmap.get_pixel(1, 1), 1);
        // Center should be interior.
        assert_eq!(bitmap.get_pixel(5, 5), 0);
    }

    #[test]
    fn test_valid_jbig2_header() {
        let header = valid_jbig2_header();
        assert_eq!(header.len(), 10);
        // Check the magic bytes.
        assert_eq!(
            &header[0..8],
            &[0x97, 0x4A, 0x42, 0x32, 0x0D, 0x0A, 0x1A, 0x0A]
        );
    }

    #[test]
    fn test_segment_header() {
        let header = segment_header(1, 0x30, 100);
        assert!(!header.is_empty());
        // Segment type should be at the expected position.
        assert_eq!(header[5], 0x30);
    }

    #[test]
    fn test_gradient_bitmap() {
        let bitmap = gradient_bitmap(10, 5, 5);
        assert_eq!(bitmap.get_pixel(4, 0), 0);
        assert_eq!(bitmap.get_pixel(5, 0), 1);
        assert_eq!(bitmap.get_pixel(9, 4), 1);
    }
}
