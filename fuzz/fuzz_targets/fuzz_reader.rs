#![no_main]

use libfuzzer_sys::fuzz_target;
use jbig2_rs::common::reader::Reader;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    // Pick a start/end window within the input.
    let start = (data[0] as usize) % (data.len() + 1);
    let end = if data.len() > 1 {
        start + ((data[1] as usize) % (data.len() - start + 1))
    } else {
        data.len()
    };

    let mut reader = Reader::new(data.to_vec(), start, end);

    // Exercise bit and byte reads.
    let _ = reader.read_byte();
    let _ = reader.read_bit();
    let _ = reader.read_bits(4);
    let _ = reader.read_bits(8);
    let _ = reader.read_bits(16);

    reader.byte_align();
    let _ = reader.get_position();

    if data.len() > 3 {
        reader.set_position(data[2] as usize);
        reader.skip(data[3] as usize % 100);
    }

    if data.len() > 4 {
        reader.set_limit(data[4] as usize);
    }

    // Read multiple bytes to stress position updates.
    for _ in 0..10 {
        if reader.read_byte().is_none() {
            break;
        }
    }
});
