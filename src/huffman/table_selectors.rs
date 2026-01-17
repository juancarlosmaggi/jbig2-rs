use super::standard_tables::get_standard_table;
use super::{HuffmanLine, HuffmanTable};
use crate::error::Jbig2Error;
use crate::reader::Reader;
use std::collections::HashMap;

// Custom Huffman table decoder
pub fn decode_tables_segment(
    data: &[u8],
    start: usize,
    end: usize,
) -> Result<HuffmanTable, Jbig2Error> {
    // Decodes a Tables segment, i.e., a custom Huffman table.
    // Annex B.2 Code table structure.
    let flags = data[start];
    let lowest_value = ((data[start + 1] as u32) << 24)
        | ((data[start + 2] as u32) << 16)
        | ((data[start + 3] as u32) << 8)
        | (data[start + 4] as u32);
    let highest_value = ((data[start + 5] as u32) << 24)
        | ((data[start + 6] as u32) << 16)
        | ((data[start + 7] as u32) << 8)
        | (data[start + 8] as u32);
    let mut reader = Reader::new(data.to_vec(), start + 9, end);

    let prefix_size_bits = (((flags >> 1) & 7) + 1) as u32;
    let range_size_bits = (((flags >> 4) & 7) + 1) as u32;

    let mut lines = Vec::new();
    let mut current_range_low = lowest_value as i32;

    // Normal table lines
    while current_range_low < highest_value as i32 {
        let prefix_length = reader.read_bits(prefix_size_bits)?;
        let range_length = reader.read_bits(range_size_bits)?;
        lines.push(HuffmanLine::new(vec![
            current_range_low,
            prefix_length as i32,
            range_length as i32,
            0,
        ]));
        current_range_low += 1i32 << range_length;
    }

    // Lower range table line
    let prefix_length = reader.read_bits(prefix_size_bits)?;
    lines.push(HuffmanLine::new(vec![
        lowest_value as i32 - 1,
        prefix_length as i32,
        32,
        0,
        1,
    ])); // "lower"

    // Upper range table line
    let prefix_length = reader.read_bits(prefix_size_bits)?;
    lines.push(HuffmanLine::new(vec![
        highest_value as i32,
        prefix_length as i32,
        32,
        0,
    ]));

    if (flags & 1) != 0 {
        // Out-of-band table line
        let prefix_length = reader.read_bits(prefix_size_bits)?;
        lines.push(HuffmanLine::new(vec![prefix_length as i32, 0]));
    }

    Ok(HuffmanTable::new(lines, false))
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
    pub huffman_ri: bool,
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
    pub table_refinement_ri: Option<HuffmanTable>,
}

fn decode_symbol_id_huffman_table(
    reader: &mut Reader,
    number_of_symbols: usize,
) -> Result<HuffmanTable, Jbig2Error> {
    let trace_huffman = std::env::var_os("JBIG2_RS_TRACE_HUFFMAN").is_some();
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
            let mut repeats = number_of_repeats;
            if i + repeats > number_of_symbols {
                repeats = number_of_symbols - i;
            }
            for _ in 0..repeats {
                codes.push(HuffmanLine::new(vec![
                    i as i32,
                    repeated_length as i32,
                    0,
                    0,
                ]));
                i += 1;
            }
        } else {
            codes.push(HuffmanLine::new(vec![i as i32, code_length as i32, 0, 0]));
            i += 1;
        }
    }
    if trace_huffman {
        let mut hist = [0u32; 33];
        let mut min_len = u32::MAX;
        let mut max_len = 0u32;
        for line in &codes {
            let len = line.prefix_length;
            if (len as usize) < hist.len() {
                hist[len as usize] = hist[len as usize].saturating_add(1);
            }
            min_len = min_len.min(len);
            max_len = max_len.max(len);
        }
        let zero_len = hist[0];
        eprintln!(
            "symbol_id_table: symbols={} len_range=[{}, {}] zero_len={}",
            number_of_symbols,
            if min_len == u32::MAX { 0 } else { min_len },
            max_len,
            zero_len
        );
        let mut sample = Vec::new();
        for (len, count) in hist.iter().enumerate() {
            if *count > 0 {
                sample.push(format!("{}:{}", len, count));
            }
        }
        eprintln!("symbol_id_table: len_hist {{ {} }}", sample.join(", "));
    }
    reader.byte_align();
    Ok(HuffmanTable::new(codes, false))
}

