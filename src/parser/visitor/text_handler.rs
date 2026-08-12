use crate::arithmetic::contexts::DecodingContext;
use crate::bitmap::Bitmap;
use crate::common::error::Jbig2Error;
use crate::common::reader::Reader;
use crate::decoders::text::decode_text_region;
use crate::huffman::{HuffmanTable, TextRegionHuffmanParams};
use crate::parser::segment::{RegionInfo, TextRegionParams, read_u16};
use std::collections::HashMap;

use super::region_handlers::{REGION_SEGMENT_INFORMATION_FIELD_LENGTH, draw_bitmap};
use super::symbol_handler::collect_input_symbols;
use super::{IntermediateResources, PageComposeTarget, SegmentSlice};

/// Decode an immediate text region and composite it onto the current page.
pub(super) fn on_immediate_text_region(
    page: PageComposeTarget<'_>,
    symbols: &HashMap<u32, Vec<Bitmap>>,
    custom_tables: &HashMap<u32, HuffmanTable>,
    params: &TextRegionParams,
    referred_to: &[u32],
    bytes: SegmentSlice<'_>,
) -> Result<(), Jbig2Error> {
    ensure_page_for_region(page.page_info, page.bitmap, &params.region_info);
    let bitmap = decode_parsed_text_region(symbols, custom_tables, params, referred_to, bytes)?;
    draw_bitmap(page, &params.region_info, &bitmap)
}

/// Decode an intermediate text region without compositing it onto the page.
pub(super) fn on_intermediate_text_region(
    resources: IntermediateResources<'_>,
    params: &TextRegionParams,
    referred_to: &[u32],
    bytes: SegmentSlice<'_>,
    segment_number: u32,
) -> Result<(), Jbig2Error> {
    for &seg_id in referred_to {
        if !resources.symbols.contains_key(&seg_id)
            && !resources.patterns.contains_key(&seg_id)
            && !resources.custom_tables.contains_key(&seg_id)
            && !resources.bitmaps.contains_key(&seg_id)
        {
            return Err(Jbig2Error::new("referred segment not found"));
        }
    }

    let bitmap = decode_parsed_text_region(
        resources.symbols,
        resources.custom_tables,
        params,
        referred_to,
        bytes,
    )?;
    resources.bitmaps.insert(segment_number, bitmap);
    Ok(())
}

fn ensure_page_for_region(
    current_page_info: &mut Option<crate::document::PageInfo>,
    current_bitmap: &mut Option<Bitmap>,
    region_info: &RegionInfo,
) {
    if current_page_info.is_some() {
        return;
    }
    *current_page_info = Some(crate::document::PageInfo {
        width: region_info.width,
        height: region_info.height,
        resolution_x: 0,
        resolution_y: 0,
        lossless: true,
        refinement: false,
        default_pixel_value: 0,
        combination_operator: 0, // OR
        requires_buffer: false,
        combination_operator_override: false,
        striped: false,
        stripe_size: 0,
        height_unknown: false,
    });
    *current_bitmap = Some(crate::bitmap::utils::create_initialized_bitmap(
        region_info.width as usize,
        region_info.height as usize,
        0,
    ));
}

