use crate::bitmap::Bitmap;
use crate::contexts::DecodingContext;
use crate::decoder::decode_integer_context;
use crate::error::Jbig2Error;

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
}

pub fn decode_symbol_dictionary(
    params: &SymbolDictionaryParams,
    decoding_context: &mut DecodingContext,
) -> Result<Vec<Bitmap>, Jbig2Error> {
    let mut new_symbols = Vec::new();
    let mut current_height = 0i32;
    let _symbol_code_length = crate::core_utils::log2((params.symbols.len() + params.number_of_new_symbols) as u32);
    while new_symbols.len() < params.number_of_new_symbols {
        let delta_height = decode_integer_context(decoding_context, "IADH")?.unwrap_or(0);
        current_height += delta_height as i32;
        let mut current_width = 0i32;
        while current_width >= 0 {
            let delta_width = decode_integer_context(decoding_context, "IADW")?;
            if delta_width.is_none() {
                break; // OOB
            }
            current_width += delta_width.unwrap() as i32;
            if params.refinement {
                // For now, skip refinement
                return Err(Jbig2Error::new("refinement not implemented"));
            } else {
                // Direct-coded symbol bitmap - simplified implementation
                let bitmap = Bitmap::new(current_width as usize, current_height as usize);
                // TODO: Implement proper bitmap decoding here
                new_symbols.push(bitmap);
            }
        }
    }
    // Exported symbols
    let mut exported_symbols = Vec::new();
    let mut flags = Vec::new();
    let total_symbols_length = params.symbols.len() + params.number_of_new_symbols;
    let mut current_flag = false;
    while flags.len() < total_symbols_length {
        let run_length = decode_integer_context(decoding_context, "IAEX")?;
        let run_length = run_length.unwrap_or(0) as usize;
        for _ in 0..run_length {
            flags.push(current_flag);
        }
        current_flag = !current_flag;
    }
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