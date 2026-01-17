use crate::error::Jbig2Error;
use crate::huffman::{HuffmanTable, decode_tables_segment};
use std::collections::HashMap;

/// Decode a custom Huffman tables segment and store it by segment id.
pub(super) fn on_tables(
    custom_tables: &mut HashMap<u32, HuffmanTable>,
    segment_number: u32,
    data: &[u8],
    start: usize,
    end: usize,
) -> Result<(), Jbig2Error> {
    let table = decode_tables_segment(data, start, end)?;
    custom_tables.insert(segment_number, table);
    Ok(())
}
