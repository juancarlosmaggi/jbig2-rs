// Utility functions for reading binary data

use crate::error::Jbig2Error;

pub fn read_u32(data: &[u8], pos: usize) -> u32 {
    ((data[pos] as u32) << 24)
        | ((data[pos + 1] as u32) << 16)
        | ((data[pos + 2] as u32) << 8)
        | (data[pos + 3] as u32)
}

pub fn read_u32_le(data: &[u8], pos: usize) -> u32 {
    (data[pos] as u32)
        | ((data[pos + 1] as u32) << 8)
        | ((data[pos + 2] as u32) << 16)
        | ((data[pos + 3] as u32) << 24)
}

pub fn read_u16(data: &[u8], pos: usize) -> u16 {
    ((data[pos] as u16) << 8) | (data[pos + 1] as u16)
}

pub fn read_u16_le(data: &[u8], pos: usize) -> u16 {
    (data[pos] as u16) | ((data[pos + 1] as u16) << 8)
}

pub fn parse_at_parameters(
    data: &[u8],
    mut pos: usize,
    at_length: usize,
) -> Result<Vec<(i8, i8)>, Jbig2Error> {
    let mut at = vec![];
    for _ in 0..at_length {
        if pos + 1 >= data.len() {
            return Err(Jbig2Error::new("insufficient data for AT flags"));
        }
        let x = data[pos] as i8;
        let y = data[pos + 1] as i8;
        at.push((x, y));
        pos += 2;
    }
    Ok(at)
}
