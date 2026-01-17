use crate::bitmap::Bitmap;
use crate::contexts::DecodingContext;
use crate::decode::decode_symbol::decode_symbol_dictionary;
use crate::error::Jbig2Error;
use crate::huffman::HuffmanTable;
use crate::reader::Reader;
use crate::segment::SymbolDictionaryParams;
use std::collections::HashMap;

/// Gather symbols referenced by the given segment list.
pub(super) fn collect_input_symbols(
    symbols: &HashMap<u32, Vec<Bitmap>>,
    referred_segments: &[u32],
) -> Vec<Bitmap> {
    let mut input_symbols = Vec::new();

    for &segment_id in referred_segments {
        if let Some(symbols) = symbols.get(&segment_id) {
            input_symbols.extend(symbols.clone());
        }
    }
    input_symbols
}

/// Decode a symbol dictionary segment and store the resulting symbols.
pub(super) fn on_symbol_dictionary(
    symbols: &mut HashMap<u32, Vec<Bitmap>>,
    custom_tables: &HashMap<u32, HuffmanTable>,
    params: &SymbolDictionaryParams,
) -> Result<(), Jbig2Error> {
    if params.start >= params.end {
        return Ok(());
    }

    // Avoid decoding obviously malformed symbol counts.
    if params.number_of_new_symbols > 10000 {
        return Ok(());
    }

    let huffman = (params.dictionary_flags & 1) != 0;
    let refinement = (params.dictionary_flags & 2) != 0;
    let template = ((params.dictionary_flags >> 10) & 3) as usize;
    let refinement_template = ((params.dictionary_flags >> 12) & 1) as usize;

    let at = params.at_pixels.clone();
    let refinement_at = params.refinement_at_pixels.clone();

    let slice = &params.data[params.start..params.end];
    let mut decoding_context = DecodingContext::new(slice.to_vec(), 0, slice.len());

    // Build Huffman tables when Huffman coding is enabled.
    let huffman_tables = if huffman {
        Some(crate::huffman::get_symbol_dictionary_huffman_tables(
            ((params.dictionary_flags >> 2) & 3) as u8, // huffmanDHSelector
            ((params.dictionary_flags >> 4) & 3) as u8, // huffmanDWSelector
            ((params.dictionary_flags >> 6) & 1) != 0,  // bitmapSizeSelector
            ((params.dictionary_flags >> 7) & 1) != 0,  // aggregationInstancesSelector
            params.referred_segments,
            custom_tables,
        )?)
    } else {
        None
    };

    let symbol_params = crate::decode::decode_symbol::SymbolDictionaryParams {
        huffman,
        refinement,
        symbols: collect_input_symbols(symbols, params.referred_segments),
        number_of_new_symbols: params.number_of_new_symbols as usize,
        number_of_exported_symbols: params.number_of_exported_symbols as usize,
        template_index: template,
        at,
        refinement_template_index: refinement_template,
        refinement_at,
        huffman_tables,
    };

    let mut huffman_input = if huffman {
        Some(Reader::new(slice.to_vec(), 0, slice.len()))
    } else {
        None
    };

    let exported_symbols = decode_symbol_dictionary(
        &symbol_params,
        &mut decoding_context,
        huffman_input.as_mut(),
    )?;

    symbols.insert(params.current_segment, exported_symbols);

    Ok(())
}
