use crate::bitmap::Bitmap;
use crate::contexts::DecodingContext;
use crate::decoder::{decode_integer_context, decode_iaid_context};
use crate::error::Jbig2Error;

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
}

pub fn decode_text_region(
    params: &TextRegionParams,
    decoding_context: &mut DecodingContext,
) -> Result<Bitmap, Jbig2Error> {
    // Prepare bitmap
    let mut bitmap = Bitmap::new(params.width, params.height);
    if params.default_pixel_value != 0 {
        for y in 0..params.height {
            for x in 0..params.width {
                bitmap.set_pixel(x, y, 1);
            }
        }
    }
    let mut strip_t = -(decode_integer_context(decoding_context, "IADT")?.unwrap_or(0) as i32);
    let mut first_s = 0i32;
    let mut i = 0;
    while i < params.number_of_symbol_instances {
        let delta_t = decode_integer_context(decoding_context, "IADT")?.unwrap_or(0);
        strip_t += delta_t as i32;
        let delta_first_s = decode_integer_context(decoding_context, "IAFS")?.unwrap_or(0);
        first_s += delta_first_s as i32;
        let mut current_s = first_s;
        loop {
            let mut current_t = 0i32;
            if params.strip_size > 1 {
                current_t = decode_integer_context(decoding_context, "IAIT")?.unwrap_or(0) as i32;
            }
            let t = (params.strip_size as i32) * strip_t + current_t;
            let symbol_id = decode_iaid_context(decoding_context, params.symbol_code_length)?;
            let symbol_id = symbol_id as usize;
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
                // For now, skip refinement - would need additional parameters
                (symbol_width, symbol_height, symbol_bitmap.clone())
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
            let delta_s = decode_integer_context(decoding_context, "IADS")?;
            if delta_s.is_none() {
                break; // OOB
            }
            current_s += increment + delta_s.unwrap() as i32 + params.ds_offset;
        }
    }
    Ok(bitmap)
}