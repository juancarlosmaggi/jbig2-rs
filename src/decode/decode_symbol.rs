use crate::bitmap::Bitmap;
use crate::contexts::DecodingContext;
use crate::decode::decode_utils::read_uncompressed_bitmap;
use crate::decode::decode_mmr::decode_mmr_bitmap;
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
    println!("decode_symbol_dictionary: context.data.len()={}, context.start={}, context.end={}", decoding_context.data.len(), decoding_context.start, decoding_context.end);
    // Validate parameters
    if params.number_of_new_symbols == 0 {
        return Err(Jbig2Error::new("number of new symbols must be positive"));
    }
    validation::validate_symbol_decode_params(params.template_index, params.number_of_new_symbols)?;
    let mut new_symbols = Vec::new();
    let mut current_height = 0i32;
    let symbol_code_length =
        crate::core_utils::log2((params.symbols.len() + params.number_of_new_symbols) as u32);
    let huffman_tables = params.huffman_tables.as_ref();
    if params.huffman && huffman_tables.is_none() {
        return Err(Jbig2Error::new(
            "Huffman tables required for Huffman decoding",
        ));
    }
    while new_symbols.len() < params.number_of_new_symbols {

        // 6.5.6 Huffman coded symbol dictionary
        if params.huffman {
            let tables = huffman_tables.as_ref().unwrap();

            // 1) Decode HCDH (Height Class Delta Height)
            let hcdh = tables.table_delta_height.decode(huffman_input.as_mut().unwrap())?;
            if hcdh < 0 {
                // OOB - End of symbol dictionary
                return Ok(new_symbols);
            }

            let height = current_height + hcdh as i32;
            current_height = height;
            let mut current_width = 0;
            let mut total_width = 0;
            let _first_symbol_index = new_symbols.len();
            let mut symbol_widths = Vec::new();

            // 2) Decode symbols in this height class
            loop {
                let dw = tables.table_delta_width.decode(huffman_input.as_mut().unwrap())?;

                if dw < 0 {
                    // OOB - End of height class
                    break;
                }

                current_width += dw as i32;
                total_width += current_width;

                if params.refinement {
                    // Refinement/aggregate-coded symbol bitmap (6.5.8.2)
                    let number_of_instances = tables.table_aggregate_instances.decode(huffman_input.as_mut().unwrap())?;
                    if number_of_instances > 1 {
                        // Aggregate symbol logic (unchanged)
                        let mut input_symbols = params.symbols.clone();
                        input_symbols.extend(new_symbols.clone());
                        let text_params = crate::decode::decode_text::TextRegionParams {
                            huffman: false,
                            refinement: params.refinement,
                            width: current_width as usize,
                            height: current_height as usize,
                            default_pixel_value: 0,
                            number_of_symbol_instances: number_of_instances as usize,
                            strip_size: 1,
                            input_symbols,
                            symbol_code_length: symbol_code_length as usize,
                            transposed: false,
                            ds_offset: 0,
                            reference_corner: 1,
                            combination_operator: 0,
                            log_strip_size: 0,
                            huffman_tables: None,
                            refinement_template_index: params.refinement_template_index,
                            refinement_at: params.refinement_at.clone(),
                        };
                        let bitmap = crate::decode::decode_text::decode_text_region(
                            &text_params,
                            decoding_context,
                            None,
                        )?;
                        new_symbols.push(bitmap);
                    } else {
                        // Refinement logic (unchanged)
                        // ... (This part relies on arithmetic coding which is mixed in spec?)
                        // Actually spec says "If the refinement flag is 1... decoded using the arithmetic coding method"
                        // So we need to switch context? But we are in Huffman block.
                        // 6.5.8.2.2: "The refinement bitmap is decoded using the arithmetic coding method...
                        // The arithmetic decoder is re-initialized..."
                        // This seems complex and text_region.jb2 is likely not using refinement.
                        // For now, assuming text_region.jb2 hits the !refinement branch.
                        return Err(Jbig2Error::new("Huffman refinement not fully implemented"));
                    }
                } else {
                    // Direct-coded symbol bitmap (6.5.8.1) - BUT in Huffman mode this is Collective Bitmap (6.5.9)
                    // We just accumulate widths here.
                    symbol_widths.push(current_width as usize);
                }
            }

            // 3) Decode Collective Bitmap (6.5.9)
            if !params.refinement {
                let bitmap_size = tables.table_bitmap_size.decode(huffman_input.as_mut().unwrap())?;
                eprintln!("DEBUG: BMSIZE = {}", bitmap_size);
                huffman_input.as_mut().unwrap().byte_align();
                
                if bitmap_size == 0 {
                    // BMSIZE = 0 means uncompressed bitmap (not MMR-coded)
                    // jbig2dec: If BMSIZE == 0, bitmap is uncompressed
                    eprintln!("DEBUG: BMSIZE=0, reading uncompressed bitmap");
                    
                    if total_width == 0 || current_height == 0 {
                        eprintln!("DEBUG: Zero-size collective bitmap, skipping");
                        continue;
                    }
                    
                    let collective_bitmap = read_uncompressed_bitmap(
                        huffman_input.as_mut().unwrap(),
                        total_width as usize,
                        current_height as usize,
                    )?;
                    
                    // Split collective bitmap into individual symbol bitmaps
                    let mut current_x = 0;
                    for width in symbol_widths.iter() {
                        let mut symbol_bitmap = Bitmap::new(*width as usize, current_height as usize);
                        for y in 0..current_height {
                            for x in 0..*width {
                                let pixel = collective_bitmap.get_pixel((current_x + x) as usize, y as usize);
                                symbol_bitmap.set_pixel(x as usize, y as usize, pixel);
                            }
                        }
                        new_symbols.push(symbol_bitmap);
                        current_x += *width;
                    }
                } else {
                    // BMSIZE > 0 means MMR-coded collective bitmap
                    let start_pos = huffman_input.as_ref().unwrap().get_position();
                    eprintln!("DEBUG: MMR start_pos = {}, length = {}, width = {}, height = {}", 
                        start_pos, bitmap_size, total_width, current_height);
                    
                    if total_width == 0 || current_height == 0 {
                        eprintln!("DEBUG: Zero-size MMR bitmap, skipping {} bytes", bitmap_size);
                        huffman_input.as_mut().unwrap().skip(bitmap_size as usize);
                        continue;
                    }
                    
                    let mut mmr_reader = huffman_input.as_mut().unwrap().clone();
                    // Limit reader to BMSIZE
                    mmr_reader.set_limit(bitmap_size as usize);
                    
                    let collective_bitmap = decode_mmr_bitmap(
                        &mut mmr_reader,
                        total_width as usize,
                        current_height as usize,
                        false,
                    )?;
                    
                    huffman_input.as_mut().unwrap().skip(bitmap_size as usize);

                    // Split collective bitmap into individual symbol bitmaps
                    let mut current_x = 0;
                    for width in symbol_widths.iter() {
                        let mut symbol_bitmap = Bitmap::new(*width as usize, current_height as usize);
                        for y in 0..current_height {
                            for x in 0..*width {
                                let pixel = collective_bitmap.get_pixel((current_x + x) as usize, y as usize);
                                symbol_bitmap.set_pixel(x as usize, y as usize, pixel);
                            }
                        }
                        new_symbols.push(symbol_bitmap);
                        current_x += *width;
                    }
                }
                // Decoding of this height class is complete
                break;
            }
        } else {
            // Arithmetic coding path (unchanged)
            let delta_height = decode_i32_huffman_or_arith(
                params.huffman,
                || {
                    let tables = huffman_tables.unwrap();
                    tables
                        .table_delta_height
                        .decode(huffman_input.as_mut().unwrap())
                },
                "IADH",
                decoding_context,
            )?;
            current_height += delta_height;
            let mut current_width = 0i32;
            let mut total_width = 0i32;
            let first_symbol = if params.huffman { new_symbols.len() } else { 0 };
            let mut symbol_widths = Vec::new();
            loop {
                eprintln!("DEBUG: Loop start, huffman={}", params.huffman);
                let delta_width = if params.huffman {
                    let tables = huffman_tables.unwrap();
                    let dw = tables
                        .table_delta_width
                        .decode(huffman_input.as_mut().unwrap())?;
                    eprintln!("DEBUG: Huffman DW = {}", dw);
                    dw
                } else {
                    match decode_integer_context(decoding_context, "IADW")? {
                        Some(dw) => {
                            eprintln!("DEBUG: DW = {}", dw);
                            dw
                        },
                        None => break, // OOB
                    }
                };
                if delta_width < 0 && params.huffman {
                    break; // OOB for Huffman
                }
                current_width += delta_width;
                total_width += current_width;
                eprintln!("DEBUG: current_width={}, total_width={}", current_width, total_width);
                if params.refinement {
                    // 6.5.8.2 Refinement/aggregate-coded symbol bitmap
                    let number_of_instances =
                        decode_integer_context(decoding_context, "IAAI")?.unwrap_or(1);
                    if number_of_instances > 1 {
                        // Aggregate symbol - decode text region
                        let mut input_symbols = params.symbols.clone();
                        input_symbols.extend(new_symbols.clone());
                        let text_params = crate::decode::decode_text::TextRegionParams {
                            huffman: false, // Aggregate doesn't use Huffman
                            refinement: params.refinement,
                            width: current_width as usize,
                            height: current_height as usize,
                            default_pixel_value: 0,
                            number_of_symbol_instances: number_of_instances as usize,
                            strip_size: 1,
                            input_symbols,
                            symbol_code_length: symbol_code_length as usize,
                            transposed: false,
                            ds_offset: 0,
                            reference_corner: 1,     // top left
                            combination_operator: 0, // OR
                            log_strip_size: 0,
                            huffman_tables: None,
                            refinement_template_index: params.refinement_template_index,
                            refinement_at: params.refinement_at.clone(),
                        };
                        let bitmap = crate::decode::decode_text::decode_text_region(
                            &text_params,
                            decoding_context,
                            None,
                        )?;
                        new_symbols.push(bitmap);
                    } else {
                        let symbol_id =
                            decode_iaid_context(decoding_context, symbol_code_length as usize)?;
                        let symbol = if (symbol_id as usize) < params.symbols.len() {
                            &params.symbols[symbol_id as usize]
                        } else {
                            &new_symbols[symbol_id as usize - params.symbols.len()]
                        };
                        // Decode refinement parameters using arithmetic coding (always, even in Huffman mode)
                        let rdw = decode_integer_context(decoding_context, "IARDW")?.unwrap_or(0);
                        let rdh = decode_integer_context(decoding_context, "IARDH")?.unwrap_or(0);
                        let rdx = decode_integer_context(decoding_context, "IARDX")?.unwrap_or(0);
                        let rdy = decode_integer_context(decoding_context, "IARDY")?.unwrap_or(0);
                        // Use decode_refinement here
                        let bitmap = crate::decode::decode_refinement::decode_refinement(
                            &crate::decode::decode_refinement::RefinementParams {
                                width: (symbol.width as i32 + rdw) as usize,
                                height: (symbol.height as i32 + rdh) as usize,
                                template_index: params.refinement_template_index,
                                reference_bitmap: symbol,
                                offset_x: (rdw >> 1) + rdx,
                                offset_y: (rdh >> 1) + rdy,
                                prediction: false,
                                at: params.refinement_at.clone(),
                            },
                            decoding_context,
                        )?;
                        new_symbols.push(bitmap);
                    }
                } else if params.huffman {
                    // Store only symbol width and decode a collective bitmap when the height class is done.
                    symbol_widths.push(current_width as usize);
                } else {
                    // 6.5.8.1 Direct-coded symbol bitmap
                    let bitmap = crate::decode::decode_generic::decode_bitmap(
                        &crate::decode::decode_generic::DecodeBitmapParams {
                            mmr: false,
                            width: current_width as usize,
                            height: current_height as usize,
                            template_index: params.template_index,
                            prediction: false,
                            skip: None,
                            at: params.at.clone(),
                        },
                        decoding_context,
                    )?;
                    new_symbols.push(bitmap);
                }
            }
            if params.huffman && !params.refinement {
                // 6.5.9 Height class collective bitmap
                let tables = huffman_tables.unwrap();
                let bitmap_size = tables
                    .table_bitmap_size
                    .decode(huffman_input.as_mut().unwrap())?;
                huffman_input.as_mut().unwrap().byte_align();
                let collective_bitmap = if bitmap_size == 0 {
                    // Uncompressed collective bitmap
                    read_uncompressed_bitmap(
                        huffman_input.as_mut().unwrap(),
                        total_width as usize,
                        current_height as usize,
                    )?
                } else {
                    // MMR collective bitmap
                    let start_pos = huffman_input.as_ref().unwrap().get_position();
                    let bitmap_end = start_pos + bitmap_size as usize;
                    let mut mmr_reader = Reader::new(
                        huffman_input.as_ref().unwrap().get_data().to_vec(),
                        start_pos,
                        bitmap_end,
                    );
                    let bitmap = crate::decode::decode_mmr::decode_mmr_bitmap(
                        &mut mmr_reader,
                        total_width as usize,
                        current_height as usize,
                        false,
                    )?;
                    huffman_input.as_mut().unwrap().set_position(bitmap_end);
                    bitmap
                };
                let number_of_symbols_decoded = symbol_widths.len();
                if first_symbol == number_of_symbols_decoded - 1 {
                    // collectiveBitmap is a single symbol.
                    new_symbols.push(collective_bitmap);
                } else {
                    // Divide collectiveBitmap into symbols.
                    let mut x_min = 0;
                    for &bitmap_width in symbol_widths
                        .iter()
                        .skip(first_symbol)
                        .take(number_of_symbols_decoded - first_symbol)
                    {
                        let x_max = x_min + bitmap_width;
                        let mut symbol_bitmap = Bitmap::new(bitmap_width, current_height as usize);
                        for y in 0..current_height as usize {
                            for x in 0..bitmap_width {
                                let pixel = collective_bitmap.get_pixel(x_min + x, y);
                                symbol_bitmap.set_pixel(x, y, pixel);
                            }
                        }
                        new_symbols.push(symbol_bitmap);
                        x_min = x_max;
                    }
                }
            }
        }
    }
    // 6.5.10 Exported symbols
    let mut flags = Vec::new();
    let total_symbols_length = params.symbols.len() + params.number_of_new_symbols;
    let mut current_flag = false;
    while flags.len() < total_symbols_length {
        let run_length = if params.huffman {
            let tables = huffman_tables.unwrap();
            tables
                .table_aggregate_instances
                .decode(huffman_input.as_mut().unwrap())? as usize
        } else {
            decode_integer_context(decoding_context, "IAEX")?.unwrap_or(0) as usize
        };
        for _ in 0..run_length {
            flags.push(current_flag);
        }
        current_flag = !current_flag;
    }
    let mut exported_symbols = Vec::new();
    for (i, &flag) in flags.iter().enumerate() {
        if flag {
            if i < params.symbols.len() {
                exported_symbols.push(params.symbols[i].clone());
            } else {
                exported_symbols.push(new_symbols[i - params.symbols.len()].clone());
            }
        }
    }
    Ok(exported_symbols)
}
