use crate::bitmap::Bitmap;
use crate::contexts::DecodingContext;
use crate::decode::decode_mmr::decode_mmr_bitmap;
use crate::decode::decode_symbol_helpers::{
    AggregateSymbolParams, decode_aggregate_symbol, split_collective_bitmap,
};
use crate::decode::decode_utils::read_uncompressed_bitmap;
use crate::decoder::{decode_i32_huffman_or_arith, decode_iaid_context, decode_integer_context};
use crate::error::Jbig2Error;
use crate::huffman::SymbolDictionaryHuffmanTables;
use crate::reader::Reader;
use crate::validation;

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

pub fn decode_symbol_dictionary(
    params: &SymbolDictionaryParams,
    decoding_context: &mut DecodingContext,
    mut huffman_input: Option<&mut Reader>,
) -> Result<Vec<Bitmap>, Jbig2Error> {
    if params.number_of_new_symbols == 0 {
        // return Err(Jbig2Error::new("number of new symbols must be positive"));
    }

    // Validate that Huffman tables are provided when Huffman mode is enabled
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
    let mut current_height: usize = 0;

    let total_symbols = params.symbols.len() + params.number_of_new_symbols;
    let symbol_code_length = if total_symbols <= 1 {
        0
    } else {
        ((total_symbols as u64 - 1).ilog2() as usize) + 1
    };

    let huffman = params.huffman;
    let refinement = params.refinement;
    let huffman_tables = params.huffman_tables.as_ref();

    while new_symbols.len() < params.number_of_new_symbols {
        // Height class delta height (IADH / SDHUFFDH)
        let delta_height: usize = if huffman {
            let tables = huffman_tables.unwrap();
            let (val, oob) = tables
                .table_delta_height
                .decode_entry(huffman_input.as_mut().unwrap())?;
            if oob {
                break;
            }
            val as usize
        } else {
            decode_i32_huffman_or_arith(huffman, || Ok(0), "IADH", decoding_context)? as usize
        };

        current_height += delta_height;
        // current_height already unsigned
        // removed arbitrary limit to match reference decoders
        if current_height == 0 {
            continue;
        }

        let mut current_width: usize = 0;
        let mut total_width: usize = 0;
        let mut symbol_widths: Vec<usize> = Vec::new();

        loop {
            // Delta width (IADW / SDHUFFDW)
            let dw: usize = if huffman {
                let tables = huffman_tables.unwrap();
                let (val, oob) = tables
                    .table_delta_width
                    .decode_entry(huffman_input.as_mut().unwrap())?;
                if oob {
                    break;
                }
                val as usize
            } else {
                match decode_integer_context(decoding_context, "IADW")? {
                    Some(v) => v as usize,
                    None => break, // OOB – end of height class
                }
            };

            current_width += dw;
            // unsigned
            total_width += current_width;
            // removed arbitrary limit

            if refinement {
                // Number of instances (IAAI / SDHUFFAGGINST)
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
                    // Single symbol refinement – always arithmetic coded, bottom-left reference corner (spec 6.5.8.2.1)
                    let symbol_id =
                        decode_iaid_context(decoding_context, symbol_code_length)? as usize;
                    let sym = if symbol_id < params.symbols.len() {
                        &params.symbols[symbol_id]
                    } else {
                        &new_symbols[symbol_id - params.symbols.len()]
                    };

                    let rdw = decode_integer_context(decoding_context, "IARDW")?.unwrap_or(0);
                    let rdh = decode_integer_context(decoding_context, "IARDH")?.unwrap_or(0);
                    let rdx = decode_integer_context(decoding_context, "IARDX")?.unwrap_or(0);
                    let rdy = decode_integer_context(decoding_context, "IARDY")?.unwrap_or(0);

                    let new_width = sym.width + rdw as usize;
                    let new_height = sym.height + rdh as usize;
                    if new_width == 0 || new_height == 0 {
                        // return Err(Jbig2Error::new("Invalid dimensions for refined symbol"));
                    }

                    let offset_x = rdx + (rdw >> 1);
                    // Bottom-left reference corner → adjust Y offset by -(REFH - 1)
                    let offset_y = rdy + (rdh >> 1) - (sym.height as i32 - 1);

                    let bitmap = crate::decode::decode_refinement::decode_refinement(
                        &crate::decode::decode_refinement::RefinementParams {
                            width: new_width as usize,
                            height: new_height as usize,
                            template_index: params.refinement_template_index,
                            reference_bitmap: sym,
                            offset_x,
                            offset_y,
                            prediction: false,
                            at: params.refinement_at.clone(),
                        },
                        decoding_context,
                    )?;

                    new_symbols.push(bitmap);
                } else {
                    // Aggregate symbol
                    let agg_params = AggregateSymbolParams {
                        current_width: current_width as i32,
                        current_height: current_height as i32,
                        number_of_instances: instances as i32,
                        symbol_code_length,
                        refinement: true,
                        refinement_template_index: params.refinement_template_index,
                        refinement_at: params.refinement_at.clone(),
                    };
                    let bitmap = decode_aggregate_symbol(
                        &agg_params,
                        &params.symbols,
                        &new_symbols,
                        decoding_context,
                    )?;
                    new_symbols.push(bitmap);
                }
            } else if huffman {
                symbol_widths.push(current_width);
            } else {
                // Direct arithmetic-coded symbol bitmap
                let bitmap = crate::decode::decode_generic::decode_bitmap(
                    &crate::decode::decode_generic::DecodeBitmapParams {
                        mmr: false,
                        width: current_width,
                        height: current_height,
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

        // Collective bitmap for Huffman + direct mode
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

            let collective_bitmap = if bitmap_size == 0 {
                read_uncompressed_bitmap(
                    huffman_input.as_mut().unwrap(),
                    total_width,
                    current_height,
                )?
            } else {
                let mut mmr_reader = huffman_input.as_mut().unwrap().clone();
                mmr_reader.set_limit(bitmap_size as usize);
                let bmp = decode_mmr_bitmap(
                    &mut mmr_reader,
                    total_width,
                    current_height,
                    false,
                )?;
                huffman_input.as_mut().unwrap().skip(bitmap_size as usize);
                bmp
            };

            let symbols = split_collective_bitmap(
                &collective_bitmap,
                &symbol_widths,
                current_height,
            );
            new_symbols.extend(symbols);
        }
    }

    // Exported symbols
    let total_symbols = params.symbols.len() + new_symbols.len();
    let mut flags = Vec::with_capacity(total_symbols);

    if huffman {
        let tables = huffman_tables.unwrap();
        let mut export = false; // first run is non-exported
        loop {
            let (run, oob) = tables
                .table_aggregate_instances // NOTE: replace with B.10 SDHUFFEXRUN when implemented
                .decode_entry(huffman_input.as_mut().unwrap())?;
            if oob || run == 0 {
                break;
            }
            let run = run as usize;
            for _ in 0..run {
                if flags.len() < total_symbols {
                    flags.push(export);
                }
            }
            export = !export;
            if flags.len() >= total_symbols {
                break;
            }
        }
        // OOB or early termination → remaining symbols exported
        while flags.len() < total_symbols {
            flags.push(true);
        }
    } else {
        // Arithmetic mode – IAEX single-bit context
        for _ in 0..total_symbols {
            let mut ctx = decoding_context.get_contexts("IAEX");
            if ctx.is_empty() {
                ctx.push(0i8);
            }
            let bit = decoding_context.get_decoder().read_bit(&mut ctx, 0)?;
            flags.push(bit != 0);
        }
        // Stream ended early → remaining symbols exported (spec 6.5.10)
        while flags.len() < total_symbols {
            flags.push(true);
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

    Ok(exported_symbols)
}
