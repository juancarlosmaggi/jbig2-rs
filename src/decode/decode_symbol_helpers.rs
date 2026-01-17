// Helper functions for symbol dictionary decoding
// Extracted from decode_symbol.rs to reduce duplication and improve maintainability
use crate::bitmap::Bitmap;
use crate::contexts::DecodingContext;
use crate::decode::decode_text::TextRegionParams;
use crate::huffman::{TextRegionHuffmanTables, get_aggregate_symbol_huffman_tables};
use crate::reader::Reader;
use crate::error::Jbig2Error;

#[derive(Clone)]
pub struct AggregateSymbolParams {
    pub current_width: i32,
    pub current_height: i32,
    pub number_of_instances: i32,
    pub symbol_code_length: usize,
    pub refinement: bool,
    pub refinement_template_index: usize,
    pub refinement_at: Vec<(i8, i8)>,
    pub huffman: bool,
}

pub fn split_collective_bitmap(
    collective_bitmap: &Bitmap,
    symbol_widths: &[usize],
    height: usize,
) -> Vec<Bitmap> {
    let mut symbols = Vec::new();
    let mut x_offset = 0; // ← fixed typo

    for &width in symbol_widths {
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

pub fn create_aggregate_text_params(
    params: &AggregateSymbolParams,
    input_symbols: Vec<Bitmap>,
    huffman_tables: Option<TextRegionHuffmanTables>,
) -> TextRegionParams {
    TextRegionParams {
        huffman: params.huffman,
        refinement: params.refinement,
        width: params.current_width as usize,
        height: params.current_height as usize,
        default_pixel_value: 0,
        number_of_symbol_instances: params.number_of_instances as usize,
        strip_size: 1, // aggregate symbol text regions use a single strip
        input_symbols,
        symbol_code_length: params.symbol_code_length,
        transposed: false,
        ds_offset: 0,
        reference_corner: 1, // top-left reference point for aggregate symbols
        combination_operator: 0, // OR
        log_strip_size: 0,
        huffman_tables,
        refinement_template_index: params.refinement_template_index,
        refinement_at: params.refinement_at.clone(),
    }
}

pub fn decode_aggregate_symbol(
    params: &AggregateSymbolParams,
    existing_symbols: &[Bitmap],
    new_symbols: &[Bitmap],
    decoding_context: &mut DecodingContext,
    mut huffman_input: Option<&mut Reader>,
) -> Result<Bitmap, Jbig2Error> {
    let mut input_symbols = existing_symbols.to_vec();
    input_symbols.extend(new_symbols.iter().cloned());

    let huffman_tables = if params.huffman {
        let reader = huffman_input
            .as_mut()
            .ok_or_else(|| Jbig2Error::new("missing Huffman input"))?;
        Some(get_aggregate_symbol_huffman_tables(
            reader,
            input_symbols.len(),
        )?)
    } else {
        None
    };

    let text_params = create_aggregate_text_params(params, input_symbols, huffman_tables);

    crate::decode::decode_text::decode_text_region(
        &text_params,
        decoding_context,
        huffman_input,
    )
}