fn decode_parsed_text_region(
    symbols: &HashMap<u32, Vec<Bitmap>>,
    custom_tables: &HashMap<u32, HuffmanTable>,
    params: &TextRegionParams,
    referred_to: &[u32],
    bytes: SegmentSlice<'_>,
) -> Result<Bitmap, Jbig2Error> {
    let region_info = &params.region_info;
    let text_region_segment_flags = params.text_region_segment_flags;
    let data = bytes.data;
    let start = bytes.start;
    let end = bytes.end;

    let huffman = (text_region_segment_flags & 1) != 0;
    let refinement = (text_region_segment_flags & 2) != 0;
    let log_strip_size = ((text_region_segment_flags >> 2) & 3) as usize;
    let strip_size = 1 << log_strip_size;
    let reference_corner = ((text_region_segment_flags >> 4) & 3) as usize;
    let transposed = (text_region_segment_flags & 64) != 0;
    let combination_operator = ((text_region_segment_flags >> 7) & 3) as usize;
    let default_pixel_value = ((text_region_segment_flags >> 9) & 1) as u8;
    let ds_offset = ((text_region_segment_flags as i32) << 17) >> 27;
    let refinement_template = ((text_region_segment_flags >> 15) & 1) as usize;

    // Gather the symbol bitmaps referenced by this segment.
    let input_symbols = collect_input_symbols(symbols, referred_to);
    let symbol_code_length = crate::common::utils::log2(input_symbols.len() as u32);

    // Parse Huffman selector flags and refinement AT offsets.
    let mut refinement_at = Vec::new();
    let mut pos = start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 2;

    let mut huffman_fs = 0u8;
    let mut huffman_ds = 0u8;
    let mut huffman_dt = 0u8;
    let mut huffman_refinement_dw = 0u8;
    let mut huffman_refinement_dh = 0u8;
    let mut huffman_refinement_dx = 0u8;
    let mut huffman_refinement_dy = 0u8;
    let mut huffman_refinement_size_selector = false;
    let mut huffman_ri = false;

    if huffman && pos + 2 <= end {
        let huffman_flags = read_u16(data, pos);
        pos += 2;
        huffman_fs = (huffman_flags & 3) as u8;
        huffman_ds = ((huffman_flags >> 2) & 3) as u8;
        huffman_dt = ((huffman_flags >> 4) & 3) as u8;
        huffman_refinement_dw = ((huffman_flags >> 6) & 3) as u8;
        huffman_refinement_dh = ((huffman_flags >> 8) & 3) as u8;
        huffman_refinement_dx = ((huffman_flags >> 10) & 3) as u8;
        huffman_refinement_dy = ((huffman_flags >> 12) & 3) as u8;
        huffman_refinement_size_selector = (huffman_flags & 0x4000) != 0;
        huffman_ri = (huffman_flags & 0x8000) != 0;
    }

    if refinement && refinement_template == 0 && pos + 4 <= end {
        for _ in 0..2 {
            let x = data[pos] as i8;
            let y = data[pos + 1] as i8;
            refinement_at.push((x, y));
            pos += 2;
        }
    }

    if pos + 4 > end {
        return Err(Jbig2Error::new(
            "text region segment too short for instance count",
        ));
    }
    pos += 4;

    let slice = &data[pos.min(end)..end];

    let mut decoding_context = DecodingContext::new(slice, 0, slice.len());

    // Build Huffman tables and readers when Huffman coding is enabled.
    let mut huffman_reader = if huffman {
        Some(Reader::new(slice, 0, slice.len()))
    } else {
        None
    };

    let huffman_tables = if let Some(ref mut reader) = huffman_reader {
        let params = TextRegionHuffmanParams {
            huffman_fs,
            huffman_ds,
            huffman_dt,
            huffman_refinement_dw,
            huffman_refinement_dh,
            huffman_refinement_dx,
            huffman_refinement_dy,
            huffman_refinement_size_selector,
            huffman_ri,
        };
        Some(crate::huffman::get_text_region_huffman_tables(
            &params,
            referred_to,
            custom_tables,
            input_symbols.len(),
            reader,
        )?)
    } else {
        None
    };

    let symbol_id_limit = input_symbols.len();
    let params = crate::decoders::text::TextRegionParams {
        huffman,
        refinement,
        width: region_info.width as usize,
        height: region_info.height as usize,
        default_pixel_value,
        number_of_symbol_instances: params.number_of_symbol_instances as usize,
        strip_size,
        input_symbols,
        symbol_code_length: symbol_code_length as usize,
        symbol_id_limit,
        transposed,
        ds_offset,
        reference_corner,
        combination_operator,
        log_strip_size,
        huffman_tables,
        refinement_template_index: refinement_template,
        refinement_at: &refinement_at,
    };

    decode_text_region(&params, &mut decoding_context, huffman_reader.as_mut())
}
