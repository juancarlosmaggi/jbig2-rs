// Helpers shared by symbol dictionary decoding paths.
use crate::arithmetic::contexts::DecodingContext;
use crate::bitmap::Bitmap;
use crate::common::error::Jbig2Error;
use crate::common::reader::Reader;
use crate::decoders::text::TextRegionParams;
use crate::huffman::{TextRegionHuffmanTables, get_aggregate_symbol_huffman_tables};

/// Parameters for decoding an aggregate symbol bitmap.
#[derive(Clone)]
pub struct AggregateSymbolParams {
    pub current_width: i32,
    pub current_height: i32,
    pub number_of_instances: i32,
    pub symbol_code_length: usize,
    pub total_symbols: usize,
    pub refinement: bool,
    pub refinement_template_index: usize,
    pub refinement_at: Vec<(i8, i8)>,
    pub huffman: bool,
}

/// Split a collective bitmap into individual symbol bitmaps.
pub fn split_collective_bitmap(
    collective_bitmap: &Bitmap,
    symbol_widths: &[usize],
    height: usize,
) -> Vec<Bitmap> {
    let mut symbols = Vec::with_capacity(symbol_widths.len());
    let mut x_offset = 0usize;
    let collective_stride = collective_bitmap.stride;

    for &width in symbol_widths {
        let mut symbol_bitmap = Bitmap::new(width, height);
        let symbol_stride = symbol_bitmap.stride;
        if symbol_stride == 0 || height == 0 {
            symbols.push(symbol_bitmap);
            x_offset += width;
            continue;
        }

        let rem_bits = width & 7;
        let tail_mask = if rem_bits == 0 {
            0xFF
        } else {
            0xFFu8 << (8 - rem_bits)
        };
        let src_byte_offset = x_offset >> 3;
        let src_bit_offset = (x_offset & 7) as u8;

        for y in 0..height {
            let src_row_start = y * collective_stride + src_byte_offset;
            let src_row_end = y * collective_stride + collective_stride;
            let src_row = &collective_bitmap.data[src_row_start..src_row_end];
            debug_assert!(src_row.len() >= symbol_stride);
            let dst_row_start = y * symbol_stride;
            let dst_row = &mut symbol_bitmap.data[dst_row_start..dst_row_start + symbol_stride];

            if src_bit_offset == 0 {
                dst_row.copy_from_slice(&src_row[..symbol_stride]);
            } else {
                let inv_shift = 8 - src_bit_offset;
                for b in 0..symbol_stride {
                    let cur = src_row[b];
                    let next = if b + 1 < src_row.len() {
                        src_row[b + 1]
                    } else {
                        0
                    };
                    dst_row[b] = (cur << src_bit_offset) | (next >> inv_shift);
                }
            }

            if rem_bits != 0 {
                dst_row[symbol_stride - 1] &= tail_mask;
            }
        }

        symbols.push(symbol_bitmap);
        x_offset += width;
    }

    symbols
}

/// Build text-region parameters for aggregate symbol decoding.
pub fn create_aggregate_text_params<'a>(
    params: &AggregateSymbolParams,
    input_symbols: Vec<&'a Bitmap>,
    huffman_tables: Option<TextRegionHuffmanTables>,
) -> TextRegionParams<'a> {
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
        symbol_id_limit: params.total_symbols,
        transposed: false,
        ds_offset: 0,
        reference_corner: 1,     // top-left reference point for aggregate symbols
        combination_operator: 0, // OR
        log_strip_size: 0,
        huffman_tables,
        refinement_template_index: params.refinement_template_index,
        refinement_at: params.refinement_at.clone(),
    }
}

/// Decode an aggregate symbol bitmap using text-region decoding.
pub fn decode_aggregate_symbol(
    params: &AggregateSymbolParams,
    existing_symbols: &[&Bitmap],
    new_symbols: &[Bitmap],
    decoding_context: &mut DecodingContext<'_>,
    mut huffman_input: Option<&mut Reader<'_>>,
) -> Result<Bitmap, Jbig2Error> {
    let mut input_symbols = Vec::with_capacity(existing_symbols.len() + new_symbols.len());
    input_symbols.extend(existing_symbols.iter().copied());
    input_symbols.extend(new_symbols.iter());

    let huffman_tables = if params.huffman {
        let reader = huffman_input
            .as_mut()
            .ok_or_else(|| Jbig2Error::new("missing Huffman input"))?;
        Some(get_aggregate_symbol_huffman_tables(
            reader,
            params.total_symbols,
        )?)
    } else {
        None
    };

    let text_params = create_aggregate_text_params(params, input_symbols, huffman_tables);

    crate::decoders::text::decode_text_region(&text_params, decoding_context, huffman_input)
}
