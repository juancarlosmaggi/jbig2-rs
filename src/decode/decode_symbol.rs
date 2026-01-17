use crate::bitmap::Bitmap;
use crate::contexts::DecodingContext;
use crate::decode::decode_mmr::decode_mmr_bitmap;
use crate::decode::decode_symbol_helpers::{
    AggregateSymbolParams, decode_aggregate_symbol, split_collective_bitmap,
};
use crate::decode::decode_utils::read_uncompressed_bitmap;
use crate::decoder::{decode_iaid_context, decode_integer_context};
use crate::error::Jbig2Error;
use crate::huffman::{SymbolDictionaryHuffmanTables, get_standard_table};
use crate::reader::Reader;
use crate::validation;

/// Inputs required to decode a symbol dictionary segment.
#[derive(Clone)]
pub struct SymbolDictionaryParams {
    pub huffman: bool,
    pub refinement: bool,
    pub symbols: Vec<Bitmap>,
    pub number_of_new_symbols: usize,
    pub number_of_exported_symbols: usize,
    pub template_index: usize,
    pub at: Vec<(i8, i8)>,
    pub refinement_template_index: usize,
    pub refinement_at: Vec<(i8, i8)>,
    pub huffman_tables: Option<SymbolDictionaryHuffmanTables>,
}

