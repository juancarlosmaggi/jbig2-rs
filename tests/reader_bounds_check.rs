use jbig2_rs::common::reader::Reader;

#[test]
fn test_reader_end_out_of_bounds_handled() {
    let data = vec![0x01];
    // end is 5, but data len is 1. Should be clamped to 1.
    let mut reader = Reader::new(data, 0, 5);

    // Consume the first byte (8 bits)
    for _ in 0..8 {
        assert!(reader.read_bit().is_ok());
    }

    // Next bit should error because we hit the end of data (clamped to 1)
    let result = reader.read_bit();
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("end of data while reading bit"));
}
