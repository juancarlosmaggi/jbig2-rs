use crate::bitmap::Bitmap;
use crate::contexts::DecodingContext;
use crate::decode::decode_symbol::decode_symbol_dictionary;
use crate::error::Jbig2Error;
use crate::huffman::HuffmanTable;
use crate::reader::Reader;
use crate::segment::SymbolDictionaryParams;
use std::collections::HashMap;

/// Collect input symbols from referred segments
pub(super) fn collect_input_symbols(
    symbols: &HashMap<u32, Vec<Bitmap>>,
    referred_segments: &[u32],
) -> Vec<Bitmap> {
    let mut input_symbols = Vec::new();
    eprintln!("collect_input_symbols: Looking for symbols in referred segments: {:?}", referred_segments);
    eprintln!("  Currently stored symbol dictionaries: {:?}", symbols.keys().collect::<Vec<_>>());
    
    for &segment_id in referred_segments {
        eprintln!("  Checking segment {}", segment_id);
        if let Some(symbols) = symbols.get(&segment_id) {
            eprintln!("    → Found {} symbols in segment {}", symbols.len(), segment_id);
            input_symbols.extend(symbols.clone());
        } else {
            eprintln!("    → Segment {} not found in symbol storage", segment_id);
        }
    }
    eprintln!("  Total symbols collected: {}", input_symbols.len());
    input_symbols
}

/// Handle symbol dictionary segment
pub(super) fn on_symbol_dictionary(
    symbols: &mut HashMap<u32, Vec<Bitmap>>,
    custom_tables: &HashMap<u32, HuffmanTable>,
    params: &SymbolDictionaryParams,
) -> Result<(), Jbig2Error> {
    if params.start >= params.end {
        println!(
            "Skipping symbol dictionary due to invalid bounds: start={}, end={}",
            params.start, params.end
        );
        return Ok(());
    }

    println!(
        "Entering on_symbol_dictionary, start: {}, end: {}",
        params.start, params.end
    );
    eprintln!("DEBUG: Segment {}, new_symbols={}, exported_symbols={}", 
              params.current_segment, params.number_of_new_symbols, params.number_of_exported_symbols);

    if params.start >= params.end {
        eprintln!("Skipping: Invalid bounds");
        println!(
            "Skipping symbol dictionary due to invalid bounds: start={}, end={}",
            params.start, params.end
        );
        return Ok(());
    }

    // Skip processing if too many symbols to prevent errors
    if params.number_of_new_symbols > 10000 {
        eprintln!("Skipping: Too many new symbols: {}", params.number_of_new_symbols);
        return Ok(());
    }

    let huffman = (params.dictionary_flags & 1) != 0;
    let refinement = (params.dictionary_flags & 2) != 0;
    let template = ((params.dictionary_flags >> 10) & 3) as usize;
    let refinement_template = ((params.dictionary_flags >> 12) & 1) as usize;

    // Parse AT parameters from the segment data if needed
    // NOTE: params.start already points AFTER flags, AT pixels, and counts!
    // So we need to look BACKWARDS in the data to find AT values
    let mut at = Vec::new();
    let mut refinement_at = Vec::new();
    
    if !huffman {
        // AT pixels are at fixed offset: 2 bytes after flags
        let _at_offset = 2;  // Right after 2-byte flags in segment data
        let at_length = if template == 0 { 4 } else { 1 };
        
        // Parse from original segment data (params.data contains full segment)
        // params.start points to decode data, so we need to calculate backwards
        let segment_data_start = params.start - 8;  // Back past counts (8 bytes)
        let at_bytes_count = if template == 0 { 8 } else { 2 };
        let at_data_start = segment_data_start - at_bytes_count;
        
        if at_data_start + at_bytes_count <= params.data.len() {
            for i in 0..at_length {
                let x = params.data[at_data_start + i * 2] as i8;
                let y = params.data[at_data_start + i * 2 + 1] as i8;
                at.push((x, y));
            }
            eprintln!("DEBUG: Parsed AT pixels: {:?}", at);
        }
    }

    if refinement && refinement_template == 0 {
        // Similar backward calculation for refinement AT
        // These come after direct coding AT (if present) and before counts
        let segment_data_start = params.start - 8;
        let at_bytes_count = if !huffman {
            if template == 0 { 8 } else { 2 }
        } else {
            0
        };
        let refinement_at_start = segment_data_start - 4;  // 4 bytes for refinement AT
        
        if refinement_at_start >= at_bytes_count && refinement_at_start + 4 <= params.data.len() {
            for i in 0..2 {
                let x = params.data[refinement_at_start + i * 2] as i8;
                let y = params.data[refinement_at_start + i * 2 + 1] as i8;
                refinement_at.push((x, y));
            }
        }
    }

    // Create decoding context starting at params.start (which is AFTER all parameters!)
    eprintln!("Symbol dict decode: params.start={} (0x{:X}), params.end={}", 
              params.start, params.start, params.end);
    eprintln!("First 20 bytes at params.start in ORIGINAL file data:");
    for i in 0..20.min(params.end - params.start) {
        eprint!("{:02X} ", params.data[params.start + i]);
    }
    eprintln!();
    
    let slice = &params.data[params.start..params.end];
    eprintln!("Symbol dict decode: Creating context from offset {}, length {}", 
              params.start, slice.len());

    let mut decoding_context = DecodingContext::new(slice.to_vec(), 0, slice.len());

    // Get Huffman tables if needed
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

    eprintln!("About to decode symbol dictionary for segment {}", params.current_segment);
    
    let exported_symbols = match decode_symbol_dictionary(
        &symbol_params,
        &mut decoding_context,
        huffman_input.as_mut(),
    ) {
        Ok(symbols) => {
            eprintln!("Symbol dictionary {}: Successfully decoded {} exported symbols", 
                      params.current_segment, symbols.len());
            symbols
        }
        Err(e) => {
            eprintln!("Symbol dictionary {}: FAILED to decode: {}", params.current_segment, e);
            return Err(e);
        }
    };
    
    symbols.insert(params.current_segment, exported_symbols.clone());
    
    eprintln!("  Stored in self.symbols[{}], total dicts now: {}", 
              params.current_segment, symbols.len());

    Ok(())
}
