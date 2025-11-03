use crate::error::Jbig2Error;
use crate::reader::Reader;
use std::collections::HashMap;

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

    pub fn decode_node(&self, reader: &mut Reader) -> Result<i32, Jbig2Error> {
        if self.is_leaf {
            if self.is_oob {
                return Ok(-1); // OOB
            }
            let ht_offset = reader.read_bits(self.range_length)?;
            Ok(self.range_low + if self.is_lower_range { -(ht_offset as i32) } else { ht_offset as i32 })
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

pub fn decode_tables_segment(data: &[u8], start: usize, end: usize) -> Result<HuffmanTable, Jbig2Error> {
    // Decodes a Tables segment, i.e., a custom Huffman table.
    // Annex B.2 Code table structure.
    let flags = data[start];
    let lowest_value = ((data[start + 1] as u32) << 24) | ((data[start + 2] as u32) << 16) | ((data[start + 3] as u32) << 8) | (data[start + 4] as u32);
    let highest_value = ((data[start + 5] as u32) << 24) | ((data[start + 6] as u32) << 16) | ((data[start + 7] as u32) << 8) | (data[start + 8] as u32);
    let mut reader = Reader::new(data.to_vec(), start + 9, end);

    let prefix_size_bits = (((flags >> 1) & 7) + 1) as u32;
    let range_size_bits = (((flags >> 4) & 7) + 1) as u32;

    let mut lines = Vec::new();
    let mut current_range_low = lowest_value as i32;

    // Normal table lines
    while current_range_low < highest_value as i32 {
        let prefix_length = reader.read_bits(prefix_size_bits)?;
        let range_length = reader.read_bits(range_size_bits)?;
        lines.push(HuffmanLine::new(vec![current_range_low, prefix_length as i32, range_length as i32, 0]));
        current_range_low += 1i32 << range_length;
    }

    // Lower range table line
    let prefix_length = reader.read_bits(prefix_size_bits)?;
    lines.push(HuffmanLine::new(vec![lowest_value as i32 - 1, prefix_length as i32, 32, 0, 1])); // "lower"

    // Upper range table line
    let prefix_length = reader.read_bits(prefix_size_bits)?;
    lines.push(HuffmanLine::new(vec![highest_value as i32, prefix_length as i32, 32, 0]));

    if (flags & 1) != 0 {
        // Out-of-band table line
        let prefix_length = reader.read_bits(prefix_size_bits)?;
        lines.push(HuffmanLine::new(vec![prefix_length as i32, 0]));
    }

    Ok(HuffmanTable::new(lines, false))
}

pub fn get_standard_table(number: u32) -> Result<HuffmanTable, Jbig2Error> {
    if number == 0 || number > 15 {
        return Err(Jbig2Error::new("invalid standard Huffman table number"));
    }
    let lines = match number {
        1 => vec![
            HuffmanLine::new(vec![0, 1, 4, 0x0]),
            HuffmanLine::new(vec![16, 2, 8, 0x2]),
            HuffmanLine::new(vec![272, 3, 16, 0x6]),
            HuffmanLine::new(vec![65808, 3, 32, 0x7]), // upper
        ],
        2 => vec![
            HuffmanLine::new(vec![0, 1, 0, 0x0]),
            HuffmanLine::new(vec![1, 2, 0, 0x2]),
            HuffmanLine::new(vec![2, 3, 0, 0x6]),
            HuffmanLine::new(vec![3, 4, 3, 0xe]),
            HuffmanLine::new(vec![11, 5, 6, 0x1e]),
            HuffmanLine::new(vec![75, 6, 32, 0x3e]), // upper
            HuffmanLine::new(vec![6, 0x3f]), // OOB
        ],
        3 => vec![
            HuffmanLine::new(vec![-256, 8, 8, 0xfe]),
            HuffmanLine::new(vec![0, 1, 0, 0x0]),
            HuffmanLine::new(vec![1, 2, 0, 0x2]),
            HuffmanLine::new(vec![2, 3, 0, 0x6]),
            HuffmanLine::new(vec![3, 4, 3, 0xe]),
            HuffmanLine::new(vec![11, 5, 6, 0x1e]),
            HuffmanLine::new(vec![-257, 8, 32, 0xff, 1]), // lower
            HuffmanLine::new(vec![75, 7, 32, 0x7e]), // upper
            HuffmanLine::new(vec![6, 0x3e]), // OOB
        ],
        4 => vec![
            HuffmanLine::new(vec![1, 1, 0, 0x0]),
            HuffmanLine::new(vec![2, 2, 0, 0x2]),
            HuffmanLine::new(vec![3, 3, 0, 0x6]),
            HuffmanLine::new(vec![4, 4, 3, 0xe]),
            HuffmanLine::new(vec![12, 5, 6, 0x1e]),
            HuffmanLine::new(vec![76, 5, 32, 0x1f]), // upper
        ],
        5 => vec![
            HuffmanLine::new(vec![-255, 7, 8, 0x7e]),
            HuffmanLine::new(vec![1, 1, 0, 0x0]),
            HuffmanLine::new(vec![2, 2, 0, 0x2]),
            HuffmanLine::new(vec![3, 3, 0, 0x6]),
            HuffmanLine::new(vec![4, 4, 3, 0xe]),
            HuffmanLine::new(vec![12, 5, 6, 0x1e]),
            HuffmanLine::new(vec![-256, 7, 32, 0x7f, 1]), // lower
            HuffmanLine::new(vec![76, 6, 32, 0x3e]), // upper
        ],
        6 => vec![
            HuffmanLine::new(vec![-2048, 5, 10, 0x1c]),
            HuffmanLine::new(vec![-1024, 4, 9, 0x8]),
            HuffmanLine::new(vec![-512, 4, 8, 0x9]),
            HuffmanLine::new(vec![-256, 4, 7, 0xa]),
            HuffmanLine::new(vec![-128, 5, 6, 0x1d]),
            HuffmanLine::new(vec![-64, 5, 5, 0x1e]),
            HuffmanLine::new(vec![-32, 4, 5, 0xb]),
            HuffmanLine::new(vec![0, 2, 7, 0x0]),
            HuffmanLine::new(vec![128, 3, 7, 0x2]),
            HuffmanLine::new(vec![256, 3, 8, 0x3]),
            HuffmanLine::new(vec![512, 4, 9, 0xc]),
            HuffmanLine::new(vec![1024, 4, 10, 0xd]),
            HuffmanLine::new(vec![-2049, 6, 32, 0x3e, 1]), // lower
            HuffmanLine::new(vec![2048, 6, 32, 0x3f]), // upper
        ],
        7 => vec![
            HuffmanLine::new(vec![-1024, 4, 9, 0x8]),
            HuffmanLine::new(vec![-512, 3, 8, 0x0]),
            HuffmanLine::new(vec![-256, 4, 7, 0x9]),
            HuffmanLine::new(vec![-128, 5, 6, 0x1a]),
            HuffmanLine::new(vec![-64, 5, 5, 0x1b]),
            HuffmanLine::new(vec![-32, 4, 5, 0xa]),
            HuffmanLine::new(vec![0, 4, 5, 0xb]),
            HuffmanLine::new(vec![32, 5, 5, 0x1c]),
            HuffmanLine::new(vec![64, 5, 6, 0x1d]),
            HuffmanLine::new(vec![128, 4, 7, 0xc]),
            HuffmanLine::new(vec![256, 3, 8, 0x1]),
            HuffmanLine::new(vec![512, 3, 9, 0x2]),
            HuffmanLine::new(vec![1024, 3, 10, 0x3]),
            HuffmanLine::new(vec![-1025, 5, 32, 0x1e, 1]), // lower
            HuffmanLine::new(vec![2048, 5, 32, 0x1f]), // upper
        ],
        8 => vec![
            HuffmanLine::new(vec![-15, 8, 3, 0xfc]),
            HuffmanLine::new(vec![-7, 9, 1, 0x1fc]),
            HuffmanLine::new(vec![-5, 8, 1, 0xfd]),
            HuffmanLine::new(vec![-3, 9, 0, 0x1fd]),
            HuffmanLine::new(vec![-2, 7, 0, 0x7c]),
            HuffmanLine::new(vec![-1, 4, 0, 0xa]),
            HuffmanLine::new(vec![0, 2, 1, 0x0]),
            HuffmanLine::new(vec![2, 5, 0, 0x1a]),
            HuffmanLine::new(vec![3, 6, 0, 0x3a]),
            HuffmanLine::new(vec![4, 3, 4, 0x4]),
            HuffmanLine::new(vec![20, 6, 1, 0x3b]),
            HuffmanLine::new(vec![22, 4, 4, 0xb]),
            HuffmanLine::new(vec![38, 4, 5, 0xc]),
            HuffmanLine::new(vec![70, 5, 6, 0x1b]),
            HuffmanLine::new(vec![134, 5, 7, 0x1c]),
            HuffmanLine::new(vec![262, 6, 7, 0x3c]),
            HuffmanLine::new(vec![390, 7, 8, 0x7d]),
            HuffmanLine::new(vec![646, 6, 10, 0x3d]),
            HuffmanLine::new(vec![-16, 9, 32, 0x1fe, 1]), // lower
            HuffmanLine::new(vec![1670, 9, 32, 0x1ff]), // upper
            HuffmanLine::new(vec![2, 0x1]), // OOB
        ],
        9 => vec![
            HuffmanLine::new(vec![-31, 8, 4, 0xfc]),
            HuffmanLine::new(vec![-15, 9, 2, 0x1fc]),
            HuffmanLine::new(vec![-11, 8, 2, 0xfd]),
            HuffmanLine::new(vec![-7, 9, 1, 0x1fd]),
            HuffmanLine::new(vec![-5, 7, 1, 0x7c]),
            HuffmanLine::new(vec![-3, 4, 1, 0xa]),
            HuffmanLine::new(vec![-1, 3, 1, 0x2]),
            HuffmanLine::new(vec![1, 3, 1, 0x3]),
            HuffmanLine::new(vec![3, 5, 1, 0x1a]),
            HuffmanLine::new(vec![5, 6, 1, 0x3a]),
            HuffmanLine::new(vec![7, 3, 5, 0x4]),
            HuffmanLine::new(vec![39, 6, 2, 0x3b]),
            HuffmanLine::new(vec![43, 4, 5, 0xb]),
            HuffmanLine::new(vec![75, 4, 6, 0xc]),
            HuffmanLine::new(vec![139, 5, 7, 0x1b]),
            HuffmanLine::new(vec![267, 5, 8, 0x1c]),
            HuffmanLine::new(vec![523, 6, 8, 0x3c]),
            HuffmanLine::new(vec![779, 7, 9, 0x7d]),
            HuffmanLine::new(vec![1291, 6, 11, 0x3d]),
            HuffmanLine::new(vec![-32, 9, 32, 0x1fe, 1]), // lower
            HuffmanLine::new(vec![3339, 9, 32, 0x1ff]), // upper
            HuffmanLine::new(vec![2, 0x0]), // OOB
        ],
        10 => vec![
            HuffmanLine::new(vec![-21, 7, 4, 0x7a]),
            HuffmanLine::new(vec![-5, 8, 0, 0xfc]),
            HuffmanLine::new(vec![-4, 7, 0, 0x7b]),
            HuffmanLine::new(vec![-3, 5, 0, 0x18]),
            HuffmanLine::new(vec![-2, 2, 2, 0x0]),
            HuffmanLine::new(vec![2, 5, 0, 0x19]),
            HuffmanLine::new(vec![3, 6, 0, 0x36]),
            HuffmanLine::new(vec![4, 7, 0, 0x7c]),
            HuffmanLine::new(vec![5, 8, 0, 0xfd]),
            HuffmanLine::new(vec![6, 2, 6, 0x1]),
            HuffmanLine::new(vec![70, 5, 5, 0x1a]),
            HuffmanLine::new(vec![102, 6, 5, 0x37]),
            HuffmanLine::new(vec![134, 6, 6, 0x38]),
            HuffmanLine::new(vec![198, 6, 7, 0x39]),
            HuffmanLine::new(vec![326, 6, 8, 0x3a]),
            HuffmanLine::new(vec![582, 6, 9, 0x3b]),
            HuffmanLine::new(vec![1094, 6, 10, 0x3c]),
            HuffmanLine::new(vec![2118, 7, 11, 0x7d]),
            HuffmanLine::new(vec![-22, 8, 32, 0xfe, 1]), // lower
            HuffmanLine::new(vec![4166, 8, 32, 0xff]), // upper
            HuffmanLine::new(vec![2, 0x2]), // OOB
        ],
        11 => vec![
            HuffmanLine::new(vec![1, 1, 0, 0x0]),
            HuffmanLine::new(vec![2, 2, 1, 0x2]),
            HuffmanLine::new(vec![4, 4, 0, 0xc]),
            HuffmanLine::new(vec![5, 4, 1, 0xd]),
            HuffmanLine::new(vec![7, 5, 1, 0x1c]),
            HuffmanLine::new(vec![9, 5, 2, 0x1d]),
            HuffmanLine::new(vec![13, 6, 2, 0x3c]),
            HuffmanLine::new(vec![17, 7, 2, 0x7a]),
            HuffmanLine::new(vec![21, 7, 3, 0x7b]),
            HuffmanLine::new(vec![29, 7, 4, 0x7c]),
            HuffmanLine::new(vec![45, 7, 5, 0x7d]),
            HuffmanLine::new(vec![77, 7, 6, 0x7e]),
            HuffmanLine::new(vec![141, 7, 32, 0x7f]), // upper
        ],
        12 => vec![
            HuffmanLine::new(vec![1, 1, 0, 0x0]),
            HuffmanLine::new(vec![2, 2, 0, 0x2]),
            HuffmanLine::new(vec![3, 3, 1, 0x6]),
            HuffmanLine::new(vec![5, 5, 0, 0x1c]),
            HuffmanLine::new(vec![6, 5, 1, 0x1d]),
            HuffmanLine::new(vec![8, 6, 1, 0x3c]),
            HuffmanLine::new(vec![10, 7, 0, 0x7a]),
            HuffmanLine::new(vec![11, 7, 1, 0x7b]),
            HuffmanLine::new(vec![13, 7, 2, 0x7c]),
            HuffmanLine::new(vec![17, 7, 3, 0x7d]),
            HuffmanLine::new(vec![25, 7, 4, 0x7e]),
            HuffmanLine::new(vec![41, 8, 5, 0xfe]),
            HuffmanLine::new(vec![73, 8, 32, 0xff]), // upper
        ],
        13 => vec![
            HuffmanLine::new(vec![1, 1, 0, 0x0]),
            HuffmanLine::new(vec![2, 3, 0, 0x4]),
            HuffmanLine::new(vec![3, 4, 0, 0xc]),
            HuffmanLine::new(vec![4, 5, 0, 0x1c]),
            HuffmanLine::new(vec![5, 4, 1, 0xd]),
            HuffmanLine::new(vec![7, 3, 3, 0x5]),
            HuffmanLine::new(vec![15, 6, 1, 0x3a]),
            HuffmanLine::new(vec![17, 6, 2, 0x3b]),
            HuffmanLine::new(vec![21, 6, 3, 0x3c]),
            HuffmanLine::new(vec![29, 6, 4, 0x3d]),
            HuffmanLine::new(vec![45, 6, 5, 0x3e]),
            HuffmanLine::new(vec![77, 7, 6, 0x7e]),
            HuffmanLine::new(vec![141, 7, 32, 0x7f]), // upper
        ],
        14 => vec![
            HuffmanLine::new(vec![-2, 3, 0, 0x4]),
            HuffmanLine::new(vec![-1, 3, 0, 0x5]),
            HuffmanLine::new(vec![0, 1, 0, 0x0]),
            HuffmanLine::new(vec![1, 3, 0, 0x6]),
            HuffmanLine::new(vec![2, 3, 0, 0x7]),
        ],
        15 => vec![
            HuffmanLine::new(vec![-24, 7, 4, 0x7c]),
            HuffmanLine::new(vec![-8, 6, 2, 0x3c]),
            HuffmanLine::new(vec![-4, 5, 1, 0x1c]),
            HuffmanLine::new(vec![-2, 4, 0, 0xc]),
            HuffmanLine::new(vec![-1, 3, 0, 0x4]),
            HuffmanLine::new(vec![0, 1, 0, 0x0]),
            HuffmanLine::new(vec![1, 3, 0, 0x5]),
            HuffmanLine::new(vec![2, 4, 0, 0xd]),
            HuffmanLine::new(vec![3, 5, 1, 0x1d]),
            HuffmanLine::new(vec![5, 6, 2, 0x3d]),
            HuffmanLine::new(vec![9, 7, 4, 0x7d]),
            HuffmanLine::new(vec![-25, 7, 32, 0x7e, 1]), // lower
            HuffmanLine::new(vec![25, 7, 32, 0x7f]), // upper
        ],
        _ => return Err(Jbig2Error::new(&format!("standard table B.{} does not exist", number))),
    };
    Ok(HuffmanTable::new(lines, true))
}

#[derive(Clone)]
pub struct SymbolDictionaryHuffmanTables {
    pub table_delta_height: HuffmanTable,
    pub table_delta_width: HuffmanTable,
    pub table_bitmap_size: HuffmanTable,
    pub table_aggregate_instances: HuffmanTable,
}

pub fn get_symbol_dictionary_huffman_tables(
    huffman_dh_selector: u8,
    huffman_dw_selector: u8,
    bitmap_size_selector: bool,
    aggregation_instances_selector: bool,
    referred_to: &[u32],
    custom_tables: &HashMap<u32, HuffmanTable>,
) -> Result<SymbolDictionaryHuffmanTables, Jbig2Error> {
    let mut custom_index = 0;
    let table_delta_height = match huffman_dh_selector {
        0 | 1 => get_standard_table(huffman_dh_selector as u32 + 4)?,
        3 => get_custom_huffman_table(custom_index, referred_to, custom_tables)?,
        _ => return Err(Jbig2Error::new("invalid Huffman DH selector")),
    };
    if huffman_dh_selector == 3 {
        custom_index += 1;
    }

    let table_delta_width = match huffman_dw_selector {
        0 | 1 => get_standard_table(huffman_dw_selector as u32 + 2)?,
        3 => get_custom_huffman_table(custom_index, referred_to, custom_tables)?,
        _ => return Err(Jbig2Error::new("invalid Huffman DW selector")),
    };
    if huffman_dw_selector == 3 {
        custom_index += 1;
    }

    let table_bitmap_size = if bitmap_size_selector {
        get_custom_huffman_table(custom_index, referred_to, custom_tables)?
    } else {
        get_standard_table(1)?
    };
    if bitmap_size_selector {
        custom_index += 1;
    }

    let table_aggregate_instances = if aggregation_instances_selector {
        get_custom_huffman_table(custom_index, referred_to, custom_tables)?
    } else {
        get_standard_table(1)?
    };
    // No need to increment custom_index as it's the last use

    Ok(SymbolDictionaryHuffmanTables {
        table_delta_height,
        table_delta_width,
        table_bitmap_size,
        table_aggregate_instances,
    })
}

#[derive(Clone)]
pub struct TextRegionHuffmanParams {
    pub huffman_fs: u8,
    pub huffman_ds: u8,
    pub huffman_dt: u8,
    pub huffman_refinement_dw: u8,
    pub huffman_refinement_dh: u8,
    pub huffman_refinement_dx: u8,
    pub huffman_refinement_dy: u8,
    pub huffman_refinement_size_selector: bool,
}

#[derive(Clone)]
pub struct TextRegionHuffmanTables {
    pub symbol_id_table: HuffmanTable,
    pub table_first_s: HuffmanTable,
    pub table_delta_s: HuffmanTable,
    pub table_delta_t: HuffmanTable,
    pub table_refinement_dw: Option<HuffmanTable>,
    pub table_refinement_dh: Option<HuffmanTable>,
    pub table_refinement_dx: Option<HuffmanTable>,
    pub table_refinement_dy: Option<HuffmanTable>,
    pub table_refinement_size: Option<HuffmanTable>,
}

pub fn get_text_region_huffman_tables(
    params: &TextRegionHuffmanParams,
    referred_to: &[u32],
    custom_tables: &HashMap<u32, HuffmanTable>,
    number_of_symbols: usize,
    reader: &mut Reader,
) -> Result<TextRegionHuffmanTables, Jbig2Error> {
    // 7.4.3.1.7 Symbol ID Huffman table decoding
    // Read code lengths for RUNCODEs 0...34.
    let mut codes = Vec::new();
    for i in 0..=34 {
        let code_length = reader.read_bits(4)?;
        codes.push(HuffmanLine::new(vec![i, code_length as i32, 0, 0]));
    }
    // Assign Huffman codes for RUNCODEs.
    let run_codes_table = HuffmanTable::new(codes, false);

    // Read a Huffman code using the assignment above.
    // Interpret the RUNCODE codes and the additional bits (if any).
    codes = Vec::new();
    let mut i = 0;
    while i < number_of_symbols {
        let code_length = run_codes_table.decode(reader)? as u32;
        if code_length >= 32 {
            let repeated_length;
            let number_of_repeats = match code_length {
                32 => {
                    if i == 0 {
                        return Err(Jbig2Error::new("no previous value in symbol ID table"));
                    }
                    repeated_length = codes[i - 1].prefix_length;
                    (reader.read_bits(2)? + 3) as usize
                }
                33 => {
                    repeated_length = 0;
                    (reader.read_bits(3)? + 3) as usize
                }
                34 => {
                    repeated_length = 0;
                    (reader.read_bits(7)? + 11) as usize
                }
                _ => return Err(Jbig2Error::new("invalid code length in symbol ID table")),
            };
            for _ in 0..number_of_repeats {
                codes.push(HuffmanLine::new(vec![i as i32, repeated_length as i32, 0, 0]));
                i += 1;
            }
        } else {
        codes.push(HuffmanLine::new(vec![i as i32, code_length as i32, 0, 0]));
            i += 1;
        }
    }
    reader.byte_align();
    let symbol_id_table = HuffmanTable::new(codes, false);

    // 7.4.3.1.6 Text region segment Huffman table selection
    let mut custom_index = 0;
    let table_first_s = match params.huffman_fs {
        0 | 1 => get_standard_table(params.huffman_fs as u32 + 6)?,
        3 => {
            let table = get_custom_huffman_table(custom_index, referred_to, custom_tables)?;
            custom_index += 1;
            table
        }
        _ => return Err(Jbig2Error::new("invalid Huffman FS selector")),
    };

    let table_delta_s = match params.huffman_ds {
        0..=2 => get_standard_table(params.huffman_ds as u32 + 8)?,
        3 => {
            let table = get_custom_huffman_table(custom_index, referred_to, custom_tables)?;
            custom_index += 1;
            table
        }
        _ => return Err(Jbig2Error::new("invalid Huffman DS selector")),
    };

    let table_delta_t = match params.huffman_dt {
        0..=2 => get_standard_table(params.huffman_dt as u32 + 11)?,
        3 => {
            custom_index += 1;
            get_custom_huffman_table(custom_index - 1, referred_to, custom_tables)?
        }
        _ => return Err(Jbig2Error::new("invalid Huffman DT selector")),
    };

    // Refinement tables
    let table_refinement_dw = match params.huffman_refinement_dw {
        0..=2 => Some(get_standard_table(params.huffman_refinement_dw as u32 + 2)?),
        3 => {
            custom_index += 1;
            Some(get_custom_huffman_table(custom_index - 1, referred_to, custom_tables)?)
        }
        _ => return Err(Jbig2Error::new("invalid Huffman refinement DW selector")),
    };

    let table_refinement_dh = match params.huffman_refinement_dh {
        0..=2 => Some(get_standard_table(params.huffman_refinement_dh as u32 + 2)?),
        3 => {
            custom_index += 1;
            Some(get_custom_huffman_table(custom_index - 1, referred_to, custom_tables)?)
        }
        _ => return Err(Jbig2Error::new("invalid Huffman refinement DH selector")),
    };

    let table_refinement_dx = match params.huffman_refinement_dx {
        0..=2 => Some(get_standard_table(params.huffman_refinement_dx as u32 + 2)?),
        3 => {
            custom_index += 1;
            Some(get_custom_huffman_table(custom_index - 1, referred_to, custom_tables)?)
        }
        _ => return Err(Jbig2Error::new("invalid Huffman refinement DX selector")),
    };

    let table_refinement_dy = match params.huffman_refinement_dy {
        0..=2 => Some(get_standard_table(params.huffman_refinement_dy as u32 + 2)?),
        3 => {
            custom_index += 1;
            Some(get_custom_huffman_table(custom_index - 1, referred_to, custom_tables)?)
        }
        _ => return Err(Jbig2Error::new("invalid Huffman refinement DY selector")),
    };

    let table_refinement_size = if params.huffman_refinement_size_selector {
        custom_index += 1;
        Some(get_custom_huffman_table(custom_index - 1, referred_to, custom_tables)?)
    } else {
        Some(get_standard_table(1)?)
    };

    Ok(TextRegionHuffmanTables {
        symbol_id_table,
        table_first_s,
        table_delta_s,
        table_delta_t,
        table_refinement_dw,
        table_refinement_dh,
        table_refinement_dx,
        table_refinement_dy,
        table_refinement_size,
    })
}

fn get_custom_huffman_table(
    index: u32,
    referred_to: &[u32],
    custom_tables: &HashMap<u32, HuffmanTable>,
) -> Result<HuffmanTable, Jbig2Error> {
    let current_index = index as usize;
    if current_index >= referred_to.len() {
        return Err(Jbig2Error::new("can't find custom Huffman table"));
    }
    let table_segment = referred_to[current_index];
    match custom_tables.get(&table_segment) {
        Some(table) => Ok(table.clone()),
        None => Err(Jbig2Error::new("can't find custom Huffman table")),
    }
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