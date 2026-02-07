use crate::bitmap::Bitmap;
use crate::bitmap::utils as bitmap_utils;
use crate::arithmetic::contexts::DecodingContext;
use crate::decoders::refinement::RefinementParams;
use crate::decoders::refinement::decode_refinement;
use crate::arithmetic::helpers::{
    decode_i32_huffman_or_arith, decode_integer_context, decode_u32_huffman_or_arith,
};
use crate::common::error::Jbig2Error;
use crate::huffman::TextRegionHuffmanTables;
use crate::common::reader::Reader;
use crate::common::validation;

/// Inputs required to decode a text region.
#[derive(Clone)]
pub struct TextRegionParams<'a> {
    pub huffman: bool,
    pub refinement: bool,
    pub width: usize,
    pub height: usize,
    pub default_pixel_value: u8,
    pub number_of_symbol_instances: usize,
    pub strip_size: usize,
    pub input_symbols: Vec<&'a Bitmap>,
    pub symbol_code_length: usize,
    pub symbol_id_limit: usize,
    pub transposed: bool,
    pub ds_offset: i32,
    pub reference_corner: usize,
    pub combination_operator: usize,
    pub log_strip_size: usize,
    pub huffman_tables: Option<TextRegionHuffmanTables>,
    pub refinement_template_index: usize,
    pub refinement_at: Vec<(i8, i8)>,
}

