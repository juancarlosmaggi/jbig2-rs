pub const MAX_INT_32: i32 = i32::MAX;
pub const MIN_INT_32: i32 = i32::MIN;

pub fn log2(x: u32) -> u32 {
    if x == 0 {
        0
    } else {
        (x as f64).log2() as u32
    }
}

pub fn read_int8(data: &[u8], pos: usize) -> i8 {
    data[pos] as i8
}

pub fn read_uint16(data: &[u8], pos: usize) -> u16 {
    ((data[pos] as u16) << 8) | (data[pos + 1] as u16)
}

pub fn read_uint32(data: &[u8], pos: usize) -> u32 {
    ((data[pos] as u32) << 24) | ((data[pos + 1] as u32) << 16) | ((data[pos + 2] as u32) << 8) | (data[pos + 3] as u32)
}