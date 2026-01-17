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

/// Inputs required to decode a text region.
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
    params: &TextRegionParams,
    decoding_context: &mut DecodingContext,
    mut huffman_input: Option<&mut Reader>,
) -> Result<Bitmap, Jbig2Error> {
    let trace_text = std::env::var_os("JBIG2_RS_TRACE_TEXT").is_some();
    let trace_text_verbose = std::env::var_os("JBIG2_RS_TRACE_TEXT_VERBOSE").is_some();
    let trace_text_limit = std::env::var("JBIG2_RS_TRACE_TEXT_LIMIT")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(50);
    let trace_text_every = std::env::var("JBIG2_RS_TRACE_TEXT_EVERY")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(1000);
    let trace_strip_match = std::env::var_os("JBIG2_RS_TRACE_TEXT_STRIP_MATCH").is_some();
    let trace_symbol_miss = std::env::var_os("JBIG2_RS_TRACE_TEXT_SYMBOL_MISS").is_some();
    let mut clipped_instances = 0u32;
    let mut outside_instances = 0u32;
    let mut refined_instances = 0u32;
    let mut oob_delta_s_count = 0u32;
    let mut total_strips = 0u32;
    let mut min_strip_instances = u32::MAX;
    let mut max_strip_instances = 0u32;
    let mut large_instances = 0u32;
    let mut min_delta_t = i32::MAX;
    let mut max_delta_t = i32::MIN;
    let mut neg_delta_t = 0u32;
    let mut min_delta_first_s = i32::MAX;
    let mut max_delta_first_s = i32::MIN;
    let mut neg_delta_first_s = 0u32;
    let mut min_delta_s = i32::MAX;
    let mut max_delta_s = i32::MIN;
    let mut neg_delta_s = 0u32;
    let mut symbol_use_counts = if trace_text {
        Some(vec![0u32; params.input_symbols.len()])
    } else {
        None
    };
    let mut symbol_black_counts: Option<Vec<u32>> = None;
    let mut symbol_extra_counts: Option<Vec<u32>> = None;
    let mut symbol_extra_totals: Option<Vec<u32>> = None;
    let mut symbol_ref_missing_counts: Option<Vec<u32>> = None;
    let mut symbol_ref_totals: Option<Vec<u32>> = None;
    let mut max_symbol_black = 0u32;
    let mut max_symbol_black_id = 0usize;
    let mut ranges_initialized = false;
    let mut min_s = 0i32;
    let mut max_s = 0i32;
    let mut min_t = 0i32;
    let mut max_t = 0i32;
    let mut min_x = 0i32;
    let mut max_x = 0i32;
    let mut min_y = 0i32;
    let mut max_y = 0i32;
    let mut min_symbol_id = 0usize;
    let mut max_symbol_id = 0usize;
    let mut ref_bitmap: Option<Bitmap> = None;
    if trace_text {
        if let Ok(path) = std::env::var("JBIG2_RS_TRACE_TEXT_REF") {
            match load_pbm_bitmap(&path) {
                Ok(bm) => {
                    if bm.width == params.width && bm.height == params.height {
                        ref_bitmap = Some(bm);
                    } else {
                        eprintln!(
                            "text_region: ref_bitmap size mismatch {}x{} (expected {}x{})",
                            bm.width, bm.height, params.width, params.height
                        );
                    }
                }
                Err(err) => {
                    eprintln!("text_region: failed to load ref_bitmap: {}", err);
                }
            }
        }
    }
    if trace_symbol_miss {
        if ref_bitmap.is_some() {
            symbol_extra_counts = Some(vec![0u32; params.input_symbols.len()]);
            symbol_extra_totals = Some(vec![0u32; params.input_symbols.len()]);
            symbol_ref_missing_counts = Some(vec![0u32; params.input_symbols.len()]);
            symbol_ref_totals = Some(vec![0u32; params.input_symbols.len()]);
        } else {
            eprintln!("text_region: symbol_miss requested without ref_bitmap");
        }
    }
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
    if trace_text {
        eprintln!(
            "text_region: huffman={} refinement={} size={}x{} default_pixel={} instances={} strip_size={} log_strip_size={} code_len={} transposed={} ref_corner={} comb_op={} ds_offset={} refine_template={} refine_at_len={} input_symbols={}",
            params.huffman,
            params.refinement,
            params.width,
            params.height,
            params.default_pixel_value,
            params.number_of_symbol_instances,
            params.strip_size,
            params.log_strip_size,
            params.symbol_code_length,
            params.transposed,
            params.reference_corner,
            params.combination_operator,
            params.ds_offset,
            params.refinement_template_index,
            params.refinement_at.len(),
            params.input_symbols.len()
        );
        let mut min_w = usize::MAX;
        let mut max_w = 0usize;
        let mut min_h = usize::MAX;
        let mut max_h = 0usize;
        let mut big_w = 0u32;
        let mut big_h = 0u32;
        let mut black_counts = Vec::with_capacity(params.input_symbols.len());
        for (idx, symbol) in params.input_symbols.iter().enumerate() {
            min_w = min_w.min(symbol.width);
            max_w = max_w.max(symbol.width);
            min_h = min_h.min(symbol.height);
            max_h = max_h.max(symbol.height);
            if symbol.width >= 500 {
                big_w = big_w.saturating_add(1);
            }
            if symbol.height >= 50 {
                big_h = big_h.saturating_add(1);
            }
            let black = symbol.count_black_pixels();
            if black > max_symbol_black {
                max_symbol_black = black;
                max_symbol_black_id = idx;
            }
            if symbol.width >= 500 {
                let total = (symbol.width.saturating_mul(symbol.height)) as u32;
                let fill_ppm = if total > 0 {
                    black.saturating_mul(1000) / total
                } else {
                    0
                };
                eprintln!(
                    "text_region: input_symbol[{}] size={}x{} black={} fill_ppm={}",
                    idx,
                    symbol.width,
                    symbol.height,
                    black,
                    fill_ppm
                );
            }
            black_counts.push(black);
        }
        eprintln!(
            "text_region: input_symbol_sizes min={}x{} max={}x{} big_w>=500={} big_h>=50={}",
            min_w, min_h, max_w, max_h, big_w, big_h
        );
        if max_symbol_black > 0 {
            eprintln!(
                "text_region: input_symbol_black max={} (id={})",
                max_symbol_black, max_symbol_black_id
            );
        }
        symbol_black_counts = Some(black_counts);
        if trace_text_verbose {
            for (idx, symbol) in params.input_symbols.iter().take(8).enumerate() {
                eprintln!(
                    "text_region: symbol[{}] size={}x{}",
                    idx, symbol.width, symbol.height
                );
            }
        }
    }
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
    if trace_text_verbose {
        eprintln!(
            "text_region: initial_strip_t={} strip_t={}",
            initial_strip_t, strip_t
        );
    }
    let mut first_s = 0i32;
    let mut i = 0;
    let mut strip_index = 0u32;
    let update_strip_stats = |count: u32,
                              total_strips: &mut u32,
                              min_strip_instances: &mut u32,
                              max_strip_instances: &mut u32| {
        *total_strips = total_strips.saturating_add(1);
        if count < *min_strip_instances {
            *min_strip_instances = count;
        }
        if count > *max_strip_instances {
            *max_strip_instances = count;
        }
    };
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
        if trace_text {
            min_delta_t = min_delta_t.min(delta_t);
            max_delta_t = max_delta_t.max(delta_t);
            if delta_t < 0 {
                neg_delta_t = neg_delta_t.saturating_add(1);
            }
        }
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
        if trace_text {
            min_delta_first_s = min_delta_first_s.min(delta_first_s);
            max_delta_first_s = max_delta_first_s.max(delta_first_s);
            if delta_first_s < 0 {
                neg_delta_first_s = neg_delta_first_s.saturating_add(1);
            }
        }
        let mut current_s = first_s;
        let mut strip_instances = 0u32;
        let mut strip_match_instances: Vec<(i32, i32, Bitmap)> = Vec::new();
        if trace_text_verbose {
            let huff_pos = huffman_input.as_ref().map(|r| r.get_position()).unwrap_or(0);
            let huff_shift = huffman_input.as_ref().map(|r| r.get_shift()).unwrap_or(0);
            eprintln!(
                "text_region: strip={} delta_t={} strip_t={} delta_first_s={} first_s={} huff_pos={} huff_shift={}",
                strip_index, delta_t, strip_t, delta_first_s, first_s, huff_pos, huff_shift
            );
        }
        strip_index = strip_index.saturating_add(1);
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
            let symbol_bitmap = params.input_symbols.get(symbol_id);
            let symbol_present = symbol_bitmap
                .map(|bm| bm.width > 0 && bm.height > 0)
                .unwrap_or(false);
            if !symbol_present && trace_text {
                let detail = if symbol_id < params.input_symbols.len() {
                    "empty symbol"
                } else {
                    "missing symbol"
                };
                eprintln!(
                    "text_region: {} id {} (available {}) at instance {} of {}",
                    detail,
                    symbol_id,
                    params.input_symbols.len(),
                    i + 1,
                    params.number_of_symbol_instances
                );
            }
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
            let mut refine_info = None;
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
                    if trace_text_verbose {
                        eprintln!(
                            "text_region: refine sym={} base={}x{} rdw={} rdh={} rdx={} rdy={} refined={}x{}",
                            symbol_id,
                            base_width,
                            base_height,
                            rdw,
                            rdh,
                            rdx,
                            rdy,
                            refined_width,
                            refined_height
                        );
                    }
                    let bitmap = if params.huffman {
                        let current_pos = huffman_input.as_ref().unwrap().get_position();
                        let data = huffman_input.as_ref().unwrap().get_data();
                        let mut temp_context =
                            DecodingContext::new(data.to_vec(), current_pos, data.len());
                        decode_refinement(
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
                                at: params.refinement_at.clone(),
                            },
                            decoding_context,
                        )?
                    };
                    if let Some(size) = bmsize {
                        huffman_input.as_mut().unwrap().skip(size as usize);
                    }
                    refine_info = Some((rdw, rdh, rdx, rdy));
                    (refined_width, refined_height, Some(bitmap))
                } else {
                    (symbol_width, symbol_height, Some(symbol_bitmap.clone()))
                }
            } else {
                if apply_refinement {
                    if let Some(size) = bmsize {
                        huffman_input.as_mut().unwrap().skip(size as usize);
                    } else if trace_text {
                        eprintln!(
                            "text_region: missing symbol refinement skipped id={} inst={}/{}",
                            symbol_id,
                            i + 1,
                            params.number_of_symbol_instances
                        );
                    }
                }
                (0, 0, None)
            };
            let (symbol_width, symbol_height, width_adjust, height_adjust) =
                if final_symbol_bitmap.is_some() {
                    let symbol_width = i32::try_from(final_symbol_width)
                        .map_err(|_| Jbig2Error::new("symbol width overflow"))?;
                    let symbol_height = i32::try_from(final_symbol_height)
                        .map_err(|_| Jbig2Error::new("symbol height overflow"))?;
                    let width_adjust = symbol_width - 1;
                    let height_adjust = symbol_height - 1;
                    (symbol_width, symbol_height, width_adjust, height_adjust)
                } else {
                    (0, 0, 0, 0)
                };

            let mut s = current_s;
            let base_s = s;
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

            if trace_text {
                if !ranges_initialized {
                    min_s = base_s;
                    max_s = base_s;
                    min_t = t;
                    max_t = t;
                    min_x = x;
                    max_x = x;
                    min_y = y;
                    max_y = y;
                    min_symbol_id = symbol_id;
                    max_symbol_id = symbol_id;
                    ranges_initialized = true;
                } else {
                    min_s = min_s.min(base_s);
                    max_s = max_s.max(base_s);
                    min_t = min_t.min(t);
                    max_t = max_t.max(t);
                    min_x = min_x.min(x);
                    max_x = max_x.max(x);
                    min_y = min_y.min(y);
                    max_y = max_y.max(y);
                    min_symbol_id = min_symbol_id.min(symbol_id);
                    max_symbol_id = max_symbol_id.max(symbol_id);
                }
                if apply_refinement {
                    refined_instances = refined_instances.saturating_add(1);
                }
                if symbol_present {
                    if let Some(counts) = symbol_use_counts.as_mut() {
                        counts[symbol_id] = counts[symbol_id].saturating_add(1);
                    }
                }
            }

            if trace_text && final_symbol_bitmap.is_some() {
                let x_i64 = x as i64;
                let y_i64 = y as i64;
                let w_i64 = symbol_width as i64;
                let h_i64 = symbol_height as i64;
                let region_w = params.width as i64;
                let region_h = params.height as i64;
                let x_end = x_i64 + w_i64;
                let y_end = y_i64 + h_i64;
                if x_end <= 0 || y_end <= 0 || x_i64 >= region_w || y_i64 >= region_h {
                    outside_instances = outside_instances.saturating_add(1);
                } else if x_i64 < 0 || y_i64 < 0 || x_end > region_w || y_end > region_h {
                    clipped_instances = clipped_instances.saturating_add(1);
                }
            }

            if trace_text
                && final_symbol_bitmap.is_some()
                && (final_symbol_width >= 500 || final_symbol_height >= 10)
            {
                large_instances = large_instances.saturating_add(1);
                let final_symbol_bitmap = final_symbol_bitmap.as_ref().unwrap();
                if final_symbol_width >= 500 {
                    let black = symbol_black_counts
                        .as_ref()
                        .and_then(|counts| counts.get(symbol_id))
                        .copied()
                        .unwrap_or(0);
                    let (row_min, row_max, full_rows) = final_symbol_bitmap.row_black_stats();
                    let total = (final_symbol_width.saturating_mul(final_symbol_height)) as u32;
                    let fill_ppm = if total > 0 {
                        black.saturating_mul(1000) / total
                    } else {
                        0
                    };
                    let mut black_missing = 0u32;
                    let mut black_outside = 0u32;
                    let mut black_total = 0u32;
                    let mut best_dy = 0i32;
                    let mut best_missing_ppm = 1000u32;
                    if let Some(ref ref_bm) = ref_bitmap {
                        for sy in 0..final_symbol_height {
                            for sx in 0..final_symbol_width {
                                if final_symbol_bitmap.get_pixel(sx, sy) == 0 {
                                    continue;
                                }
                                black_total = black_total.saturating_add(1);
                                let rx = x + sx as i32;
                                let ry = y + sy as i32;
                                if rx < 0
                                    || ry < 0
                                    || (rx as usize) >= ref_bm.width
                                    || (ry as usize) >= ref_bm.height
                                {
                                    black_outside = black_outside.saturating_add(1);
                                    continue;
                                }
                                if ref_bm.get_pixel(rx as usize, ry as usize) == 0 {
                                    black_missing = black_missing.saturating_add(1);
                                }
                            }
                        }
                        for dy in -12..=12 {
                            let mut miss = 0u32;
                            let mut out = 0u32;
                            let mut total = 0u32;
                            for sy in 0..final_symbol_height {
                                for sx in 0..final_symbol_width {
                                    if final_symbol_bitmap.get_pixel(sx, sy) == 0 {
                                        continue;
                                    }
                                    total = total.saturating_add(1);
                                    let rx = x + sx as i32;
                                    let ry = y + dy + sy as i32;
                                    if rx < 0
                                        || ry < 0
                                        || (rx as usize) >= ref_bm.width
                                        || (ry as usize) >= ref_bm.height
                                    {
                                        out = out.saturating_add(1);
                                        continue;
                                    }
                                    if ref_bm.get_pixel(rx as usize, ry as usize) == 0 {
                                        miss = miss.saturating_add(1);
                                    }
                                }
                            }
                            let ppm = if total > out {
                                miss.saturating_mul(1000) / (total.saturating_sub(out))
                            } else {
                                1000
                            };
                            if ppm < best_missing_ppm {
                                best_missing_ppm = ppm;
                                best_dy = dy;
                            }
                        }
                    }
                    let missing_ppm = if black_total > black_outside {
                        black_missing
                            .saturating_mul(1000)
                            / (black_total.saturating_sub(black_outside))
                    } else {
                        0
                    };
                    eprintln!(
                        "text_region: wide inst={} sym={} t={} x={} y={} size={}x{} black={} fill_ppm={} row_min={} row_max={} full_rows={} missing_ppm={} outside_black={} best_dy={} best_missing_ppm={}",
                        i + 1,
                        symbol_id,
                        t,
                        x,
                        y,
                        final_symbol_width,
                        final_symbol_height,
                        black,
                        fill_ppm,
                        row_min,
                        row_max,
                        full_rows,
                        missing_ppm,
                        black_outside,
                        best_dy,
                        best_missing_ppm
                    );
                }
                if trace_text_verbose && final_symbol_height >= 20 {
                    eprintln!(
                        "text_region: tall inst={} sym={} t={} x={} y={} size={}x{}",
                        i + 1,
                        symbol_id,
                        t,
                        x,
                        y,
                        final_symbol_width,
                        final_symbol_height
                    );
                }
            }

            if trace_symbol_miss && final_symbol_bitmap.is_some() {
                let final_symbol_bitmap = final_symbol_bitmap.as_ref().unwrap();
                if let (
                    Some(ref ref_bm),
                    Some(ref mut extra),
                    Some(ref mut extra_totals),
                    Some(ref mut ref_missing),
                    Some(ref mut ref_totals),
                ) = (
                    ref_bitmap.as_ref(),
                    symbol_extra_counts.as_mut(),
                    symbol_extra_totals.as_mut(),
                    symbol_ref_missing_counts.as_mut(),
                    symbol_ref_totals.as_mut(),
                ) {
                    let mut extra_count = 0u32;
                    let mut extra_total = 0u32;
                    let mut missing_count = 0u32;
                    let mut missing_total = 0u32;
                    for sy in 0..final_symbol_height {
                        for sx in 0..final_symbol_width {
                            let rx = x + sx as i32;
                            let ry = y + sy as i32;
                            if rx < 0
                                || ry < 0
                                || (rx as usize) >= ref_bm.width
                                || (ry as usize) >= ref_bm.height
                            {
                                continue;
                            }
                            let ref_black = ref_bm.get_pixel(rx as usize, ry as usize) != 0;
                            let sym_black = final_symbol_bitmap.get_pixel(sx, sy) != 0;
                            if sym_black {
                                extra_total = extra_total.saturating_add(1);
                                if !ref_black {
                                    extra_count = extra_count.saturating_add(1);
                                }
                            }
                            if ref_black {
                                missing_total = missing_total.saturating_add(1);
                                if !sym_black {
                                    missing_count = missing_count.saturating_add(1);
                                }
                            }
                        }
                    }
                    extra[symbol_id] = extra[symbol_id].saturating_add(extra_count);
                    extra_totals[symbol_id] = extra_totals[symbol_id].saturating_add(extra_total);
                    ref_missing[symbol_id] =
                        ref_missing[symbol_id].saturating_add(missing_count);
                    ref_totals[symbol_id] = ref_totals[symbol_id].saturating_add(missing_total);
                }
            }

            if trace_text_verbose {
                let should_log = i < trace_text_limit
                    || (trace_text_every > 0 && i % trace_text_every == 0)
                    || i + 1 == params.number_of_symbol_instances;
                if should_log {
                    let huff_pos = huffman_input.as_ref().map(|r| r.get_position()).unwrap_or(0);
                    let huff_shift = huffman_input.as_ref().map(|r| r.get_shift()).unwrap_or(0);
                    if let Some((rdw, rdh, rdx, rdy)) = refine_info {
                        eprintln!(
                            "text_region: inst={} sym={} t={} s0={} s_adj={} x={} y={} size={}x{} refine=1 rdw={} rdh={} rdx={} rdy={} huff_pos={} huff_shift={}",
                            i + 1,
                            symbol_id,
                            t,
                            base_s,
                            s,
                            x,
                            y,
                            final_symbol_width,
                            final_symbol_height,
                            rdw,
                            rdh,
                            rdx,
                            rdy,
                            huff_pos,
                            huff_shift
                        );
                    } else {
                        eprintln!(
                            "text_region: inst={} sym={} t={} s0={} s_adj={} x={} y={} size={}x{} refine=0 huff_pos={} huff_shift={}",
                            i + 1,
                            symbol_id,
                            t,
                            base_s,
                            s,
                            x,
                            y,
                            final_symbol_width,
                            final_symbol_height,
                            huff_pos,
                            huff_shift
                        );
                    }
                }
            }

            if let Some(final_symbol_bitmap) = final_symbol_bitmap.as_ref() {
                bitmap.combine(
                    final_symbol_bitmap,
                    x as isize,
                    y as isize,
                    params.combination_operator as u8,
                );
                if trace_strip_match {
                    strip_match_instances.push((x, y, final_symbol_bitmap.clone()));
                }
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
            strip_instances += 1;
            if i >= params.number_of_symbol_instances {
                if trace_text {
                    update_strip_stats(
                        strip_instances,
                        &mut total_strips,
                        &mut min_strip_instances,
                        &mut max_strip_instances,
                    );
                }
                if trace_strip_match {
                    if let Some(ref ref_bm) = ref_bitmap {
                        let mut best_dy = 0i32;
                        let mut best_score = u32::MAX;
                        for dy in -16..=16 {
                            let mut missing = 0u32;
                            let mut outside = 0u32;
                            for (sx, sy, inst_bm) in &strip_match_instances {
                                for yy in 0..inst_bm.height {
                                    for xx in 0..inst_bm.width {
                                        if inst_bm.get_pixel(xx, yy) == 0 {
                                            continue;
                                        }
                                        let rx = *sx + xx as i32;
                                        let ry = *sy + dy + yy as i32;
                                        if rx < 0
                                            || ry < 0
                                            || (rx as usize) >= ref_bm.width
                                            || (ry as usize) >= ref_bm.height
                                        {
                                            outside = outside.saturating_add(1);
                                            continue;
                                        }
                                        if ref_bm.get_pixel(rx as usize, ry as usize) == 0 {
                                            missing = missing.saturating_add(1);
                                        }
                                    }
                                }
                            }
                            let score = missing.saturating_add(outside);
                            if score < best_score {
                                best_score = score;
                                best_dy = dy;
                            }
                        }
                        if best_dy != 0 {
                            eprintln!(
                                "text_region: strip_match strip={} strip_t={} best_dy={} score={}",
                                strip_index.saturating_sub(1),
                                strip_t,
                                best_dy,
                                best_score
                            );
                        }
                    }
                }
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
                if trace_text_verbose {
                    eprintln!(
                        "text_region: inst={} delta_s=OOB break",
                        i
                    );
                }
                if trace_text {
                    oob_delta_s_count = oob_delta_s_count.saturating_add(1);
                    update_strip_stats(
                        strip_instances,
                        &mut total_strips,
                        &mut min_strip_instances,
                        &mut max_strip_instances,
                    );
                }
                if trace_strip_match {
                    if let Some(ref ref_bm) = ref_bitmap {
                        let mut best_dy = 0i32;
                        let mut best_score = u32::MAX;
                        for dy in -16..=16 {
                            let mut missing = 0u32;
                            let mut outside = 0u32;
                            for (sx, sy, inst_bm) in &strip_match_instances {
                                for yy in 0..inst_bm.height {
                                    for xx in 0..inst_bm.width {
                                        if inst_bm.get_pixel(xx, yy) == 0 {
                                            continue;
                                        }
                                        let rx = *sx + xx as i32;
                                        let ry = *sy + dy + yy as i32;
                                        if rx < 0
                                            || ry < 0
                                            || (rx as usize) >= ref_bm.width
                                            || (ry as usize) >= ref_bm.height
                                        {
                                            outside = outside.saturating_add(1);
                                            continue;
                                        }
                                        if ref_bm.get_pixel(rx as usize, ry as usize) == 0 {
                                            missing = missing.saturating_add(1);
                                        }
                                    }
                                }
                            }
                            let score = missing.saturating_add(outside);
                            if score < best_score {
                                best_score = score;
                                best_dy = dy;
                            }
                        }
                        if best_dy != 0 {
                            eprintln!(
                                "text_region: strip_match strip={} strip_t={} best_dy={} score={}",
                                strip_index.saturating_sub(1),
                                strip_t,
                                best_dy,
                                best_score
                            );
                        }
                    }
                }
                break; // OOB
            }
            let delta_s = delta_s.unwrap();
            if trace_text {
                min_delta_s = min_delta_s.min(delta_s);
                max_delta_s = max_delta_s.max(delta_s);
                if delta_s < 0 {
                    neg_delta_s = neg_delta_s.saturating_add(1);
                }
            }
            current_s = current_s
                .wrapping_add(delta_s)
                .wrapping_add(params.ds_offset);
        }
    }
    if trace_text {
        let (used_symbols, unused_symbols) = if let Some(counts) = symbol_use_counts.as_ref() {
            let mut used = 0u32;
            let mut unused = 0u32;
            let mut used_once = 0u32;
            let mut used_more = 0u32;
            let mut max_use = 0u32;
            let mut max_use_id = 0usize;
            for (idx, &count) in counts.iter().enumerate() {
                if count == 0 {
                    unused += 1;
                } else {
                    used += 1;
                    if count == 1 {
                        used_once += 1;
                    } else {
                        used_more += 1;
                    }
                    if count > max_use {
                        max_use = count;
                        max_use_id = idx;
                    }
                }
            }
            if used > 0 {
                eprintln!(
                    "text_region: symbol_use used_once={} used_more={} max_use={} (id={})",
                    used_once, used_more, max_use, max_use_id
                );
            }
            (used, unused)
        } else {
            (0, 0)
        };
        eprintln!(
            "text_region: clipped_instances={} outside_instances={} refined_instances={}",
            clipped_instances, outside_instances, refined_instances
        );
        if min_delta_t != i32::MAX {
            eprintln!(
                "text_region: delta_t range=[{}, {}] neg_delta_t={}",
                min_delta_t, max_delta_t, neg_delta_t
            );
        }
        if min_delta_first_s != i32::MAX {
            eprintln!(
                "text_region: delta_first_s range=[{}, {}] neg_delta_first_s={}",
                min_delta_first_s, max_delta_first_s, neg_delta_first_s
            );
        }
        if min_delta_s != i32::MAX {
            eprintln!(
                "text_region: delta_s range=[{}, {}] neg_delta_s={}",
                min_delta_s, max_delta_s, neg_delta_s
            );
        }
        if total_strips > 0 {
            let min_instances = if min_strip_instances == u32::MAX {
                0
            } else {
                min_strip_instances
            };
            eprintln!(
                "text_region: strips={} min_strip_instances={} max_strip_instances={} oob_delta_s={} large_instances={}",
                total_strips,
                min_instances,
                max_strip_instances,
                oob_delta_s_count,
                large_instances
            );
        }
        if trace_symbol_miss {
            if let (
                Some(ref_missing),
                Some(ref_totals),
                Some(extra),
                Some(extra_totals),
            ) = (
                symbol_ref_missing_counts.as_ref(),
                symbol_ref_totals.as_ref(),
                symbol_extra_counts.as_ref(),
                symbol_extra_totals.as_ref(),
            ) {
                let summary: Vec<(u64, u32, u32, u64, u32, u32, usize)> = ref_totals
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, &total)| {
                        let miss = ref_missing[idx];
                        let extra_count = extra[idx];
                        let extra_total = extra_totals[idx];
                        if total == 0 && extra_total == 0 {
                            return None;
                        }
                        let miss_ppm = (miss as u64).saturating_mul(1000) / total.max(1) as u64;
                        let extra_ppm =
                            (extra_count as u64).saturating_mul(1000) / extra_total.max(1) as u64;
                        Some((miss_ppm, miss, total, extra_ppm, extra_count, extra_total, idx))
                    })
                    .collect();
                let mut by_ppm = summary.clone();
                by_ppm.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));
                let mut shown = 0u32;
                for (miss_ppm, miss, total, extra_ppm, extra_count, extra_total, idx) in by_ppm {
                    if shown >= 12 {
                        break;
                    }
                    if miss == 0 && extra_count == 0 {
                        continue;
                    }
                    let black = symbol_black_counts
                        .as_ref()
                        .and_then(|counts| counts.get(idx))
                        .copied()
                        .unwrap_or(0);
                    let (sym_w, sym_h) = params
                        .input_symbols
                        .get(idx)
                        .map(|bm| (bm.width, bm.height))
                        .unwrap_or((0, 0));
                    let use_count = symbol_use_counts
                        .as_ref()
                        .and_then(|counts| counts.get(idx))
                        .copied()
                        .unwrap_or(0);
                    eprintln!(
                        "text_region: symbol_miss_ppm sym={} size={}x{} use_count={} ref_missing={} ref_total={} ref_ppm={} extra={} extra_total={} extra_ppm={} black={}",
                        idx,
                        sym_w,
                        sym_h,
                        use_count,
                        miss,
                        total,
                        miss_ppm,
                        extra_count,
                        extra_total,
                        extra_ppm,
                        black
                    );
                    shown = shown.saturating_add(1);
                }
                let mut by_missing = summary;
                by_missing.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| b.0.cmp(&a.0)));
                let mut shown = 0u32;
                for (miss_ppm, miss, total, extra_ppm, extra_count, extra_total, idx) in by_missing {
                    if shown >= 12 {
                        break;
                    }
                    if miss == 0 && extra_count == 0 {
                        continue;
                    }
                    let black = symbol_black_counts
                        .as_ref()
                        .and_then(|counts| counts.get(idx))
                        .copied()
                        .unwrap_or(0);
                    let (sym_w, sym_h) = params
                        .input_symbols
                        .get(idx)
                        .map(|bm| (bm.width, bm.height))
                        .unwrap_or((0, 0));
                    let use_count = symbol_use_counts
                        .as_ref()
                        .and_then(|counts| counts.get(idx))
                        .copied()
                        .unwrap_or(0);
                    eprintln!(
                        "text_region: symbol_miss_count sym={} size={}x{} use_count={} ref_missing={} ref_total={} ref_ppm={} extra={} extra_total={} extra_ppm={} black={}",
                        idx,
                        sym_w,
                        sym_h,
                        use_count,
                        miss,
                        total,
                        miss_ppm,
                        extra_count,
                        extra_total,
                        extra_ppm,
                        black
                    );
                    shown = shown.saturating_add(1);
                }
            }
        }
        if ranges_initialized {
            eprintln!(
                "text_region: s_range=[{}, {}] t_range=[{}, {}] x_range=[{}, {}] y_range=[{}, {}] symbol_id_range=[{}, {}] used_symbols={} unused_symbols={}",
                min_s,
                max_s,
                min_t,
                max_t,
                min_x,
                max_x,
                min_y,
                max_y,
                min_symbol_id,
                max_symbol_id,
                used_symbols,
                unused_symbols
            );
        }
    }
    Ok(bitmap)
}

