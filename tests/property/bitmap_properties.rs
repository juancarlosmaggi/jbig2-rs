use jbig2_rs::bitmap::Bitmap;
use proptest::prelude::*;

proptest! {
    /// Bitmap dimensions should match the requested size.
    #[test]
    fn prop_bitmap_dimensions(width in 1usize..200, height in 1usize..200) {
        let bitmap = Bitmap::new(width, height);
        prop_assert_eq!(bitmap.width, width);
        prop_assert_eq!(bitmap.height, height);
    }

    /// New bitmaps should start with all pixels cleared.
    #[test]
    fn prop_new_bitmap_is_zero(width in 1usize..100, height in 1usize..100) {
        let bitmap = Bitmap::new(width, height);
        for y in 0..height {
            for x in 0..width {
                prop_assert_eq!(bitmap.get_pixel(x, y), 0);
            }
        }
    }

    /// set_pixel followed by get_pixel should round-trip the value.
    #[test]
    fn prop_set_get_pixel_roundtrip(
        width in 10usize..50,
        height in 10usize..50,
        x in 0usize..10,
        y in 0usize..10,
        value in 0u8..=1
    ) {
        let mut bitmap = Bitmap::new(width, height);
        bitmap.set_pixel(x, y, value);
        prop_assert_eq!(bitmap.get_pixel(x, y), value);
    }

    /// Setting one pixel should not affect others.
    #[test]
    fn prop_set_pixel_isolation(
        width in 20usize..50,
        height in 20usize..50,
        x1 in 0usize..10,
        y1 in 0usize..10,
    ) {
        let mut bitmap = Bitmap::new(width, height);
        bitmap.set_pixel(x1, y1, 1);

        // Check that only the target pixel is set
        for y in 0..height {
            for x in 0..width {
                if x == x1 && y == y1 {
                    prop_assert_eq!(bitmap.get_pixel(x, y), 1);
                } else {
                    prop_assert_eq!(bitmap.get_pixel(x, y), 0);
                }
            }
        }
    }

    /// Out-of-bounds get_pixel should return 0.
    #[test]
    fn prop_get_pixel_oob_returns_zero(
        width in 1usize..50,
        height in 1usize..50,
        x_offset in 0usize..50,
        y_offset in 0usize..50
    ) {
        let bitmap = Bitmap::new(width, height);
        let x = width + x_offset;
        let y = height + y_offset;
        prop_assert_eq!(bitmap.get_pixel(x, y), 0);
    }

    /// Out-of-bounds set_pixel should not panic.
    #[test]
    fn prop_set_pixel_oob_no_panic(
        width in 1usize..50,
        height in 1usize..50,
        x_offset in 0usize..50,
        y_offset in 0usize..50,
        value in 0u8..=1
    ) {
        let mut bitmap = Bitmap::new(width, height);
        let x = width + x_offset;
        let y = height + y_offset;
        bitmap.set_pixel(x, y, value); // Should not panic
    }

    /// OR combine should be commutative.
    #[test]
    fn prop_combine_or_commutative(
        width in 10usize..30,
        height in 10usize..30,
        pixels1 in prop::collection::vec(any::<u8>(), 10..100),
        pixels2 in prop::collection::vec(any::<u8>(), 10..100),
    ) {
        let mut bm1a = Bitmap::new(width, height);
        let mut bm1b = Bitmap::new(width, height);
        let mut bm2 = Bitmap::new(width, height);
        let mut bm3 = Bitmap::new(width, height);

        // Set some pixels in bm1 and bm2
        for (i, (&p1, &p2)) in pixels1.iter().zip(pixels2.iter()).enumerate() {
            let x = i % width;
            let y = i / width;
            if y >= height { break; }
            bm1a.set_pixel(x, y, p1 & 1);
            bm1b.set_pixel(x, y, p1 & 1);
            bm2.set_pixel(x, y, p2 & 1);
            bm3.set_pixel(x, y, p2 & 1);
        }

        // A | B
        bm1a.combine(&bm2, 0, 0, 0);
        // B | A
        bm3.combine(&bm1b, 0, 0, 0);

        // Should be equal
        for y in 0..height {
            for x in 0..width {
                prop_assert_eq!(bm1a.get_pixel(x, y), bm3.get_pixel(x, y));
            }
        }
    }

    /// AND with itself should preserve the bitmap.
    #[test]
    fn prop_combine_and_idempotent(
        width in 10usize..30,
        height in 10usize..30,
        pixels in prop::collection::vec(any::<u8>(), 10..100),
    ) {
        let mut bm = Bitmap::new(width, height);

        // Set some pixels
        for (i, &p) in pixels.iter().enumerate() {
            let x = i % width;
            let y = i / width;
            if y >= height { break; }
            bm.set_pixel(x, y, p & 1);
        }

        let original = bm.clone();
        let bm_copy = bm.clone(); // Clone AFTER setting pixels

        // A & A should equal A
        bm.combine(&bm_copy, 0, 0, 1);

        for y in 0..height {
            for x in 0..width {
                prop_assert_eq!(bm.get_pixel(x, y), original.get_pixel(x, y));
            }
        }
    }

    /// AND with a zero bitmap should clear all pixels.
    #[test]
    fn prop_combine_and_with_zero(
        width in 10usize..30,
        height in 10usize..30,
        pixels in prop::collection::vec(any::<u8>(), 10..100),
    ) {
        let mut bm = Bitmap::new(width, height);
        let zero_bm = Bitmap::new(width, height);

        // Set some pixels in bm
        for (i, &p) in pixels.iter().enumerate() {
            let x = i % width;
            let y = i / width;
            if y >= height { break; }
            bm.set_pixel(x, y, p & 1);
        }

        // A & 0 should equal 0
        bm.combine(&zero_bm, 0, 0, 1);

        for y in 0..height {
            for x in 0..width {
                prop_assert_eq!(bm.get_pixel(x, y), 0);
            }
        }
    }
}
