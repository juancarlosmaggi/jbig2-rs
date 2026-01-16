//! Huffman Decoding for JBIG2
//!
//! This module implements Huffman decoding used in JBIG2 for various encoding tasks,
//! including symbol widths, height deltas, and text region symbol IDs.
//!
//! ## Overview
//!
//! JBIG2 uses Huffman coding as one of its entropy coding methods (alongside arithmetic coding).
//! Huffman tables can be either predefined standard tables or custom tables defined within
//! the data stream.
//!
//! ## Standard Tables
//!
//! The JBIG2 specification defines 16 standard Huffman tables (ITU T.88 Annex B):
//! - Tables A-D: Out-of-band (OOB) values supported
//! - Table E: Special table for HCHEIGHT
//! - Tables 1-16: Various standard encodings
//!
//! Standard tables are accessed via [`get_standard_table`].
//!
//! ## Custom Tables
//!
//! Custom Huffman tables can be defined in Table segments (type 53) and referenced
//! by other segments. Custom tables are parsed via [`decode_tables_segment`].
//!
//! ## Module Structure
//!
//! - **`standard_tables`** - Predefined standard Huffman tables
//! - **`table_selectors`** - Table selection logic for symbol dictionaries and text regions
//!
//! ## Usage
//!
//! ```no_run
//! use jbig2_rs::huffman::get_standard_table;
//! use jbig2_rs::reader::Reader;
//!
//! # fn example() -> Result<(), jbig2_rs::Jbig2Error> {
//! let table = get_standard_table(1)?;
//! let mut reader = Reader::new(vec![0x12, 0x34], 0, 2);
//! let value = table.decode(&mut reader)?;
//! # Ok(())
//! # }
//! ```

// Huffman module - organized into focused submodules
mod standard_tables;
mod table_selectors;

// Re-export public types and functions
pub use standard_tables::get_standard_table;
pub use table_selectors::{
    SymbolDictionaryHuffmanTables, TextRegionHuffmanParams, TextRegionHuffmanTables,
    decode_tables_segment, get_aggregate_symbol_huffman_tables,
    get_symbol_dictionary_huffman_tables, get_text_region_huffman_tables,
};

// Core Huffman types and decoding logic
use crate::error::Jbig2Error;
use crate::reader::Reader;

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
    pub fn new(line_data: Vec<i32>) -> Self {
        if line_data.len() == 2 {
            // OOB line
            HuffmanLine {
                is_oob: true,
                range_low: 0,
                prefix_length: line_data[0] as u32,
                range_length: 0,
                prefix_code: line_data[1] as u32,
                is_lower_range: false,
            }
        } else {
            // Normal, upper range or lower range line
            HuffmanLine {
                is_oob: false,
                range_low: line_data[0],
                prefix_length: line_data[1] as u32,
                range_length: line_data[2] as u32,
                prefix_code: line_data[3] as u32,
                is_lower_range: line_data.len() > 4 && line_data[4] == 1, // "lower" as 1
            }
        }
    }
}

pub struct HuffmanTreeNode {
    pub children: [Option<Box<HuffmanTreeNode>>; 2],
    pub is_leaf: bool,
    pub range_length: u32,
    pub range_low: i32,
    pub is_lower_range: bool,
    pub is_oob: bool,
}

impl Clone for HuffmanTreeNode {
    fn clone(&self) -> Self {
        HuffmanTreeNode {
            children: [
                self.children[0].as_ref().map(|c| Box::new((**c).clone())),
                self.children[1].as_ref().map(|c| Box::new((**c).clone())),
            ],
            is_leaf: self.is_leaf,
            range_length: self.range_length,
            range_low: self.range_low,
            is_lower_range: self.is_lower_range,
            is_oob: self.is_oob,
        }
    }
}

impl HuffmanTreeNode {
    pub fn new_leaf(line: &HuffmanLine) -> Self {
        HuffmanTreeNode {
            children: [None, None],
            is_leaf: true,
            range_length: line.range_length,
            range_low: line.range_low,
            is_lower_range: line.is_lower_range,
            is_oob: line.is_oob,
        }
    }

    pub fn new_intermediate() -> Self {
        HuffmanTreeNode {
            children: [None, None],
            is_leaf: false,
            range_length: 0,
            range_low: 0,
            is_lower_range: false,
            is_oob: false,
        }
    }

    pub fn build_tree(&mut self, line: &HuffmanLine, shift: u32) {
        let bit = ((line.prefix_code >> shift) & 1) as usize;
        if shift == 0 {
            self.children[bit] = Some(Box::new(HuffmanTreeNode::new_leaf(line)));
        } else {
            if self.children[bit].is_none() {
                self.children[bit] = Some(Box::new(HuffmanTreeNode::new_intermediate()));
            }
            if let Some(ref mut child) = self.children[bit] {
                child.build_tree(line, shift - 1);
            }
        }
    }

    pub fn decode_node(&self, reader: &mut Reader) -> Result<(i32, bool), Jbig2Error> {
        if self.is_leaf {
            if self.is_oob {
                return Ok((0, true)); // OOB
            }
            let ht_offset = reader.read_bits(self.range_length)?;
            let val = self.range_low
                + if self.is_lower_range {
                    -(ht_offset as i32)
                } else {
                    ht_offset as i32
                };
            Ok((val, false))
        } else {
            let bit = reader.read_bit()? as usize;
            if let Some(ref child) = self.children[bit] {
                child.decode_node(reader)
            } else {
                Err(Jbig2Error::new("invalid Huffman data"))
            }
        }
    }
}

#[derive(Clone)]
pub struct HuffmanTable {
    pub root_node: HuffmanTreeNode,
}

impl HuffmanTable {
    pub fn new(mut lines: Vec<HuffmanLine>, prefix_codes_done: bool) -> Self {
        if !prefix_codes_done {
            Self::assign_prefix_codes(&mut lines);
        }
        let mut root = HuffmanTreeNode::new_intermediate();
        for line in &lines {
            if line.prefix_length > 0 {
                root.build_tree(line, line.prefix_length - 1);
            }
        }
        HuffmanTable { root_node: root }
    }

    pub fn decode(&self, reader: &mut Reader) -> Result<i32, Jbig2Error> {
        self.root_node.decode_node(reader).map(|(val, _)| val)
    }

    pub fn decode_entry(&self, reader: &mut Reader) -> Result<(i32, bool), Jbig2Error> {
        self.root_node.decode_node(reader)
    }

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
