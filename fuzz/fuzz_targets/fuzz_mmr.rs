#![no_main]

use libfuzzer_sys::fuzz_target;
use jbig2_rs::decode::decode_mmr::decode_mmr_bitmap;

fuzz_target!(|data: &[u8]| {
    if data.len() < 5 {
        return;
    }
    
    // Extract dimensions (limit to reasonable sizes)
    let width = ((data[0] as usize) % 100) + 1;
    let height = ((data[1] as usize) % 100) + 1;
    let end_of_block = (data[2] & 1) != 0;
    
    // Use remaining data as MMR-encoded data
    let mmr_data = &data[3..];
    
    // Create a Reader from the data
    let mut reader = jbig2_rs::reader::Reader::new(mmr_data.to_vec(), 0, mmr_data.len());
    
    // Try to decode the MMR bitmap
    // This should not panic regardless of input
    let _ = decode_mmr_bitmap(&mut reader, width, height, end_of_block);
});