fn load_pbm_bitmap(path: &str) -> Result<Bitmap, Jbig2Error> {
    let data = std::fs::read(path)
        .map_err(|e| Jbig2Error::new(&format!("read {} failed: {}", path, e)))?;
    if !data.starts_with(b"P4") {
        return Err(Jbig2Error::new("unsupported PBM format"));
    }
    let mut idx = 2;
    let mut values = Vec::new();
    while values.len() < 2 {
        while idx < data.len() {
            let b = data[idx];
            if b == b'#' {
                while idx < data.len() && data[idx] != b'\n' {
                    idx += 1;
                }
                continue;
            }
            if b.is_ascii_whitespace() {
                idx += 1;
                continue;
            }
            break;
        }
        if idx >= data.len() {
            return Err(Jbig2Error::new("PBM header truncated"));
        }
        let start = idx;
        while idx < data.len() && !data[idx].is_ascii_whitespace() {
            idx += 1;
        }
        let num = std::str::from_utf8(&data[start..idx])
            .map_err(|_| Jbig2Error::new("PBM header invalid"))?
            .parse::<usize>()
            .map_err(|_| Jbig2Error::new("PBM header invalid"))?;
        values.push(num);
    }
    while idx < data.len() && data[idx].is_ascii_whitespace() {
        idx += 1;
    }
    if values.len() != 2 {
        return Err(Jbig2Error::new("PBM header missing dimensions"));
    }
    let width = values[0];
    let height = values[1];
    let stride = (width + 7) / 8;
    let expected = stride * height;
    if data.len() < idx + expected {
        return Err(Jbig2Error::new("PBM data truncated"));
    }
    let mut bitmap = Bitmap::new(width, height);
    bitmap.data.copy_from_slice(&data[idx..idx + expected]);
    Ok(bitmap)
}
