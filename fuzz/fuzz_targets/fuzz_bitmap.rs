#![no_main]

use libfuzzer_sys::fuzz_target;
use jbig2_rs::bitmap::Bitmap;

fuzz_target!(|data: &[u8]| {
    if data.len() < 6 {
        return;
    }

    // Extract dimensions with caps to avoid excessive allocations.
    let width = ((data[0] as usize) % 200) + 1;
    let height = ((data[1] as usize) % 200) + 1;

    let mut bm1 = Bitmap::new(width, height);
    let mut bm2 = Bitmap::new(width / 2 + 1, height / 2 + 1);

    // Set a sample of pixels based on the input data.
    for (i, &byte) in data.iter().skip(2).take(20).enumerate() {
        let x = (byte as usize) % width;
        let y = i % height;
        bm1.set_pixel(x, y, byte & 1);

        let x2 = (byte as usize) % (width / 2 + 1);
        let y2 = i % (height / 2 + 1);
        bm2.set_pixel(x2, y2, (byte >> 1) & 1);
    }

    // Exercise get_pixel with in-range and out-of-range coordinates.
    for i in 0..10 {
        let x = (data[(i + 2) % data.len()] as usize) % (width + 10);
        let y = (data[(i + 3) % data.len()] as usize) % (height + 10);
        let _ = bm1.get_pixel(x, y);
    }

    // Exercise combine operations with varying offsets and operators.
    if data.len() > 30 {
        let x_offset = (data[22] as i8) as isize;
        let y_offset = (data[23] as i8) as isize;
        let operator = data[24] % 5; // 0-4 (OR, AND, XOR, XNOR, REPLACE)

        bm1.combine(&bm2, x_offset, y_offset, operator);
    }

    // Exercise cloning paths.
    let _ = bm1.clone();
});
