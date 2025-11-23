use crate::bitmap::Bitmap;
use crate::contexts::DecodingContext;
use crate::decode::decode_text::decode_text_region;
use crate::error::Jbig2Error;
use crate::huffman::{HuffmanTable, TextRegionHuffmanParams};
use crate::reader::Reader;
use crate::segment::{PageInfo, RegionInfo, read_u16};
use std::collections::HashMap;

use super::region_handlers::{REGION_SEGMENT_INFORMATION_FIELD_LENGTH, draw_bitmap};
use super::symbol_handler::collect_input_symbols;

/// Handle immediate text region
#[allow(clippy::too_many_arguments)]
pub(super) fn on_immediate_text_region(
    current_page_info: &mut Option<PageInfo>,
    current_bitmap: &mut Option<Bitmap>,
    current_y: usize,
    symbols: &HashMap<u32, Vec<Bitmap>>,
    custom_tables: &HashMap<u32, HuffmanTable>,
    region_info: &RegionInfo,
    text_region_segment_flags: u16,
    number_of_symbol_instances: u32,
    referred_to: &[u32],
    data: &[u8],
    start: usize,
    end: usize,
) -> Result<(), Jbig2Error> {
    if current_page_info.is_none() {
        *current_page_info = Some(PageInfo {
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
        });
        let width = region_info.width as usize;
        let height = region_info.height as usize;
        *current_bitmap = Some(crate::bitmap_utils::create_initialized_bitmap(
            width, height, 0,
        ));
    }

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

    // Collect input symbols from referred segments
    let input_symbols = collect_input_symbols(symbols, referred_to);
    let symbol_code_length = crate::core_utils::log2(input_symbols.len() as u32);

    // Parse Huffman flags and refinement AT
    let mut refinement_at = Vec::new();
    let mut pos = start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 2; // flags are 2 bytes

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
    } // else default 0

    if refinement && refinement_template == 0 && pos + 4 <= end {
        for _ in 0..2 {
            let x = data[pos] as i8;
            let y = data[pos + 1] as i8;
            refinement_at.push((x, y));
            pos += 2;
        }
    } // else default empty

    let slice = &data[pos.min(end)..end];

    let mut decoding_context = DecodingContext::new(slice.to_vec(), 0, slice.len());

    // Get Huffman tables if needed
    let mut huffman_reader = if huffman {
        Some(Reader::new(slice.to_vec(), 0, slice.len()))
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

    let params = crate::decode::decode_text::TextRegionParams {
        huffman,
        refinement,
        width: region_info.width as usize,
        height: region_info.height as usize,
        default_pixel_value,
        number_of_symbol_instances: number_of_symbol_instances as usize,
        strip_size,
        input_symbols,
        symbol_code_length: symbol_code_length as usize,
        transposed,
        ds_offset,
        reference_corner,
        combination_operator,
        log_strip_size,
        huffman_tables,
        refinement_template_index: refinement_template,
        refinement_at,
    };

    let bitmap = decode_text_region(&params, &mut decoding_context, huffman_reader.as_mut())?;

    draw_bitmap(
        current_page_info,
        current_bitmap,
        current_y,
        region_info,
        &bitmap,
    )?;
    Ok(())
}

/// Handle intermediate text region
#[allow(clippy::too_many_arguments)]
pub(super) fn on_intermediate_text_region(
    symbols: &HashMap<u32, Vec<Bitmap>>,
    patterns: &HashMap<u32, Vec<Bitmap>>,
    custom_tables: &HashMap<u32, HuffmanTable>,
    bitmaps: &HashMap<u32, Bitmap>,
    region_info: &RegionInfo,
    text_region_segment_flags: u16,
    number_of_symbol_instances: u32,
    referred_to: &[u32],
    data: &[u8],
    start: usize,
    end: usize,
    _segment_number: u32,
) -> Result<(), Jbig2Error> {
    // Basic validation: check that referred segments exist
    for &seg_id in referred_to {
        if !symbols.contains_key(&seg_id)
            && !patterns.contains_key(&seg_id)
            && !custom_tables.contains_key(&seg_id)
            && !bitmaps.contains_key(&seg_id)
        {
            return Err(Jbig2Error::new("referred segment not found"));
        }
    }

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

    // Collect input symbols from referred segments
    let input_symbols = collect_input_symbols(symbols, referred_to);
    let symbol_code_length = crate::core_utils::log2(input_symbols.len() as u32);

    // Parse Huffman flags and refinement AT
    let mut refinement_at = Vec::new();
    let mut pos = start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 2; // flags are 2 bytes

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
    } // else default 0

    if refinement && refinement_template == 0 && pos + 4 <= end {
        for _ in 0..2 {
            let x = data[pos] as i8;
            let y = data[pos + 1] as i8;
            refinement_at.push((x, y));
            pos += 2;
        }
    } // else default empty

    let slice = &data[pos.min(end)..end];

    let mut decoding_context = DecodingContext::new(slice.to_vec(), 0, slice.len());

    // Get Huffman tables if needed
    let mut huffman_reader = if huffman {
        Some(Reader::new(slice.to_vec(), 0, slice.len()))
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

    let params = crate::decode::decode_text::TextRegionParams {
        huffman,
        refinement,
        width: region_info.width as usize,
        height: region_info.height as usize,
        default_pixel_value,
        number_of_symbol_instances: number_of_symbol_instances as usize,
        strip_size,
        input_symbols,
        symbol_code_length: symbol_code_length as usize,
        transposed,
        ds_offset,
        reference_corner,
        combination_operator,
        log_strip_size,
        huffman_tables,
        refinement_template_index: refinement_template,
        refinement_at,
    };

    let _bitmap = decode_text_region(&params, &mut decoding_context, huffman_reader.as_mut())?;

    // Note: intermediate text region doesn't draw to current page
    // The bitmap would typically be stored for later use, but current implementation doesn't
    Ok(())
}
