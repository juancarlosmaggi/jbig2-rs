use crate::common::create_test_reader;
use jbig2_rs::decode::decode_mmr::decode_mmr_bitmap;

#[test]
fn test_decode_mmr_simple() {
    let data = vec![0u8; 10];
    let mut reader = create_test_reader(data);
    let result = decode_mmr_bitmap(&mut reader, 8, 8, false);
    assert!(result.is_ok());
    let bitmap = result.unwrap();
    assert_eq!(bitmap.width, 8);
    assert_eq!(bitmap.height, 8);
    for y in 0..8 {
        for x in 0..8 {
            assert_eq!(bitmap.get_pixel(x, y), 0);
        }
    }
}

#[test]
fn test_decode_mmr_zero_dimensions() {
    let data = vec![0u8; 10];

    // Zero width
    let mut reader = create_test_reader(data.clone());
    let result = decode_mmr_bitmap(&mut reader, 0, 8, false);
    assert!(result.is_ok());
    let bitmap = result.unwrap();
    assert_eq!(bitmap.width, 0);
    assert_eq!(bitmap.height, 8);

    // Zero height
    let mut reader = create_test_reader(data);
    let result = decode_mmr_bitmap(&mut reader, 8, 0, false);
    assert!(result.is_ok());
    let bitmap = result.unwrap();
    assert_eq!(bitmap.width, 8);
    assert_eq!(bitmap.height, 0);
}

#[test]
fn test_decode_mmr_empty_data() {
    let data = vec![];
    let mut reader = create_test_reader(data);
    let result = decode_mmr_bitmap(&mut reader, 8, 8, false);
    assert!(result.is_ok());
    let bitmap = result.unwrap();
    assert_eq!(bitmap.width, 8);
    assert_eq!(bitmap.height, 8);
    for y in 0..8 {
        for x in 0..8 {
            assert_eq!(bitmap.get_pixel(x, y), 0);
        }
    }
}

#[test]
fn test_decode_mmr_insufficient_data() {
    let data = vec![0xFF];
    let mut reader = create_test_reader(data);
    let result = decode_mmr_bitmap(&mut reader, 64, 64, false);
    assert!(result.is_ok());
    let bitmap = result.unwrap();
    assert_eq!(bitmap.width, 64);
    assert_eq!(bitmap.height, 64);
    // Spot check zeros
    assert_eq!(bitmap.get_pixel(0, 0), 0);
    assert_eq!(bitmap.get_pixel(63, 0), 0);
    assert_eq!(bitmap.get_pixel(0, 63), 0);
    assert_eq!(bitmap.get_pixel(63, 63), 0);
}

#[test]
fn test_decode_mmr_end_of_block() {
    let data = vec![0u8; 100];
    let mut reader = create_test_reader(data);
    let result = decode_mmr_bitmap(&mut reader, 16, 16, true);
    assert!(result.is_ok());
    let bitmap = result.unwrap();
    assert_eq!(bitmap.width, 16);
    assert_eq!(bitmap.height, 16);
    for y in 0..16 {
        for x in 0..16 {
            assert_eq!(bitmap.get_pixel(x, y), 0);
        }
    }
}

#[test]
fn test_decode_mmr_different_sizes() {
    let test_cases = vec![
        (1, 1),
        (8, 8),
        (16, 16),
        (32, 32),
        (64, 64),
        (100, 50),
        (50, 100),
    ];

    for (width, height) in test_cases {
        let data = vec![0u8; 1000];
        let mut reader = create_test_reader(data);
        let result = decode_mmr_bitmap(&mut reader, width, height, false);
        assert!(result.is_ok(), "Failed for size {}x{}", width, height);
        let bitmap = result.unwrap();
        assert_eq!(bitmap.width, width);
        assert_eq!(bitmap.height, height);
        if width > 0 && height > 0 {
            assert_eq!(bitmap.get_pixel(0, 0), 0);
            if width > 1 {
                assert_eq!(bitmap.get_pixel(width - 1, 0), 0);
            }
            if height > 1 {
                assert_eq!(bitmap.get_pixel(0, height - 1), 0);
            }
        }
    }
}

#[test]
fn test_decode_mmr_large_dimensions() {
    let data = vec![0u8; 10000];
    let mut reader = create_test_reader(data);
    let result = decode_mmr_bitmap(&mut reader, 1000, 1000, false);
    assert!(result.is_ok());
    let bitmap = result.unwrap();
    assert_eq!(bitmap.width, 1000);
    assert_eq!(bitmap.height, 1000);
    // Spot-check corners
    assert_eq!(bitmap.get_pixel(0, 0), 0);
    assert_eq!(bitmap.get_pixel(999, 0), 0);
    assert_eq!(bitmap.get_pixel(0, 999), 0);
    assert_eq!(bitmap.get_pixel(999, 999), 0);
}
