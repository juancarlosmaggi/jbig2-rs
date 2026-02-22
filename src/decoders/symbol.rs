use crate::arithmetic::contexts::DecodingContext;
use crate::arithmetic::helpers::{decode_iaid_context, decode_integer_context};
use crate::bitmap::Bitmap;
use crate::common::error::Jbig2Error;
use crate::common::reader::Reader;
use crate::common::validation;
use crate::decoders::mmr::decode_mmr_bitmap;
use crate::decoders::symbol_helpers::{
    AggregateSymbolParams, decode_aggregate_symbol, split_collective_bitmap,
};
use crate::decoders::utils::read_uncompressed_bitmap;
use crate::huffman::{SymbolDictionaryHuffmanTables, get_standard_table};

/// Inputs required to decode a symbol dictionary segment.
#[derive(Clone)]
pub struct SymbolDictionaryParams<'a> {
    pub huffman: bool,
    pub refinement: bool,
    pub symbols: Vec<&'a Bitmap>,
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
    params: &SymbolDictionaryParams<'_>,
    decoding_context: &mut DecodingContext<'_>,
    mut huffman_input: Option<&mut Reader<'_>>,
) -> Result<Vec<Bitmap>, Jbig2Error> {
    if params.number_of_new_symbols == 0 {
        return Err(Jbig2Error::new("number of new symbols must be positive"));
    }

    // Validate that Huffman tables are provided when Huffman mode is enabled.
    if params.huffman && params.huffman_tables.is_none() {
        return Err(Jbig2Error::new(
            "Huffman tables required when Huffman mode is enabled",
        ));
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
                return Err(Jbig2Error::new("OOB when decoding height class delta"));
            }
            val
        } else {
            match decode_integer_context(decoding_context, "IADH")? {
                Some(v) => v,
                None => return Err(Jbig2Error::new("OOB when decoding height class delta")),
            }
        };

        current_height = current_height
            .checked_add(delta_height)
            .ok_or_else(|| Jbig2Error::new("height class overflow"))?;
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

            if new_symbols.len() >= params.number_of_new_symbols {
                break;
            }

            current_width = current_width
                .checked_add(dw)
                .ok_or_else(|| Jbig2Error::new("symbol width overflow"))?;
            if current_width < 0 {
                return Err(Jbig2Error::new("DW value would make symbol width negative"));
            }
            total_width = total_width
                .checked_add(current_width)
                .ok_or_else(|| Jbig2Error::new("total width overflow"))?;

            let current_width_usize = current_width as usize;
            if refinement {
                // Decode the instance count for refinement aggregation.
                let instances = if huffman {
                    let tables = huffman_tables.unwrap();
                    let (val, oob) = tables
                        .table_aggregate_instances
                        .decode_entry(huffman_input.as_mut().unwrap())?;
                    if oob {
                        return Err(Jbig2Error::new(
                            "OOB when decoding aggregate instance count",
                        ));
                    }
                    val
                } else {
                    match decode_integer_context(decoding_context, "IAAI")? {
                        Some(v) => v,
                        None => {
                            return Err(Jbig2Error::new(
                                "OOB when decoding aggregate instance count",
                            ));
                        }
                    }
                };
                if instances <= 0 {
                    return Err(Jbig2Error::new(
                        "invalid number of symbols in aggregate glyph",
                    ));
                }
                let instances = instances as usize;

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
                        let rdx = decode_integer_context(decoding_context, "IARDX")?.ok_or_else(
                            || Jbig2Error::new("OOB when decoding refinement x offset"),
                        )?;
                        let rdy = decode_integer_context(decoding_context, "IARDY")?.ok_or_else(
                            || Jbig2Error::new("OOB when decoding refinement y offset"),
                        )?;
                        (symbol_id, rdx, rdy, None)
                    };

                    let total_symbols = params.symbols.len() + new_symbols.len();
                    if symbol_id >= total_symbols {
                        return Err(Jbig2Error::new("invalid refinement symbol id"));
                    }
                    let sym = if symbol_id < params.symbols.len() {
                        params.symbols[symbol_id]
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
                        let mut temp_context = DecodingContext::new(data, current_pos, data.len());
                        crate::decoders::refinement::decode_refinement(
                            &crate::decoders::refinement::RefinementParams {
                                width,
                                height,
                                template_index: params.refinement_template_index,
                                reference_bitmap: sym,
                                offset_x: rdx,
                                offset_y: rdy,
                                prediction: false,
                                at: params.refinement_at.as_slice(),
                            },
                            &mut temp_context,
                        )?
                    } else {
                        crate::decoders::refinement::decode_refinement(
                            &crate::decoders::refinement::RefinementParams {
                                width,
                                height,
                                template_index: params.refinement_template_index,
                                reference_bitmap: sym,
                                offset_x: rdx,
                                offset_y: rdy,
                                prediction: false,
                                at: params.refinement_at.as_slice(),
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
                        total_symbols,
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
                        huffman_input.as_deref_mut(),
                    )?;
                    new_symbols.push(bitmap);
                }
            } else if huffman {
                symbol_widths.push(current_width_usize);
            } else {
                // Decode a directly coded symbol bitmap.
                let bitmap = crate::decoders::generic::decode_bitmap(
                    &crate::decoders::generic::DecodeBitmapParams {
                        mmr: false,
                        width: current_width_usize,
                        height: current_height_usize,
                        template_index: params.template_index,
                        prediction: false,
                        skip: None,
                        at: params.at.as_slice(),
                    },
                    decoding_context,
                )?;
                new_symbols.push(bitmap);
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

            let symbols =
                split_collective_bitmap(&collective_bitmap, &symbol_widths, current_height_usize);
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

            let run_len_u32 = run as u32;
            if run_len_u32 == 0 {
                empty_runs += 1;
                if empty_runs == 1000 {
                    return Err(Jbig2Error::new(
                        "run length too small in export symbol table",
                    ));
                }
            } else {
                empty_runs = 0;
            }
            let mut run_len = run_len_u32 as usize;

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
                    ));
                }
            };
            let run_len_u32 = run as u32;
            if run_len_u32 == 0 {
                empty_runs += 1;
                if empty_runs == 1000 {
                    return Err(Jbig2Error::new(
                        "run length too small in export symbol table",
                    ));
                }
            } else {
                empty_runs = 0;
            }
            let mut run_len = run_len_u32 as usize;
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
    }

    let mut exported_symbols = Vec::with_capacity(params.number_of_exported_symbols);
    let input_symbols_len = params.symbols.len();
    for (symbol, &export) in params.symbols.iter().zip(flags.iter()) {
        if export {
            exported_symbols.push((*symbol).clone());
        }
    }

    let offset = input_symbols_len;
    for (j, symbol) in new_symbols.into_iter().enumerate() {
        let flag_idx = offset + j;
        if flag_idx < flags.len() && flags[flag_idx] {
            exported_symbols.push(symbol);
        }
    }

    if params.number_of_exported_symbols > total_symbols {
        return Err(Jbig2Error::new(
            "exported symbol count exceeds available symbols",
        ));
    }
    if exported_symbols.len() != params.number_of_exported_symbols {
        if exported_symbols.len() < params.number_of_exported_symbols {
            let missing = params.number_of_exported_symbols - exported_symbols.len();
            exported_symbols.extend((0..missing).map(|_| Bitmap::new(0, 0)));
        } else {
            exported_symbols.truncate(params.number_of_exported_symbols);
        }
    }

    Ok(exported_symbols)
}
