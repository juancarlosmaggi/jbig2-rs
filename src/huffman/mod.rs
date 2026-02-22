//! Huffman decoding support used by multiple region decoders.

// Huffman module split into tables and selector helpers.
mod standard_tables;
mod table_selectors;

// Re-export public types and functions.
pub use standard_tables::get_standard_table;
pub use table_selectors::{
    SymbolDictionaryHuffmanTables, TextRegionHuffmanParams, TextRegionHuffmanTables,
    decode_tables_segment, get_aggregate_symbol_huffman_tables,
    get_symbol_dictionary_huffman_tables, get_text_region_huffman_tables,
};

// Core Huffman types and decoding logic.
use crate::common::error::Jbig2Error;
use crate::common::reader::Reader;

/// Represents a single line in a Huffman table definition.
#[derive(Clone)]
pub struct HuffmanLine {
    pub is_oob: bool,
    pub range_low: i32,
    pub prefix_length: u32,
    pub range_length: u32,
    pub prefix_code: u32,
    pub is_lower_range: bool,
}

impl HuffmanLine {
    /// Build a Huffman line from the parsed numeric representation.
    pub fn new(line_data: Vec<i32>) -> Self {
        if line_data.len() == 2 {
            // OOB line.
            HuffmanLine {
                is_oob: true,
                range_low: 0,
                prefix_length: line_data[0] as u32,
                range_length: 0,
                prefix_code: line_data[1] as u32,
                is_lower_range: false,
            }
        } else {
            // Normal, upper range, or lower range line.
            HuffmanLine {
                is_oob: false,
                range_low: line_data[0],
                prefix_length: line_data[1] as u32,
                range_length: line_data[2] as u32,
                prefix_code: line_data[3] as u32,
                is_lower_range: line_data.len() > 4 && line_data[4] == 1,
            }
        }
    }
}

const NO_CHILD: u32 = u32::MAX;

/// Flattened Huffman tree node.
#[derive(Clone, Copy, Debug)]
pub enum HuffmanNode {
    Internal {
        left: u32,
        right: u32,
    },
    Leaf {
        range_length: u32,
        range_low: i32,
        is_lower_range: bool,
        is_oob: bool,
    },
}

/// Huffman table with a decoded binary tree.
#[derive(Clone)]
pub struct HuffmanTable {
    pub nodes: Vec<HuffmanNode>,
}

impl HuffmanTable {
    /// Build a table from line definitions, assigning prefix codes as needed.
    pub fn new(mut lines: Vec<HuffmanLine>, prefix_codes_done: bool) -> Self {
        if !prefix_codes_done {
            Self::assign_prefix_codes(&mut lines);
        }

        // A Huffman tree with N leaves has at most 2*N nodes.
        // We use lines.len() * 2 as a safe upper bound estimate.
        let mut nodes = Vec::with_capacity(lines.len() * 2);
        nodes.push(HuffmanNode::Internal {
            left: NO_CHILD,
            right: NO_CHILD,
        });

        for line in &lines {
            if line.prefix_length > 0 {
                Self::add_line(&mut nodes, line);
            }
        }

        HuffmanTable { nodes }
    }

    fn add_line(nodes: &mut Vec<HuffmanNode>, line: &HuffmanLine) {
        let mut current_index = 0;
        let len = line.prefix_length;

        for i in 0..len {
            let shift = len - 1 - i;
            let bit = ((line.prefix_code >> shift) & 1) as usize;

            if shift == 0 {
                // We are at the leaf position. Create a leaf node.
                let leaf = HuffmanNode::Leaf {
                    range_length: line.range_length,
                    range_low: line.range_low,
                    is_lower_range: line.is_lower_range,
                    is_oob: line.is_oob,
                };
                let new_index = nodes.len() as u32;
                nodes.push(leaf);

                // Link the new leaf to the current parent.
                if let HuffmanNode::Internal {
                    ref mut left,
                    ref mut right,
                } = nodes[current_index]
                {
                    if bit == 0 {
                        *left = new_index;
                    } else {
                        *right = new_index;
                    }
                }
            } else {
                // We are at an intermediate position.
                // Check if the child exists.
                let child_index = match nodes[current_index] {
                    HuffmanNode::Internal { left, right } => {
                        if bit == 0 {
                            left
                        } else {
                            right
                        }
                    }
                    _ => NO_CHILD, // Should not happen for valid tables (prefix property)
                };

                if child_index != NO_CHILD {
                    // Child exists, move down.
                    current_index = child_index as usize;
                } else {
                    // Child does not exist, create a new internal node.
                    let new_node = HuffmanNode::Internal {
                        left: NO_CHILD,
                        right: NO_CHILD,
                    };
                    let new_index = nodes.len() as u32;
                    nodes.push(new_node);

                    // Link the new node to the parent.
                    if let HuffmanNode::Internal {
                        ref mut left,
                        ref mut right,
                    } = nodes[current_index]
                    {
                        if bit == 0 {
                            *left = new_index;
                        } else {
                            *right = new_index;
                        }
                    }
                    current_index = new_index as usize;
                }
            }
        }
    }

    /// Decode a value from the input stream.
    pub fn decode(&self, reader: &mut Reader<'_>) -> Result<i32, Jbig2Error> {
        self.decode_entry(reader).map(|(val, _)| val)
    }

    /// Decode a value and return whether it was an OOB marker.
    pub fn decode_entry(&self, reader: &mut Reader<'_>) -> Result<(i32, bool), Jbig2Error> {
        let mut current_index = 0;
        loop {
            // Safety: We construct the tree such that indices are valid.
            let node = unsafe { self.nodes.get_unchecked(current_index) };

            match node {
                HuffmanNode::Leaf {
                    range_length,
                    range_low,
                    is_lower_range,
                    is_oob,
                } => {
                    if *is_oob {
                        return Ok((0, true));
                    }
                    let ht_offset = reader.read_bits(*range_length)?;
                    let val = *range_low
                        + if *is_lower_range {
                            -(ht_offset as i32)
                        } else {
                            ht_offset as i32
                        };
                    return Ok((val, false));
                }
                HuffmanNode::Internal { left, right } => {
                    let bit = reader.read_bit()?;
                    let next_index = if bit == 0 { *left } else { *right };

                    if next_index == NO_CHILD {
                        return Err(Jbig2Error::new("invalid Huffman data"));
                    }
                    current_index = next_index as usize;
                }
            }
        }
    }

    /// Assign canonical prefix codes based on line prefix lengths.
    fn assign_prefix_codes(lines: &mut Vec<HuffmanLine>) {
        let mut prefix_length_max = 0;
        for line in &*lines {
            prefix_length_max = prefix_length_max.max(line.prefix_length);
        }

        let mut histogram = vec![0u32; (prefix_length_max + 1) as usize];
        for line in &*lines {
            histogram[line.prefix_length as usize] += 1;
        }
        histogram[0] = 0;

        let mut current_length = 1;
        let mut first_code = 0;
        let mut current_code;

        while current_length <= prefix_length_max {
            first_code = (first_code + histogram[(current_length - 1) as usize]) << 1;
            current_code = first_code;
            for line in lines.iter_mut() {
                if line.prefix_length == current_length {
                    line.prefix_code = current_code;
                    current_code += 1;
                }
            }
            current_length += 1;
        }
    }
}
