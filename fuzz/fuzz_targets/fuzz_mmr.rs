#![no_main]

use libfuzzer_sys::fuzz_target;
use jbig2_rs::decode::decode_mmr::decode_mmr_bitmap;

fuzz_target!(|data: &[u8]| {
    if data.len() < 5 {
        return;
    }

    // Extract dimensions with caps to limit work.
    let width = ((data[0] as usize) % 100) + 1;
    let height = ((data[1] as usize) % 100) + 1;
    let end_of_block = (data[2] & 1) != 0;

    // Use remaining data as the MMR payload.
    let mmr_data = &data[3..];

    let mut reader = jbig2_rs::reader::Reader::new(mmr_data.to_vec(), 0, mmr_data.len());

    // Decode the bitmap; fuzzing ensures this does not panic.
    let _ = decode_mmr_bitmap(&mut reader, width, height, end_of_block);
});
