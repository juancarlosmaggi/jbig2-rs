// Helper functions for symbol dictionary decoding
// Extracted from decode_symbol.rs to reduce duplication and improve maintainability

use crate::bitmap::Bitmap;
use crate::contexts::DecodingContext;
use crate::decode::decode_text::TextRegionParams;
use crate::error::Jbig2Error;

/// Split a collective bitmap into individual symbol bitmaps
/// This logic was duplicated 3 times in the original code
pub fn split_collective_bitmap(
    collective_bitmap: &Bitmap,
    symbol_widths: &[usize],
    height: usize,
    first_symbol: usize,
) -> Vec<Bitmap> {
    let mut symbols = Vec::new();
    let mut x_offset = 0;
    
    for &width in symbol_widths.iter().skip(first_symbol) {
        let mut symbol_bitmap = Bitmap::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let pixel = collective_bitmap.get_pixel(x_offset + x, y);
                symbol_bitmap.set_pixel(x, y, pixel);
            }
        }
        symbols.push(symbol_bitmap);
        x_offset += width;
    }
    
    symbols
}

/// Parameters for decoding an aggregate symbol
pub struct AggregateSymbolParams {
    pub current_width: i32,
    pub current_height: i32,
    pub number_of_instances: i32,
    pub symbol_code_length: usize,
    pub refinement: bool,
    pub refinement_template_index: usize,
    pub refinement_at: Vec<(i8, i8)>,
}

/// Create TextRegionParams for aggregate symbol decoding
/// This construction was duplicated in both Huffman and Arithmetic paths
pub fn create_aggregate_text_params(
    params: &AggregateSymbolParams,
    input_symbols: Vec<Bitmap>,
) -> TextRegionParams {
    TextRegionParams {
        huffman: false, // Aggregate doesn't use Huffman
        refinement: params.refinement,
        width: params.current_width as usize,
        height: params.current_height as usize,
        default_pixel_value: 0,
        number_of_symbol_instances: params.number_of_instances as usize,
        strip_size: 1,
        input_symbols,
        symbol_code_length: params.symbol_code_length,
        transposed: false,
        ds_offset: 0,
        reference_corner: 1, // top left
        combination_operator: 0, // OR
        log_strip_size: 0,
        huffman_tables: None,
        refinement_template_index: params.refinement_template_index,
        refinement_at: params.refinement_at.clone(),
    }
}

/// Decode an aggregate symbol by invoking text region decoding
pub fn decode_aggregate_symbol(
    params: &AggregateSymbolParams,
    existing_symbols: &[Bitmap],
    new_symbols: &[Bitmap],
    decoding_context: &mut DecodingContext,
) -> Result<Bitmap, Jbig2Error> {
    let mut input_symbols = existing_symbols.to_vec();
    input_symbols.extend(new_symbols.iter().cloned());
    
    let text_params = create_aggregate_text_params(params, input_symbols);
    
    crate::decode::decode_text::decode_text_region(
        &text_params,
        decoding_context,
        None,
    )
}

/// Validate and clamp dimension values to prevent overflow
pub fn validate_dimension(value: i32, max: i32, name: &str) -> Result<i32, Jbig2Error> {
    if value > max {
        return Err(Jbig2Error::new(&format!("{} too large: {} > {}", name, value, max)));
    }
    Ok(value.max(0))
}