/// Decode a symbol dictionary and return the exported symbols.
pub fn decode_symbol_dictionary(
    params: &SymbolDictionaryParams,
    decoding_context: &mut DecodingContext,
    mut huffman_input: Option<&mut Reader>,
) -> Result<Vec<Bitmap>, Jbig2Error> {
    let trace_symbol = std::env::var_os("JBIG2_RS_TRACE_SYMBOL").is_some();
    let mut trace_height_classes = 0u32;
    let mut min_delta_height = i32::MAX;
    let mut max_delta_height = i32::MIN;
    let mut neg_delta_height = 0u32;
    let mut min_delta_width = i32::MAX;
    let mut max_delta_width = i32::MIN;
    let mut neg_delta_width = 0u32;
    let mut min_symbol_width = usize::MAX;
    let mut max_symbol_width = 0usize;
    let mut min_symbol_height = usize::MAX;
    let mut max_symbol_height = 0usize;
    if params.number_of_new_symbols == 0 {
        // return Err(Jbig2Error::new("number of new symbols must be positive"));
    }

    // Validate that Huffman tables are provided when Huffman mode is enabled.
    if params.huffman && params.huffman_tables.is_none() {
        return Err(Jbig2Error::new(
            "Huffman tables required when Huffman mode is enabled",
        ));
    }

    if params.number_of_new_symbols == 0 {
        return Err(Jbig2Error::new("number of new symbols must be positive"));
    }

    validation::validate_symbol_decode_params(params.template_index, params.number_of_new_symbols)?;

    let mut new_symbols = Vec::with_capacity(params.number_of_new_symbols);
    let mut current_height: i32 = 0;

    let total_symbols = params.symbols.len() + params.number_of_new_symbols;
    let symbol_code_length = if total_symbols <= 1 {
        0
    } else {
        ((total_symbols as u64 - 1).ilog2() as usize) + 1
    };

    let huffman = params.huffman;
    let refinement = params.refinement;
    let huffman_tables = params.huffman_tables.as_ref();
    let huff_refine_delta = if huffman && refinement {
        Some(get_standard_table(15)?)
    } else {
        None
    };
    let huff_refine_size = if huffman && refinement {
        Some(get_standard_table(1)?)
    } else {
        None
    };
    let huff_export_run = if huffman {
        Some(get_standard_table(1)?)
    } else {
        None
    };

    while new_symbols.len() < params.number_of_new_symbols {
        // Decode the delta for the next height class.
        let delta_height: i32 = if huffman {
            let tables = huffman_tables.unwrap();
            let (val, oob) = tables
                .table_delta_height
                .decode_entry(huffman_input.as_mut().unwrap())?;
            if oob {
                return Err(Jbig2Error::new(
                    "OOB when decoding height class delta",
                ));
            }
            val
        } else {
            match decode_integer_context(decoding_context, "IADH")? {
                Some(v) => v,
                None => {
                    return Err(Jbig2Error::new(
                        "OOB when decoding height class delta",
                    ))
                }
            }
        };

        current_height = current_height
            .checked_add(delta_height)
            .ok_or_else(|| Jbig2Error::new("height class overflow"))?;
        if trace_symbol {
            min_delta_height = min_delta_height.min(delta_height);
            max_delta_height = max_delta_height.max(delta_height);
            if delta_height < 0 {
                neg_delta_height = neg_delta_height.saturating_add(1);
            }
        }
        if current_height < 0 {
            return Err(Jbig2Error::new("invalid height class value"));
        }
        if current_height == 0 {
            continue;
        }

        let mut current_width: i32 = 0;
        let mut total_width: i32 = 0;
        let mut symbol_widths: Vec<usize> = Vec::new();
        let current_height_usize = current_height as usize;

        loop {
            // Decode the delta width for this height class.
            let dw: i32 = if huffman {
                let tables = huffman_tables.unwrap();
                let (val, oob) = tables
                    .table_delta_width
                    .decode_entry(huffman_input.as_mut().unwrap())?;
                if oob {
                    break;
                }
                val
            } else {
                match decode_integer_context(decoding_context, "IADW")? {
                    Some(v) => v,
                    None => break, // OOB – end of height class
                }
            };

            current_width = current_width
                .checked_add(dw)
                .ok_or_else(|| Jbig2Error::new("symbol width overflow"))?;
            if trace_symbol {
                min_delta_width = min_delta_width.min(dw);
                max_delta_width = max_delta_width.max(dw);
                if dw < 0 {
                    neg_delta_width = neg_delta_width.saturating_add(1);
                }
            }
            if current_width < 0 {
                return Err(Jbig2Error::new(
                    "DW value would make symbol width negative",
                ));
            }
            total_width = total_width
                .checked_add(current_width)
                .ok_or_else(|| Jbig2Error::new("total width overflow"))?;

            let current_width_usize = current_width as usize;
            if trace_symbol {
                min_symbol_width = min_symbol_width.min(current_width_usize);
                max_symbol_width = max_symbol_width.max(current_width_usize);
                min_symbol_height = min_symbol_height.min(current_height_usize);
                max_symbol_height = max_symbol_height.max(current_height_usize);
            }
            if refinement {
                // Decode the instance count for refinement aggregation.
                let instances = if huffman {
                    let tables = huffman_tables.unwrap();
                    let (val, oob) = tables
                        .table_aggregate_instances
                        .decode_entry(huffman_input.as_mut().unwrap())?;
                    if oob {
                        break; // treat OOB as end of height class (remaining exported later)
                    }
                    val as usize + 1
                } else {
                    decode_integer_context(decoding_context, "IAAI")?
                        .map(|v| v + 1)
                        .unwrap_or(1) as usize
                };

                if instances == 1 {
                    let (symbol_id, rdx, rdy, bmsize) = if huffman {
                        let reader = huffman_input
                            .as_mut()
                            .ok_or_else(|| Jbig2Error::new("missing Huffman input"))?;
                        let symbol_id = reader.read_bits(symbol_code_length as u32)? as usize;
                        let delta_table = huff_refine_delta.as_ref().unwrap();
                        let rdx = delta_table.decode(reader)?;
                        let rdy = delta_table.decode(reader)?;
                        let size_table = huff_refine_size.as_ref().unwrap();
                        let bmsize = size_table.decode(reader)?;
                        if bmsize < 0 {
                            return Err(Jbig2Error::new("invalid refinement bitmap size"));
                        }
                        reader.byte_align();
                        (symbol_id, rdx, rdy, Some(bmsize as usize))
                    } else {
                        let symbol_id =
                            decode_iaid_context(decoding_context, symbol_code_length)? as usize;
                        let rdx = decode_integer_context(decoding_context, "IARDX")?.unwrap_or(0);
                        let rdy = decode_integer_context(decoding_context, "IARDY")?.unwrap_or(0);
                        (symbol_id, rdx, rdy, None)
                    };

                    let total_symbols = params.symbols.len() + new_symbols.len();
                    if symbol_id >= total_symbols {
                        return Err(Jbig2Error::new("invalid refinement symbol id"));
                    }
                    let sym = if symbol_id < params.symbols.len() {
                        &params.symbols[symbol_id]
                    } else {
                        &new_symbols[symbol_id - params.symbols.len()]
                    };

                    if current_width <= 0 || current_height <= 0 {
                        return Err(Jbig2Error::new("invalid refinement symbol dimensions"));
                    }
                    let width = current_width as usize;
                    let height = current_height as usize;

                    let bitmap = if huffman {
                        let reader = huffman_input
                            .as_mut()
                            .ok_or_else(|| Jbig2Error::new("missing Huffman input"))?;
                        let current_pos = reader.get_position();
                        let data = reader.get_data();
                        let mut temp_context =
                            DecodingContext::new(data.to_vec(), current_pos, data.len());
                        crate::decode::decode_refinement::decode_refinement(
                            &crate::decode::decode_refinement::RefinementParams {
                                width,
                                height,
                                template_index: params.refinement_template_index,
                                reference_bitmap: sym,
                                offset_x: rdx,
                                offset_y: rdy,
                                prediction: false,
                                at: params.refinement_at.clone(),
                            },
                            &mut temp_context,
                        )?
                    } else {
                        crate::decode::decode_refinement::decode_refinement(
                            &crate::decode::decode_refinement::RefinementParams {
                                width,
                                height,
                                template_index: params.refinement_template_index,
                                reference_bitmap: sym,
                                offset_x: rdx,
                                offset_y: rdy,
                                prediction: false,
                                at: params.refinement_at.clone(),
                            },
                            decoding_context,
                        )?
                    };

                    new_symbols.push(bitmap);

                    if let Some(mut bmsize) = bmsize {
                        if bmsize == 0 {
                            let stride = (width + 7) >> 3;
                            bmsize = height * stride;
                        }
                        huffman_input.as_mut().unwrap().skip(bmsize);
                    }
                } else {
                    // Decode an aggregate symbol from multiple instances.
                    let agg_params = AggregateSymbolParams {
                        current_width,
                        current_height,
                        number_of_instances: instances as i32,
                        symbol_code_length,
                        refinement: true,
                        refinement_template_index: params.refinement_template_index,
                        refinement_at: params.refinement_at.clone(),
                        huffman,
                    };
                    let bitmap = decode_aggregate_symbol(
                        &agg_params,
                        &params.symbols,
                        &new_symbols,
                        decoding_context,
                        huffman_input.as_mut().map(|reader| &mut **reader),
                    )?;
                    new_symbols.push(bitmap);
                }
            } else if huffman {
                symbol_widths.push(current_width_usize);
            } else {
                // Decode a directly coded symbol bitmap.
                let bitmap = crate::decode::decode_generic::decode_bitmap(
                    &crate::decode::decode_generic::DecodeBitmapParams {
                        mmr: false,
                        width: current_width_usize,
                        height: current_height_usize,
                        template_index: params.template_index,
                        prediction: false,
                        skip: None,
                        at: params.at.clone(),
                    },
                    decoding_context,
                )?;
                new_symbols.push(bitmap);
            }

            if new_symbols.len() >= params.number_of_new_symbols {
                break;
            }
        }

        // Decode a collective bitmap for Huffman direct-mode symbols.
        if huffman
            && !refinement
            && !symbol_widths.is_empty()
            && total_width > 0
            && current_height > 0
        {
            let tables = huffman_tables.unwrap();
            let bitmap_size = tables
                .table_bitmap_size
                .decode(huffman_input.as_mut().unwrap())?;
            if trace_symbol && trace_height_classes < 5 {
                eprintln!(
                    "symbol_dict: height_class={} widths={} total_width={} bitmap_size={}",
                    current_height,
                    symbol_widths.len(),
                    total_width,
                    bitmap_size
                );
            }
            if trace_symbol {
                trace_height_classes = trace_height_classes.saturating_add(1);
                if bitmap_size > 0 {
                    let huff_pos = huffman_input.as_ref().map(|r| r.get_position()).unwrap_or(0);
                    let huff_shift = huffman_input.as_ref().map(|r| r.get_shift()).unwrap_or(0);
                    eprintln!(
                        "symbol_dict: mmr_bitmap height_class={} width={} height={} bitmap_size={} huff_pos={} huff_shift={}",
                        current_height,
                        total_width,
                        current_height_usize,
                        bitmap_size,
                        huff_pos,
                        huff_shift
                    );
                }
            }

            huffman_input.as_mut().unwrap().byte_align();

            let total_width_usize = total_width as usize;
            let collective_bitmap = if bitmap_size == 0 {
                read_uncompressed_bitmap(
                    huffman_input.as_mut().unwrap(),
                    total_width_usize,
                    current_height_usize,
                )?
            } else {
                let mut mmr_reader = huffman_input.as_mut().unwrap().clone();
                mmr_reader.set_limit(bitmap_size as usize);
                let bmp = decode_mmr_bitmap(
                    &mut mmr_reader,
                    total_width_usize,
                    current_height_usize,
                    true,
                )?;
                huffman_input.as_mut().unwrap().skip(bitmap_size as usize);
                bmp
            };

            let base_index = new_symbols.len();
            let symbols = split_collective_bitmap(
                &collective_bitmap,
                &symbol_widths,
                current_height_usize,
            );
            if trace_symbol {
                for (idx, symbol) in symbols.iter().enumerate() {
                    if symbol.width >= 500 {
                        let black = symbol.count_black_pixels();
                        let (row_min, row_max, full_rows) = symbol.row_black_stats();
                        let mut full_row_idx = Vec::new();
                        let mut max_row_idx = Vec::new();
                        let full_bytes = symbol.width / 8;
                        let rem_bits = symbol.width % 8;
                        let mask = if rem_bits == 0 {
                            0xFF
                        } else {
                            0xFFu8 << (8 - rem_bits)
                        };
                        for y in 0..symbol.height {
                            let row_start = y * symbol.stride;
                            let row = &symbol.data[row_start..row_start + symbol.stride];
                            let mut row_count = 0u32;
                            for &b in &row[..full_bytes] {
                                row_count += b.count_ones();
                            }
                            if rem_bits != 0 {
                                row_count += (row[full_bytes] & mask).count_ones();
                            }
                            if row_count == row_max {
                                max_row_idx.push(y);
                            }
                            if row_count as usize == symbol.width {
                                full_row_idx.push(y);
                            }
                        }
                        let max_row_sample = max_row_idx
                            .iter()
                            .take(6)
                            .map(|v| v.to_string())
                            .collect::<Vec<_>>()
                            .join(",");
                        let full_row_sample = full_row_idx
                            .iter()
                            .take(6)
                            .map(|v| v.to_string())
                            .collect::<Vec<_>>()
                            .join(",");
                        let total = (symbol.width.saturating_mul(symbol.height)) as u32;
                        let fill_ppm = if total > 0 {
                            black.saturating_mul(1000) / total
                        } else {
                            0
                        };
                        let combined_index = params.symbols.len() + base_index + idx;
                        eprintln!(
                            "symbol_dict: sym={} size={}x{} black={} fill_ppm={} row_min={} row_max={} full_rows={} row_max_idx=[{}] row_full_idx=[{}]",
                            combined_index,
                            symbol.width,
                            symbol.height,
                            black,
                            fill_ppm,
                            row_min,
                            row_max,
                            full_rows,
                            max_row_sample,
                            full_row_sample
                        );
                    }
                }
            }
            new_symbols.extend(symbols);
        }
    }

    if new_symbols.len() != params.number_of_new_symbols {
        return Err(Jbig2Error::new(&format!(
            "symbol dictionary decoded {} of {} symbols",
            new_symbols.len(),
            params.number_of_new_symbols
        )));
    }
    if trace_symbol {
        if min_delta_height != i32::MAX {
            eprintln!(
                "symbol_dict: delta_height range=[{}, {}] neg_delta_height={}",
                min_delta_height, max_delta_height, neg_delta_height
            );
        }
        if min_delta_width != i32::MAX {
            eprintln!(
                "symbol_dict: delta_width range=[{}, {}] neg_delta_width={}",
                min_delta_width, max_delta_width, neg_delta_width
            );
        }
        if min_symbol_width != usize::MAX && min_symbol_height != usize::MAX {
            eprintln!(
                "symbol_dict: symbol_size min={}x{} max={}x{}",
                min_symbol_width, min_symbol_height, max_symbol_width, max_symbol_height
            );
        }
    }

    // Build the export list based on run-length flags.
    let total_symbols = params.symbols.len() + new_symbols.len();
    let mut flags = Vec::with_capacity(total_symbols);

    if huffman {
        let export_table = huff_export_run.as_ref().unwrap();
        let mut export = false; // first run is non-exported
        let mut i = 0usize;
        let mut exported_count = 0usize;
        let mut empty_runs = 0u32;
        while i < total_symbols {
            let (run, oob) = export_table.decode_entry(huffman_input.as_mut().unwrap())?;
            if oob {
                return Err(Jbig2Error::new(
                    "OOB when decoding runlength for exported symbols",
                ));
            }

            let mut run_len = run;
            if run_len <= 0 {
                empty_runs += 1;
                if empty_runs == 1000 {
                    return Err(Jbig2Error::new(
                        "run length too small in export symbol table",
                    ));
                }
            } else {
                empty_runs = 0;
            }
            if run_len < 0 {
                run_len = 0;
            }
            let mut run_len = run_len as usize;

            if run_len > total_symbols - i {
                run_len = total_symbols - i;
            }
            if export && exported_count + run_len > params.number_of_exported_symbols {
                run_len = params.number_of_exported_symbols - exported_count;
            }

            for _ in 0..run_len {
                flags.push(export);
                if export {
                    exported_count += 1;
                }
                i += 1;
            }
            export = !export;
        }
    } else {
        // Arithmetic mode uses run-lengths to toggle export flags.
        let mut export = false;
        let mut i = 0usize;
        let mut exported_count = 0usize;
        let mut empty_runs = 0u32;
        while i < total_symbols {
            let run = match decode_integer_context(decoding_context, "IAEX")? {
                Some(v) => v,
                None => {
                    return Err(Jbig2Error::new(
                        "OOB when decoding runlength for exported symbols",
                    ))
                }
            };
            let mut run_len = run;
            if run_len <= 0 {
                empty_runs += 1;
                if empty_runs == 1000 {
                    return Err(Jbig2Error::new(
                        "run length too small in export symbol table",
                    ));
                }
            } else {
                empty_runs = 0;
            }
            if run_len < 0 {
                run_len = 0;
            }
            let mut run_len = run_len as usize;
            if run_len > total_symbols - i {
                run_len = total_symbols - i;
            }
            if export
                && exported_count + run_len > params.number_of_exported_symbols
            {
                run_len = params.number_of_exported_symbols - exported_count;
            }

            for _ in 0..run_len {
                flags.push(export);
                if export {
                    exported_count += 1;
                }
                i += 1;
            }
            export = !export;
        }
    }

    let mut exported_symbols = Vec::with_capacity(params.number_of_exported_symbols);
    for (i, &export) in flags.iter().enumerate() {
        if export {
            let sym = if i < params.symbols.len() {
                &params.symbols[i]
            } else {
                &new_symbols[i - params.symbols.len()]
            };
            exported_symbols.push(sym.clone());
        }
    }

    if exported_symbols.len() != params.number_of_exported_symbols {
        if trace_symbol {
            eprintln!(
                "symbol_dict: export count mismatch (got {}, expected {})",
                exported_symbols.len(),
                params.number_of_exported_symbols
            );
        }
        if params.number_of_exported_symbols <= total_symbols {
            exported_symbols.clear();
            for i in 0..params.number_of_exported_symbols {
                let sym = if i < params.symbols.len() {
                    &params.symbols[i]
                } else {
                    &new_symbols[i - params.symbols.len()]
                };
                exported_symbols.push(sym.clone());
            }
        } else {
            return Err(Jbig2Error::new(
                "exported symbol count exceeds available symbols",
            ));
        }
    }

    Ok(exported_symbols)
}
