use crate::bitmap::Bitmap;
use crate::bitmap_utils::{self, apply_combination_operator};
use crate::contexts::DecodingContext;
use crate::decode::decode_refinement::RefinementParams;
use crate::decode::decode_refinement::decode_refinement;
use crate::decoder::{
    decode_i32_huffman_or_arith, decode_integer_context, decode_option_i32_huffman_or_arith,
    decode_u32_huffman_or_arith,
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
    if params.refinement && params.huffman {
        return Err(Jbig2Error::new("refinement with Huffman is not supported"));
    }
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
    let mut strip_t = -decode_i32_huffman_or_arith(
        params.huffman,
        || {
            let tables = huffman_tables.unwrap();
            tables.table_delta_t.decode(huffman_input.as_mut().unwrap())
        },
        "IADT",
        decoding_context,
    )?;
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
        strip_t += delta_t;
        let delta_first_s = decode_i32_huffman_or_arith(
            params.huffman,
            || {
                let tables = huffman_tables.unwrap();
                tables.table_first_s.decode(huffman_input.as_mut().unwrap())
            },
            "IAFS",
            decoding_context,
        )?;
        first_s += delta_first_s;
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
            let t = (params.strip_size as i32) * strip_t + current_t;
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
            let (final_symbol_width, final_symbol_height, final_symbol_bitmap) = if apply_refinement
            {
                let (rdw, rdh, rdx, rdy) = if let Some(ref huffman_tables) = params.huffman_tables {
                    // Use Huffman tables for refinement parameters
                    let rdw = huffman_tables
                        .table_refinement_dw
                        .as_ref()
                        .unwrap()
                        .decode(huffman_input.as_mut().unwrap())?;
                    let rdh = huffman_tables
                        .table_refinement_dh
                        .as_ref()
                        .unwrap()
                        .decode(huffman_input.as_mut().unwrap())?;
                    let rdx = huffman_tables
                        .table_refinement_dx
                        .as_ref()
                        .unwrap()
                        .decode(huffman_input.as_mut().unwrap())?;
                    let rdy = huffman_tables
                        .table_refinement_dy
                        .as_ref()
                        .unwrap()
                        .decode(huffman_input.as_mut().unwrap())?;
                    (rdw, rdh, rdx, rdy)
                } else {
                    // Use arithmetic decoding
                    let rdw = decode_integer_context(decoding_context, "IARDW")?.unwrap_or(0);
                    let rdh = decode_integer_context(decoding_context, "IARDH")?.unwrap_or(0);
                    let rdx = decode_integer_context(decoding_context, "IARDX")?.unwrap_or(0);
                    let rdy = decode_integer_context(decoding_context, "IARDY")?.unwrap_or(0);
                    (rdw, rdh, rdx, rdy)
                };
                let refined_width = (symbol_width as i32 + rdw) as usize;
                let refined_height = (symbol_height as i32 + rdh) as usize;
                let refined_bitmap = decode_refinement(
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
                (refined_width, refined_height, refined_bitmap)
            } else {
                (symbol_width, symbol_height, symbol_bitmap.clone())
            };
            let offset_t = t - if (params.reference_corner & 1) != 0 {
                0
            } else {
                final_symbol_height as i32 - 1
            };
            let offset_s = current_s
                - if (params.reference_corner & 2) != 0 {
                    final_symbol_width as i32 - 1
                } else {
                    0
                };
            // Draw the symbol with correct transformations and clipping
            for i in 0..final_symbol_height {
                let region_y = offset_t + i as i32;
                if region_y < 0 || region_y >= params.height as i32 {
                    continue;
                }
                for j in 0..final_symbol_width {
                    let region_x = offset_s + j as i32;
                    if region_x < 0 || region_x >= params.width as i32 {
                        continue;
                    }
                    let mut cx = j as i32;
                    let mut cy = final_symbol_height as i32 - 1 - i as i32;
                    if params.transposed {
                        cx = i as i32;
                        cy = final_symbol_height as i32 - 1 - j as i32;
                        if params.ds_offset < 0 {
                            cy += params.ds_offset;
                        } else if params.ds_offset > 0 {
                            cx += params.ds_offset;
                        }
                    } else {
                        cy += params.ds_offset;
                    }
                    if cx < 0
                        || cy < 0
                        || cx >= final_symbol_width as i32
                        || cy >= final_symbol_height as i32
                    {
                        continue;
                    }
                    let src_pixel = final_symbol_bitmap.get_pixel(cx as usize, cy as usize);
                    let dst_pixel = bitmap.get_pixel(region_x as usize, region_y as usize);
                    let new_pixel = apply_combination_operator(
                        dst_pixel,
                        src_pixel,
                        params.combination_operator as u8,
                    );
                    bitmap.set_pixel(region_x as usize, region_y as usize, new_pixel);
                }
            }
            i += 1;
            if i >= params.number_of_symbol_instances {
                break; // Processed all symbols
            }
            let delta_s = decode_option_i32_huffman_or_arith(
                params.huffman,
                || {
                    let tables = huffman_tables.unwrap();
                    tables.table_delta_s.decode(huffman_input.as_mut().unwrap())
                },
                "IADS",
                decoding_context,
            )?;
            if delta_s.is_none() {
                break; // OOB
            }
            let increment = if !params.transposed {
                if params.reference_corner > 1 {
                    final_symbol_width as i32 - 1
                } else {
                    0
                }
            } else if params.reference_corner & 1 != 0 {
                final_symbol_height as i32 - 1
            } else {
                0
            };
            current_s = current_s
                .wrapping_add(increment)
                .wrapping_add(delta_s.unwrap())
                .wrapping_add(params.ds_offset);
        }
    }
    Ok(bitmap)
}
