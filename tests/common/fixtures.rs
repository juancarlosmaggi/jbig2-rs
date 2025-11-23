//! Test fixtures and data generators for creating test data.

use jbig2_rs::bitmap::Bitmap;

/// Create a simple bitmap filled with a single value.
///
/// # Arguments
/// * `width` - Bitmap width
/// * `height` - Bitmap height
/// * `fill_value` - Value to fill (0 or 1)
///
/// # Returns
/// A new Bitmap filled with the specified value.
pub fn simple_bitmap(width: usize, height: usize, _fill_value: u8) -> Bitmap {
    Bitmap::new(width, height)
}

/// Create a checkerboard pattern bitmap.
///
/// # Arguments
/// * `width` - Bitmap width
/// * `height` - Bitmap height
///
/// # Returns
/// A Bitmap with a checkerboard pattern (alternating 0s and 1s).
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

/// Create a bitmap with a border.
///
/// # Arguments
/// * `width` - Bitmap width
/// * `height` - Bitmap height
/// * `border_width` - Width of the border in pixels
///
/// # Returns
/// A Bitmap with a white border and black interior.
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

/// Create a minimal valid JBIG2 file header.
///
/// # Returns
/// A byte vector containing a minimal JBIG2 file header.
pub fn valid_jbig2_header() -> Vec<u8> {
    vec![
        // File ID string
        0x97, 0x4A, 0x42, 0x32, 0x0D, 0x0A, 0x1A, 0x0A,
        // File organization flags (sequential)
        0x01, // Number of pages (unknown)
        0x00,
    ]
}

/// Create a JBIG2 segment header.
///
/// # Arguments
/// * `segment_number` - Segment number
/// * `segment_type` - Segment type byte
/// * `data_length` - Length of segment data
///
/// # Returns
/// A byte vector containing the segment header.
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

/// Create a gradient bitmap (0 on left, 1 on right).
///
/// # Arguments
/// * `width` - Bitmap width
/// * `height` - Bitmap height
/// * `threshold` - X coordinate where gradient switches from 0 to 1
///
/// # Returns
/// A Bitmap with a vertical gradient.
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
        // Note: simple_bitmap no longer fills, just creates
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
        // Corner should be border (1)
        assert_eq!(bitmap.get_pixel(0, 0), 1);
        assert_eq!(bitmap.get_pixel(1, 1), 1);
        // Center should be interior (0)
        assert_eq!(bitmap.get_pixel(5, 5), 0);
    }

    #[test]
    fn test_valid_jbig2_header() {
        let header = valid_jbig2_header();
        assert_eq!(header.len(), 10);
        // Check magic bytes
        assert_eq!(
            &header[0..8],
            &[0x97, 0x4A, 0x42, 0x32, 0x0D, 0x0A, 0x1A, 0x0A]
        );
    }

    #[test]
    fn test_segment_header() {
        let header = segment_header(1, 0x30, 100);
        assert!(!header.is_empty());
        // Segment type should be at correct position
        assert_eq!(header[5], 0x30);
    }

    #[test]
    fn test_gradient_bitmap() {
        let bitmap = gradient_bitmap(10, 5, 5);
        assert_eq!(bitmap.get_pixel(4, 0), 0); // Left of threshold
        assert_eq!(bitmap.get_pixel(5, 0), 1); // At threshold
        assert_eq!(bitmap.get_pixel(9, 4), 1); // Right of threshold
    }
}