pub fn get_text_region_huffman_tables(
    params: &TextRegionHuffmanParams,
    referred_to: &[u32],
    custom_tables: &HashMap<u32, HuffmanTable>,
    number_of_symbols: usize,
    reader: &mut Reader,
) -> Result<TextRegionHuffmanTables, Jbig2Error> {
    let symbol_id_table = decode_symbol_id_huffman_table(reader, number_of_symbols)?;

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
            Some(get_custom_huffman_table(
                custom_index - 1,
                referred_to,
                custom_tables,
            )?)
        }
        _ => return Err(Jbig2Error::new("invalid Huffman refinement DW selector")),
    };

    let table_refinement_dh = match params.huffman_refinement_dh {
        0..=2 => Some(get_standard_table(params.huffman_refinement_dh as u32 + 2)?),
        3 => {
            custom_index += 1;
            Some(get_custom_huffman_table(
                custom_index - 1,
                referred_to,
                custom_tables,
            )?)
        }
        _ => return Err(Jbig2Error::new("invalid Huffman refinement DH selector")),
    };

    let table_refinement_dx = match params.huffman_refinement_dx {
        0..=2 => Some(get_standard_table(params.huffman_refinement_dx as u32 + 2)?),
        3 => {
            custom_index += 1;
            Some(get_custom_huffman_table(
                custom_index - 1,
                referred_to,
                custom_tables,
            )?)
        }
        _ => return Err(Jbig2Error::new("invalid Huffman refinement DX selector")),
    };

    let table_refinement_dy = match params.huffman_refinement_dy {
        0..=2 => Some(get_standard_table(params.huffman_refinement_dy as u32 + 2)?),
        3 => {
            custom_index += 1;
            Some(get_custom_huffman_table(
                custom_index - 1,
                referred_to,
                custom_tables,
            )?)
        }
        _ => return Err(Jbig2Error::new("invalid Huffman refinement DY selector")),
    };

    let table_refinement_size = if params.huffman_refinement_size_selector {
        custom_index += 1;
        Some(get_custom_huffman_table(
            custom_index - 1,
            referred_to,
            custom_tables,
        )?)
    } else {
        Some(get_standard_table(1)?)
    };

    let table_refinement_ri = if params.huffman_ri {
        custom_index += 1;
        Some(get_custom_huffman_table(
            custom_index - 1,
            referred_to,
            custom_tables,
        )?)
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
        table_refinement_ri,
    })
}

pub fn get_aggregate_symbol_huffman_tables(
    reader: &mut Reader,
    number_of_symbols: usize,
) -> Result<TextRegionHuffmanTables, Jbig2Error> {
    let symbol_id_table = decode_symbol_id_huffman_table(reader, number_of_symbols)?;

    Ok(TextRegionHuffmanTables {
        symbol_id_table,
        table_first_s: get_standard_table(6)?,  // B.6
        table_delta_s: get_standard_table(8)?,  // B.8
        table_delta_t: get_standard_table(11)?, // B.11
        table_refinement_dw: Some(get_standard_table(15)?), // B.15
        table_refinement_dh: Some(get_standard_table(15)?),
        table_refinement_dx: Some(get_standard_table(15)?),
        table_refinement_dy: Some(get_standard_table(15)?),
        table_refinement_size: Some(get_standard_table(1)?), // B.1
        table_refinement_ri: Some(get_standard_table(1)?),
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
