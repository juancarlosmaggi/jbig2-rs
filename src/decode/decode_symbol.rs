use crate::bitmap::Bitmap;
use crate::contexts::DecodingContext;
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
    // Symbol refinement with Huffman is now supported

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

    while new_symbols.len() < params.number_of_new_symbols {
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
            let delta_width = if params.huffman {
                let tables = huffman_tables.unwrap();
                tables
                    .table_delta_width
                    .decode(huffman_input.as_mut().unwrap())?
            } else {
                match decode_integer_context(decoding_context, "IADW")? {
                    Some(dw) => dw,
                    None => break, // OOB
                }
            };
            if delta_width < 0 && params.huffman {
                break; // OOB for Huffman
            }
            current_width += delta_width;
            total_width += current_width;

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
                    let rdx = decode_integer_context(decoding_context, "IARDX")?.unwrap_or(0);
                    let rdy = decode_integer_context(decoding_context, "IARDY")?.unwrap_or(0);
                    let symbol = if (symbol_id as usize) < params.symbols.len() {
                        &params.symbols[symbol_id as usize]
                    } else {
                        &new_symbols[symbol_id as usize - params.symbols.len()]
                    };
                    // Use decode_refinement here
                    let bitmap = crate::decode::decode_refinement::decode_refinement(
                        &crate::decode::decode_refinement::RefinementParams {
                            width: current_width as usize,
                            height: current_height as usize,
                            template_index: params.refinement_template_index,
                            reference_bitmap: symbol,
                            offset_x: rdx,
                            offset_y: rdy,
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
