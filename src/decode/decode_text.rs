use crate::bitmap::Bitmap;
use crate::bitmap_utils;
use crate::contexts::DecodingContext;
use crate::decode::decode_refinement::RefinementParams;
use crate::decode::decode_refinement::decode_refinement;
use crate::decoder::{
    decode_i32_huffman_or_arith, decode_integer_context, decode_u32_huffman_or_arith,
};
use crate::error::Jbig2Error;
use crate::huffman::TextRegionHuffmanTables;
use crate::reader::Reader;
use crate::validation;
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
    validation::validate_text_decode_params(
        params.width,
        params.height,
        params.reference_corner,
        params.combination_operator,
    )?;
    if params.input_symbols.is_empty() {
        return Err(Jbig2Error::new("no input symbols for text region"));
    }
    // Refinement with Huffman is now supported
    // if params.refinement && params.huffman {
    //     return Err(Jbig2Error::new("refinement with Huffman is not supported"));
    // }
    if params.huffman && params.huffman_tables.is_none() {
        return Err(Jbig2Error::new(
            "Huffman tables required for Huffman decoding",
        ));
    }
    // Prepare bitmap
    let mut bitmap = bitmap_utils::create_initialized_bitmap(
        params.width,
        params.height,
        params.default_pixel_value,
    );
    let huffman_tables = params.huffman_tables.as_ref();
    let initial_strip_t = decode_i32_huffman_or_arith(
        params.huffman,
        || {
            let tables = huffman_tables.unwrap();
            tables.table_delta_t.decode(huffman_input.as_mut().unwrap())
        },
        "IADT",
        decoding_context,
    )?;
    let mut strip_t = initial_strip_t
        .checked_neg()
        .ok_or_else(|| Jbig2Error::new("strip T overflow"))?;
    let mut first_s = 0i32;
    let mut i = 0;
    while i < params.number_of_symbol_instances {
        let delta_t = decode_i32_huffman_or_arith(
            params.huffman,
            || {
                let tables = huffman_tables.unwrap();
                tables.table_delta_t.decode(huffman_input.as_mut().unwrap())
            },
            "IADT",
            decoding_context,
        )?;
        strip_t = strip_t
            .checked_add(delta_t)
            .ok_or_else(|| Jbig2Error::new("strip T overflow"))?;
        let delta_first_s = decode_i32_huffman_or_arith(
            params.huffman,
            || {
                let tables = huffman_tables.unwrap();
                tables.table_first_s.decode(huffman_input.as_mut().unwrap())
            },
            "IAFS",
            decoding_context,
        )?;
        first_s = first_s
            .checked_add(delta_first_s)
            .ok_or_else(|| Jbig2Error::new("first S overflow"))?;
        let mut current_s = first_s;
        loop {
            let mut current_t = 0i32;
            if params.strip_size > 1 {
                current_t = if params.huffman {
                    huffman_input
                        .as_mut()
                        .unwrap()
                        .read_bits(params.log_strip_size as u32)? as i32
                } else {
                    decode_integer_context(decoding_context, "IAIT")?.unwrap_or(0) as i32
                };
            }
            let t = (params.strip_size as i32)
                .checked_mul(strip_t)
                .and_then(|val| val.checked_add(current_t))
                .ok_or_else(|| Jbig2Error::new("text region T overflow"))?;
            let symbol_id = decode_u32_huffman_or_arith(
                params.huffman,
                || {
                    let tables = huffman_tables.unwrap();
                    tables
                        .symbol_id_table
                        .decode(huffman_input.as_mut().unwrap())
                },
                params.symbol_code_length,
                decoding_context,
            )? as usize;
            if symbol_id >= params.input_symbols.len() {
                return Err(Jbig2Error::new(&format!(
                    "invalid symbol id {} (max {}) at instance {} of {}",
                    symbol_id,
                    params.input_symbols.len().saturating_sub(1),
                    i + 1,
                    params.number_of_symbol_instances
                )));
            }
            let symbol_bitmap = &params.input_symbols[symbol_id];
            let symbol_width = symbol_bitmap.width;
            let symbol_height = symbol_bitmap.height;
            let apply_refinement = if params.refinement {
                if params.huffman {
                    huffman_input.as_mut().unwrap().read_bits(1)? != 0
                } else {
                    decode_integer_context(decoding_context, "IARI")?.unwrap_or(0) != 0
                }
            } else {
                false
            };
            let (final_symbol_width, final_symbol_height, final_symbol_bitmap) = if apply_refinement
            {
                let (rdw, rdh, _rdx, _rdy, refined_bitmap) = if params.huffman {
                    // Mixed Huffman/Arithmetic coding for refinement
                    // 1. Decode refinement parameters using Huffman
                    let tables = huffman_tables.unwrap();
                    let rdw = tables
                        .table_refinement_dw
                        .as_ref()
                        .unwrap()
                        .decode(huffman_input.as_mut().unwrap())?;
                    let rdh = tables
                        .table_refinement_dh
                        .as_ref()
                        .unwrap()
                        .decode(huffman_input.as_mut().unwrap())?;
                    let rdx = tables
                        .table_refinement_dx
                        .as_ref()
                        .unwrap()
                        .decode(huffman_input.as_mut().unwrap())?;
                    let rdy = tables
                        .table_refinement_dy
                        .as_ref()
                        .unwrap()
                        .decode(huffman_input.as_mut().unwrap())?;
                    let bmsize = tables
                        .table_refinement_size
                        .as_ref()
                        .unwrap()
                        .decode(huffman_input.as_mut().unwrap())?;
                    if bmsize < 0 {
                        return Err(Jbig2Error::new("invalid refinement bitmap size"));
                    }

                    // 2. Switch to Arithmetic for the bitmap
                    huffman_input.as_mut().unwrap().byte_align();
                    let current_pos = huffman_input.as_ref().unwrap().get_position();
                    let data = huffman_input.as_ref().unwrap().get_data();

                    // Create a temporary decoding context for the arithmetic part
                    // We use the remaining data from the current position
                    let mut temp_context =
                        DecodingContext::new(data.to_vec(), current_pos, data.len());

                    let refined_width = (symbol_width as i32 + rdw) as usize;
                    let refined_height = (symbol_height as i32 + rdh) as usize;

                    let bitmap = decode_refinement(
                        &RefinementParams {
                            width: refined_width,
                            height: refined_height,
                            template_index: params.refinement_template_index,
                            reference_bitmap: symbol_bitmap,
                            offset_x: (rdw >> 1) + rdx,
                            offset_y: (rdh >> 1) + rdy,
                            prediction: false,
                            at: params.refinement_at.clone(),
                        },
                        &mut temp_context,
                    )?;

                    // 3. Advance Huffman reader by the refinement bitmap size
                    huffman_input
                        .as_mut()
                        .unwrap()
                        .skip(bmsize as usize);

                    (rdw, rdh, rdx, rdy, bitmap)
                } else {
                    // Pure Arithmetic coding
                    let rdw = decode_integer_context(decoding_context, "IARDW")?.unwrap_or(0);
                    let rdh = decode_integer_context(decoding_context, "IARDH")?.unwrap_or(0);
                    let rdx = decode_integer_context(decoding_context, "IARDX")?.unwrap_or(0);
                    let rdy = decode_integer_context(decoding_context, "IARDY")?.unwrap_or(0);

                    let refined_width = (symbol_width as i32 + rdw) as usize;
                    let refined_height = (symbol_height as i32 + rdh) as usize;

                    let bitmap = decode_refinement(
                        &RefinementParams {
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
                    (rdw, rdh, rdx, rdy, bitmap)
                };

                let refined_width = (symbol_width as i32 + rdw) as usize;
                let refined_height = (symbol_height as i32 + rdh) as usize;
                (refined_width, refined_height, refined_bitmap)
            } else {
                (symbol_width, symbol_height, symbol_bitmap.clone())
            };
            let symbol_width = i32::try_from(final_symbol_width)
                .map_err(|_| Jbig2Error::new("symbol width overflow"))?;
            let symbol_height = i32::try_from(final_symbol_height)
                .map_err(|_| Jbig2Error::new("symbol height overflow"))?;
            let width_adjust = symbol_width
                .checked_sub(1)
                .ok_or_else(|| Jbig2Error::new("symbol width invalid"))?;
            let height_adjust = symbol_height
                .checked_sub(1)
                .ok_or_else(|| Jbig2Error::new("symbol height invalid"))?;

            let mut s = current_s;
            if !params.transposed {
                if params.reference_corner > 1 {
                    s = s.wrapping_add(width_adjust);
                }
            } else if (params.reference_corner & 1) == 0 {
                s = s.wrapping_add(height_adjust);
            }

            let (x, y) = if !params.transposed {
                match params.reference_corner {
                    0 => (s, t),
                    1 => (s.wrapping_sub(width_adjust), t),
                    2 => (s, t.wrapping_sub(height_adjust)),
                    _ => (
                        s.wrapping_sub(width_adjust),
                        t.wrapping_sub(height_adjust),
                    ),
                }
            } else {
                match params.reference_corner {
                    0 => (t, s),
                    1 => (t.wrapping_sub(width_adjust), s),
                    2 => (t, s.wrapping_sub(height_adjust)),
                    _ => (
                        t.wrapping_sub(width_adjust),
                        s.wrapping_sub(height_adjust),
                    ),
                }
            };

            bitmap.combine(
                &final_symbol_bitmap,
                x as isize,
                y as isize,
                params.combination_operator as u8,
            );

            if !params.transposed {
                if params.reference_corner < 2 {
                    s = s.wrapping_add(width_adjust);
                }
            } else if (params.reference_corner & 1) != 0 {
                s = s.wrapping_add(height_adjust);
            }
            current_s = s;
            i += 1;
            if i >= params.number_of_symbol_instances {
                break; // Processed all symbols
            }
            let delta_s = if params.huffman {
                let tables = huffman_tables.unwrap();
                let (val, oob) =
                    tables
                        .table_delta_s
                        .decode_entry(huffman_input.as_mut().unwrap())?;
                if oob {
                    None
                } else {
                    Some(val)
                }
            } else {
                decode_integer_context(decoding_context, "IADS")?
            };
            if delta_s.is_none() {
                break; // OOB
            }
            let delta_s = delta_s.unwrap();
            current_s = current_s
                .wrapping_add(delta_s)
                .wrapping_add(params.ds_offset);
        }
    }
    Ok(bitmap)
}
