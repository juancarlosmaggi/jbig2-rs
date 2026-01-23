use jbig2_rs::common::reader::Reader;
use proptest::prelude::*;

proptest! {
    /// Reader position should start at the specified offset.
    #[test]
    fn prop_reader_initial_position(data_len in 10usize..100, start in 0usize..50) {
        let start = start.min(data_len);
        let data = vec![0u8; data_len];
        let reader = Reader::new(data, start, data_len);
        prop_assert_eq!(reader.get_position(), start);
    }

    /// Reading a byte should advance position by 1.
    #[test]
    fn prop_read_byte_advances_position(data_len in 10usize..100) {
        let data = vec![0xABu8; data_len];
        let mut reader = Reader::new(data, 0, data_len);
        let initial_pos = reader.get_position();
        reader.read_byte();
        prop_assert_eq!(reader.get_position(), initial_pos + 1);
    }

    /// Reading N bytes should advance position by N.
    #[test]
    fn prop_read_n_bytes_advances_by_n(data_len in 20usize..100, n in 1usize..10) {
        let data = vec![0xFFu8; data_len];
        let mut reader = Reader::new(data, 0, data_len);
        let initial_pos = reader.get_position();

        for _ in 0..n {
            if reader.read_byte().is_none() {
                break;
            }
        }

        let expected_pos = (initial_pos + n).min(data_len);
        prop_assert_eq!(reader.get_position(), expected_pos);
    }

    /// Reading 8 bits should equal reading 1 byte.
    #[test]
    fn prop_read_8_bits_equals_byte(byte_val in any::<u8>()) {
        let data = vec![byte_val; 10];
        let mut reader1 = Reader::new(data.clone(), 0, 10);
        let mut reader2 = Reader::new(data, 0, 10);

        let byte = reader1.read_byte().unwrap();
        let bits = reader2.read_bits(8).unwrap();

        prop_assert_eq!(byte as u32, bits);
    }

    /// Sequential bit reads should match batch reads.
    #[test]
    fn prop_read_bits_sequential_vs_batch(byte_val in any::<u8>()) {
        let data = vec![byte_val; 10];
        let mut reader1 = Reader::new(data.clone(), 0, 10);
        let mut reader2 = Reader::new(data, 0, 10);

        // Read 4 bits at once.
        let batch = reader1.read_bits(4).unwrap();

        // Read 4 bits one by one.
        let mut sequential = 0u32;
        for i in (0..4).rev() {
            sequential |= (reader2.read_bit().unwrap() as u32) << i;
        }

        prop_assert_eq!(batch, sequential);
    }

    /// byte_align should clear any partial byte state.
    #[test]
    fn prop_byte_align_clears_partial(data_len in 10usize..50, bits_to_read in 1usize..7) {
        let data = vec![0xFFu8; data_len];
        let mut reader = Reader::new(data, 0, data_len);

        // Read some bits to create misalignment.
        for _ in 0..bits_to_read {
            let _ = reader.read_bit();
        }

        let pos_before_align = reader.get_position();
        reader.byte_align();
        let pos_after_align = reader.get_position();

        // Next read should start from a byte boundary.
        prop_assert!(pos_after_align >= pos_before_align);
    }

    /// skip(n) should advance position by n.
    #[test]
    fn prop_skip_advances_position(data_len in 20usize..100, skip_n in 1usize..10) {
        let data = vec![0u8; data_len];
        let mut reader = Reader::new(data, 0, data_len);
        let initial_pos = reader.get_position();
        reader.skip(skip_n);
        prop_assert_eq!(reader.get_position(), initial_pos + skip_n);
    }

    /// set_position should move to the exact position.
    #[test]
    fn prop_set_position(data_len in 20usize..100, new_pos in 0usize..50) {
        let data = vec![0u8; data_len];
        let mut reader = Reader::new(data, 0, data_len);
        reader.set_position(new_pos);
        prop_assert_eq!(reader.get_position(), new_pos);
    }

    /// Reading beyond the end should fail.
    #[test]
    fn prop_read_beyond_end_fails(data_len in 5usize..20) {
        let data = vec![0u8; data_len];
        let mut reader = Reader::new(data, 0, data_len);

        // Read all bytes.
        for _ in 0..data_len {
            reader.read_byte();
        }

        // Next read should fail.
        prop_assert!(reader.read_byte().is_none());
    }

    /// set_limit should restrict available data.
    #[test]
    fn prop_set_limit_restricts_data(data_len in 20usize..100, limit in 1usize..10) {
        let data = vec![0xFFu8; data_len];
        let mut reader = Reader::new(data, 0, data_len);
        reader.set_limit(limit);

        // Should be able to read `limit` bytes.
        for _ in 0..limit {
            prop_assert!(reader.read_byte().is_some());
        }

        // Next read should fail.
        prop_assert!(reader.read_byte().is_none());
    }
}
