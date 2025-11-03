use crate::bitmap::Bitmap;
use crate::contexts::DecodingContext;
use crate::decoder::{decode_integer_context, decode_iaid_context};
use crate::error::Jbig2Error;
use crate::huffman::TextRegionHuffmanTables;
use crate::reader::Reader;

#[derive(Clone)]
pub struct TextRegionParams {
    pub huffman: bool,
    pub refinement: bool,
    pub width: usize,
    pub height: usize,
    pub default_pixel_value: u8,
    pub number_of_symbol_instances: usize,
    pub strip_size: usize,
    pub input_symbols: Vec<Bitmap>,
    pub symbol_code_length: usize,
    pub transposed: bool,
    pub ds_offset: i32,
    pub reference_corner: usize,
    pub combination_operator: usize,
    pub log_strip_size: usize,
    pub huffman_tables: Option<TextRegionHuffmanTables>,
    pub refinement_template_index: usize,
    pub refinement_at: Vec<(i8, i8)>,
}

pub fn decode_text_region(
    params: &TextRegionParams,
    decoding_context: &mut DecodingContext,
    mut huffman_input: Option<&mut Reader>,
) -> Result<Bitmap, Jbig2Error> {
    // Validate parameters
    if params.width == 0 || params.height == 0 {
        return Err(Jbig2Error::new("invalid text region dimensions"));
    }
    if params.width > 65535 || params.height > 65535 {
        return Err(Jbig2Error::new("text region dimensions too large"));
    }
    if params.input_symbols.is_empty() {
        return Err(Jbig2Error::new("no input symbols for text region"));
    }
    if params.reference_corner > 3 {
        return Err(Jbig2Error::new("invalid reference corner"));
    }
    if params.combination_operator > 7 {
        return Err(Jbig2Error::new("invalid combination operator"));
    }

    // Prepare bitmap
    let mut bitmap = Bitmap::new(params.width, params.height);
    if params.default_pixel_value != 0 {
        for y in 0..params.height {
            for x in 0..params.width {
                bitmap.set_pixel(x, y, 1);
            }
        }
    }
    let huffman_tables = params.huffman_tables.as_ref();
    let mut strip_t = if params.huffman {
        let tables = huffman_tables.unwrap();
        -(tables.table_delta_t.decode(huffman_input.as_mut().unwrap())?)
    } else {
        -(decode_integer_context(decoding_context, "IADT")?.unwrap_or(0) as i32)
    };
    let mut first_s = 0i32;
    let mut i = 0;
    while i < params.number_of_symbol_instances {
        let delta_t = if params.huffman {
            let tables = huffman_tables.unwrap();
            tables.table_delta_t.decode(huffman_input.as_mut().unwrap())?
        } else {
            decode_integer_context(decoding_context, "IADT")?.unwrap_or(0) as i32
        };
        strip_t += delta_t;
        let delta_first_s = if params.huffman {
            let tables = huffman_tables.unwrap();
            tables.table_first_s.decode(huffman_input.as_mut().unwrap())?
        } else {
            decode_integer_context(decoding_context, "IAFS")?.unwrap_or(0) as i32
        };
        first_s += delta_first_s;
        let mut current_s = first_s;
        loop {
            let mut current_t = 0i32;
            if params.strip_size > 1 {
                current_t = if params.huffman {
                    huffman_input.as_mut().unwrap().read_bits(params.log_strip_size as u32)? as i32
                } else {
                    decode_integer_context(decoding_context, "IAIT")?.unwrap_or(0) as i32
                };
            }
            let t = (params.strip_size as i32) * strip_t + current_t;
            let symbol_id = if params.huffman {
                let tables = huffman_tables.unwrap();
                tables.symbol_id_table.decode(huffman_input.as_mut().unwrap())? as usize
            } else {
                decode_iaid_context(decoding_context, params.symbol_code_length)? as usize
            };
            if symbol_id >= params.input_symbols.len() {
                return Err(Jbig2Error::new("invalid symbol id"));
            }
            let symbol_bitmap = &params.input_symbols[symbol_id];
            let symbol_width = symbol_bitmap.width;
            let symbol_height = symbol_bitmap.height;
            let apply_refinement = if params.refinement {
                decode_integer_context(decoding_context, "IARI")?.unwrap_or(0) != 0
            } else {
                false
            };
            let (final_symbol_width, final_symbol_height, final_symbol_bitmap) = if apply_refinement {
                let (rdw, rdh, rdx, rdy) = if let Some(ref huffman_tables) = params.huffman_tables {
                    // Use Huffman tables for refinement parameters
                    let rdw = huffman_tables.table_refinement_dw.as_ref().unwrap().decode(huffman_input.as_mut().unwrap())?;
                    let rdh = huffman_tables.table_refinement_dh.as_ref().unwrap().decode(huffman_input.as_mut().unwrap())?;
                    let rdx = huffman_tables.table_refinement_dx.as_ref().unwrap().decode(huffman_input.as_mut().unwrap())?;
                    let rdy = huffman_tables.table_refinement_dy.as_ref().unwrap().decode(huffman_input.as_mut().unwrap())?;
                    (rdw, rdh, rdx, rdy)
                } else {
                    // Use arithmetic decoding
                    let rdw = decode_integer_context(decoding_context, "IARDW")?.unwrap_or(0);
                    let rdh = decode_integer_context(decoding_context, "IARDH")?.unwrap_or(0);
                    let rdx = decode_integer_context(decoding_context, "IARDX")?.unwrap_or(0);
                    let rdy = decode_integer_context(decoding_context, "IARDY")?.unwrap_or(0);
                    (rdw, rdh, rdx, rdy)
                };
                let refined_width = symbol_width + rdw as usize;
                let refined_height = symbol_height + rdh as usize;
                let refined_bitmap = crate::decode_refinement::decode_refinement(
                    &crate::decode_refinement::RefinementParams {
                        width: refined_width,
                        height: refined_height,
                        template_index: params.refinement_template_index,
                        reference_bitmap: symbol_bitmap,
                        offset_x: (rdw >> 1) + rdx,
                        offset_y: (rdh >> 1) + rdy,
                        prediction: false,
                        at: params.refinement_at.clone(),
                    },
                    decoding_context,
                )?;
                (refined_width, refined_height, refined_bitmap)
            } else {
                (symbol_width, symbol_height, symbol_bitmap.clone())
            };
            let increment = if !params.transposed {
                if params.reference_corner > 1 {
                    current_s += final_symbol_width as i32 - 1;
                    final_symbol_width as i32 - 1
                } else {
                    final_symbol_width as i32 - 1
                }
            } else {
                final_symbol_height as i32 - 1
            };
            let offset_t = t - if (params.reference_corner & 1) != 0 { 0 } else { final_symbol_height as i32 - 1 };
            let offset_s = current_s - if (params.reference_corner & 2) != 0 { final_symbol_width as i32 - 1 } else { 0 };
            // Draw the symbol
            if params.transposed {
                for s2 in 0..final_symbol_height {
                    let row = offset_s + s2 as i32;
                    if row < 0 || row >= params.height as i32 {
                        continue;
                    }
                    for t2 in 0..final_symbol_width {
                        let col = offset_t + t2 as i32;
                        if col >= 0 && col < params.width as i32 {
                            let src_pixel = final_symbol_bitmap.get_pixel(t2, s2);
                            let dst_pixel = bitmap.get_pixel(col as usize, row as usize);
                            let new_pixel = match params.combination_operator {
                                0 => src_pixel, // OR
                                2 => dst_pixel ^ src_pixel, // XOR
                                _ => return Err(Jbig2Error::new("unsupported combination operator")),
                            };
                            bitmap.set_pixel(col as usize, row as usize, new_pixel);
                        }
                    }
                }
            } else {
                for t2 in 0..final_symbol_height {
                    let row = offset_t + t2 as i32;
                    if row < 0 || row >= params.height as i32 {
                        continue;
                    }
                    for s2 in 0..final_symbol_width {
                        let col = offset_s + s2 as i32;
                        if col >= 0 && col < params.width as i32 {
                            let src_pixel = final_symbol_bitmap.get_pixel(s2, t2);
                            let dst_pixel = bitmap.get_pixel(col as usize, row as usize);
                            let new_pixel = match params.combination_operator {
                                0 => src_pixel, // OR
                                2 => dst_pixel ^ src_pixel, // XOR
                                _ => return Err(Jbig2Error::new("unsupported combination operator")),
                            };
                            bitmap.set_pixel(col as usize, row as usize, new_pixel);
                        }
                    }
                }
            }
            i += 1;
            let delta_s = if params.huffman {
                let tables = huffman_tables.unwrap();
                Some(tables.table_delta_s.decode(huffman_input.as_mut().unwrap())?)
            } else {
                decode_integer_context(decoding_context, "IADS")?
            };
            if delta_s.is_none() {
                break; // OOB
            }
            current_s += increment + delta_s.unwrap() + params.ds_offset;
        }
    }
    Ok(bitmap)
}