/// Decode a text region and return the composed bitmap.
pub fn decode_text_region(
    params: &TextRegionParams<'_>,
    decoding_context: &mut DecodingContext<'_>,
    mut huffman_input: Option<&mut Reader<'_>>,
) -> Result<Bitmap, Jbig2Error> {
    // Validate parameters before decoding.
    validation::validate_text_decode_params(
        params.width,
        params.height,
        params.reference_corner,
        params.combination_operator,
    )?;
    if params.input_symbols.is_empty() {
        return Err(Jbig2Error::new("no input symbols for text region"));
    }
    if params.huffman && params.huffman_tables.is_none() {
        return Err(Jbig2Error::new(
            "Huffman tables required for Huffman decoding",
        ));
    }
    // Initialize the output bitmap.
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
            if symbol_id >= params.symbol_id_limit {
                return Err(Jbig2Error::new(&format!(
                    "invalid symbol id {} (max {}) at instance {} of {}",
                    symbol_id,
                    params.symbol_id_limit.saturating_sub(1),
                    i + 1,
                    params.number_of_symbol_instances
                )));
            }
            let symbol_bitmap = params.input_symbols.get(symbol_id).copied();
            let symbol_present = symbol_bitmap
                .map(|bm| bm.width > 0 && bm.height > 0)
                .unwrap_or(false);
            let symbol_bitmap = if symbol_present { symbol_bitmap } else { None };
            let symbol_width = symbol_bitmap.map(|bm| bm.width).unwrap_or(0);
            let symbol_height = symbol_bitmap.map(|bm| bm.height).unwrap_or(0);
            let apply_refinement = if params.refinement {
                if params.huffman {
                    huffman_input.as_mut().unwrap().read_bits(1)? != 0
                } else {
                    decode_integer_context(decoding_context, "IARI")?.unwrap_or(0) != 0
                }
            } else {
                false
            };
            let mut rdw = 0i32;
            let mut rdh = 0i32;
            let mut rdx = 0i32;
            let mut rdy = 0i32;
            let mut bmsize: Option<i32> = None;
            if apply_refinement {
                if params.huffman {
                    // Mixed Huffman/Arithmetic coding for refinement
                    let tables = huffman_tables.unwrap();
                    rdw = tables
                        .table_refinement_dw
                        .as_ref()
                        .unwrap()
                        .decode(huffman_input.as_mut().unwrap())?;
                    rdh = tables
                        .table_refinement_dh
                        .as_ref()
                        .unwrap()
                        .decode(huffman_input.as_mut().unwrap())?;
                    rdx = tables
                        .table_refinement_dx
                        .as_ref()
                        .unwrap()
                        .decode(huffman_input.as_mut().unwrap())?;
                    rdy = tables
                        .table_refinement_dy
                        .as_ref()
                        .unwrap()
                        .decode(huffman_input.as_mut().unwrap())?;
                    let size = tables
                        .table_refinement_size
                        .as_ref()
                        .unwrap()
                        .decode(huffman_input.as_mut().unwrap())?;
                    if size < 0 {
                        return Err(Jbig2Error::new("invalid refinement bitmap size"));
                    }
                    bmsize = Some(size);
                    huffman_input.as_mut().unwrap().byte_align();
                } else {
                    // Pure Arithmetic coding
                    rdw = decode_integer_context(decoding_context, "IARDW")?
                        .ok_or_else(|| Jbig2Error::new("OOB when decoding refinement width delta"))?;
                    rdh = decode_integer_context(decoding_context, "IARDH")?
                        .ok_or_else(|| Jbig2Error::new("OOB when decoding refinement height delta"))?;
                    rdx = decode_integer_context(decoding_context, "IARDX")?
                        .ok_or_else(|| Jbig2Error::new("OOB when decoding refinement x offset"))?;
                    rdy = decode_integer_context(decoding_context, "IARDY")?
                        .ok_or_else(|| Jbig2Error::new("OOB when decoding refinement y offset"))?;
                }
            }
            let (final_symbol_width, final_symbol_height, final_symbol_bitmap) = if symbol_present {
                let symbol_bitmap = symbol_bitmap.unwrap();
                if apply_refinement {
                    let base_width = i32::try_from(symbol_width)
                        .map_err(|_| Jbig2Error::new("symbol width overflow"))?;
                    let base_height = i32::try_from(symbol_height)
                        .map_err(|_| Jbig2Error::new("symbol height overflow"))?;
                    let refined_width_i32 = base_width
                        .checked_add(rdw)
                        .ok_or_else(|| Jbig2Error::new("refinement width overflow"))?;
                    let refined_height_i32 = base_height
                        .checked_add(rdh)
                        .ok_or_else(|| Jbig2Error::new("refinement height overflow"))?;
                    if refined_width_i32 < 0 || refined_height_i32 < 0 {
                        return Err(Jbig2Error::new(&format!(
                            "invalid refinement dimensions base={}x{} rdw={} rdh={} sym={} inst={}",
                            base_width,
                            base_height,
                            rdw,
                            rdh,
                            symbol_id,
                            i + 1
                        )));
                    }
                    let refined_width = refined_width_i32 as usize;
                    let refined_height = refined_height_i32 as usize;
                    let bitmap = if params.huffman {
                        let current_pos = huffman_input.as_ref().unwrap().get_position();
                        let data = huffman_input.as_ref().unwrap().get_data();
                        let mut temp_context = DecodingContext::new(data, current_pos, data.len());
                        decode_refinement(
                            &RefinementParams {
                                width: refined_width,
                                height: refined_height,
                                template_index: params.refinement_template_index,
                                reference_bitmap: symbol_bitmap,
                                offset_x: (rdw >> 1) + rdx,
                                offset_y: (rdh >> 1) + rdy,
                                prediction: false,
                                at: params.refinement_at.as_slice(),
                            },
                            &mut temp_context,
                        )?
                    } else {
                        decode_refinement(
                            &RefinementParams {
                                width: refined_width,
                                height: refined_height,
                                template_index: params.refinement_template_index,
                                reference_bitmap: symbol_bitmap,
                                offset_x: (rdw >> 1) + rdx,
                                offset_y: (rdh >> 1) + rdy,
                                prediction: false,
                                at: params.refinement_at.as_slice(),
                            },
                            decoding_context,
                        )?
                    };
                    if let Some(size) = bmsize {
                        huffman_input.as_mut().unwrap().skip(size as usize);
                    }
                    (refined_width, refined_height, Some(bitmap))
                } else {
                    (symbol_width, symbol_height, Some(symbol_bitmap.clone()))
                }
            } else {
                if apply_refinement {
                    if let Some(size) = bmsize {
                        huffman_input.as_mut().unwrap().skip(size as usize);
                    }
                }
                (0, 0, None)
            };
            let (width_adjust, height_adjust) = if final_symbol_bitmap.is_some() {
                let symbol_width = i32::try_from(final_symbol_width)
                    .map_err(|_| Jbig2Error::new("symbol width overflow"))?;
                let symbol_height = i32::try_from(final_symbol_height)
                    .map_err(|_| Jbig2Error::new("symbol height overflow"))?;
                (symbol_width - 1, symbol_height - 1)
            } else {
                (0, 0)
            };

            let mut s = current_s;
            if !params.transposed {
                if params.reference_corner > 1 {
                    s = s.wrapping_add(width_adjust);
                }
            } else if (params.reference_corner & 1) == 0 {
                s = s.wrapping_add(height_adjust);
            }

            let (x, y) = if final_symbol_bitmap.is_some() {
                if !params.transposed {
                    match params.reference_corner {
                        0 => (s, t.wrapping_sub(height_adjust)), // bottom-left
                        1 => (s, t),                             // top-left
                        2 => (
                            s.wrapping_sub(width_adjust),
                            t.wrapping_sub(height_adjust),
                        ), // bottom-right
                        _ => (s.wrapping_sub(width_adjust), t), // top-right
                    }
                } else {
                    match params.reference_corner {
                        0 => (t, s.wrapping_sub(height_adjust)), // bottom-left
                        1 => (t, s),                             // top-left
                        2 => (
                            t.wrapping_sub(width_adjust),
                            s.wrapping_sub(height_adjust),
                        ), // bottom-right
                        _ => (t.wrapping_sub(width_adjust), s), // top-right
                    }
                }
            } else if !params.transposed {
                match params.reference_corner {
                    0 => (s, t.wrapping_add(1)), // bottom-left
                    1 => (s, t),                 // top-left
                    2 => (s.wrapping_add(1), t.wrapping_add(1)), // bottom-right
                    _ => (s.wrapping_add(1), t), // top-right
                }
            } else {
                match params.reference_corner {
                    0 => (t, s.wrapping_add(1)), // bottom-left
                    1 => (t, s),                 // top-left
                    2 => (t.wrapping_add(1), s.wrapping_add(1)), // bottom-right
                    _ => (t.wrapping_add(1), s), // top-right
                }
            };

            if let Some(final_symbol_bitmap) = final_symbol_bitmap.as_ref() {
                bitmap.combine(
                    final_symbol_bitmap,
                    x as isize,
                    y as isize,
                    params.combination_operator as u8,
                );
            }

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
