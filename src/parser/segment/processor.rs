// Dispatch parsed segments to visitor callbacks.

use super::parser::{
    parse_halftone_region_params, parse_pattern_dictionary_params, parse_text_region_params,
    read_region_segment_information,
};
use super::types::*;
use super::utils::*;
use crate::common::error::Jbig2Error;
use crate::document::PageInfo;
use crate::parser::visitor::SimpleSegmentVisitor;

/// Group segments by page and dispatch them to the visitor in decode order.
pub fn process_segments<'a>(
    segments: &[Segment<'a>],
    visitor: &mut SimpleSegmentVisitor,
) -> Result<(), Jbig2Error> {
    let strict = false;
    let mut current_page = 0u32;
    let mut page_segments = Vec::new();
    for segment in segments {
        let page_association = segment.header.page_association;
        let is_global = page_association == 0;
        let is_page_info = segment.header.segment_type == 48;
        let is_extension = segment.header.segment_type == 62;
        let should_process = is_global
            || is_page_info
            || is_extension
            || (page_association == current_page && current_page > 0)
            || (page_association == 1 && current_page == 0);
        if !should_process {
            continue;
        }
        // Extension segments may carry page metadata, so process them immediately.
        if is_extension {
            if let Err(err) = process_segment(segment, visitor) {
                if strict || visitor.current_page_info.is_none() {
                    return Err(err);
                }
                return Ok(());
            }
            continue;
        }
        page_segments.push(segment);
        if segment.header.segment_type == 48 {
            if let Err(err) = process_page_segments(&page_segments, visitor) {
                if strict || visitor.current_page_info.is_none() {
                    return Err(err);
                }
                return Ok(());
            }
            page_segments.clear();
            current_page += 1;
        }
    }
    if !page_segments.is_empty() {
        if let Err(err) = process_page_segments(&page_segments, visitor) {
            if strict || visitor.current_page_info.is_none() {
                return Err(err);
            }
            return Ok(());
        }
    }
    Ok(())
}

/// Dispatch segments for a single page, honoring retain ordering.
fn process_page_segments<'a>(
    segments: &[&Segment<'a>],
    visitor: &mut SimpleSegmentVisitor,
) -> Result<(), Jbig2Error> {
    let mut retain_segments = Vec::with_capacity(segments.len());
    let mut non_retain_segments = Vec::new();
    for &segment in segments {
        if segment.header.deferred_non_retain {
            non_retain_segments.push(segment);
        } else {
            retain_segments.push(segment);
        }
    }
    for &segment in &retain_segments {
        process_segment(segment, visitor)?;
    }
    for &segment in &non_retain_segments {
        process_segment(segment, visitor)?;
    }
    Ok(())
}

/// Parse a segment payload and call the appropriate visitor hook.
pub fn process_segment<'a>(
    segment: &Segment<'a>,
    visitor: &mut SimpleSegmentVisitor,
) -> Result<(), Jbig2Error> {
    let header = &segment.header;
    let data = segment.data;
    let start = segment.start;
    let end = segment.end;
    if start > end || end > data.len() {
        return Err(Jbig2Error::new(ERR_INVALID_SEGMENT));
    }
    match header.segment_type {
        0 => {
            require_payload(start, end, 10)?;
            let dictionary_flags = read_u16(data, start);

            // Decode dictionary flags and optional AT parameters.
            let sdhuff = (dictionary_flags & 1) != 0;
            let sdrefagg = ((dictionary_flags >> 1) & 1) != 0;
            let sdtemplate = ((dictionary_flags >> 10) & 3) as usize;
            let sdrtemplate = ((dictionary_flags >> 12) & 1) != 0;

            let mut offset = start + 2;
            let mut at_pixels = Vec::new();
            let mut refinement_at_pixels = Vec::new();

            // Parse direct coding AT pixels when present.
            if !sdhuff {
                let at_length = if sdtemplate == 0 { 4 } else { 1 };
                at_pixels = parse_at_parameters(data, offset, at_length)?;
                offset += at_length * 2;
            }

            // Parse refinement AT pixels when present.
            if sdrefagg && !sdrtemplate {
                refinement_at_pixels = parse_at_parameters(data, offset, 2)?;
                offset += 4; // 4 bytes for refinement AT
            }

            if offset + 8 > end {
                return Err(
                    Jbig2Error::insufficient_data(offset + 8 - start, end - start)
                        .with_position(start),
                );
            }

            // Read symbol counts after the variable-size header fields.
            let number_of_exported_symbols = read_u32(data, offset);
            let number_of_new_symbols = read_u32(data, offset + 4);

            // Guard against obviously malformed counts.
            if number_of_new_symbols > 1_000_000 {
                return Err(Jbig2Error::new(&format!(
                    "Unreasonable symbol count: {} (likely parameter parsing error)",
                    number_of_new_symbols
                )));
            }

            let params = SymbolDictionaryParams {
                dictionary_flags,
                number_of_exported_symbols,
                number_of_new_symbols,
                current_segment: header.number,
                referred_segments: &header.referred_to,
                data,
                start: offset + 8, // Data starts after both counts
                end,
                at_pixels,
                refinement_at_pixels,
            };
            visitor.on_symbol_dictionary(&params)?;
        }
        6 | 7 => {
            require_payload(start, end, REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 6)?;
            // Dispatch immediate text region parameters and payload.
            let params = parse_text_region_params(data, start);

            visitor.on_immediate_text_region(
                &params.region_info,
                params.text_region_segment_flags,
                params.number_of_symbol_instances,
                &header.referred_to,
                data,
                start,
                end,
            )?;
        }
        48 => {
            require_payload(start, end, 19)?;
            // Parse page information and initialize the page state.
            let mut width = read_u32(data, start);
            let mut height = read_u32(data, start + 4);
            let resolution_x = read_u32(data, start + 8);
            let resolution_y = read_u32(data, start + 12);
            let page_segment_flags = data[start + 16];
            let striping = read_u16(data, start + 17);
            let mut striped = (striping & 0x8000) != 0;
            let mut stripe_size = striping & 0x7fff;
            let height_unknown = height == 0xffffffff;

            if width == 0 || height == 0 {
                width = 1;
                height = 1;
            }
            let lossless = (page_segment_flags & 1) != 0;
            let refinement = (page_segment_flags & 2) != 0;
            let default_pixel_value = (page_segment_flags >> 2) & 1;
            let combination_operator = (page_segment_flags >> 3) & 3;
            let requires_buffer = (page_segment_flags & 32) != 0;
            let combination_operator_override = (page_segment_flags & 64) != 0;
            if height_unknown && !striped {
                striped = true;
                stripe_size = 0x7fff;
            }
            if height_unknown {
                let initial_height = if stripe_size == 0 {
                    1
                } else {
                    stripe_size as u32
                };
                height = initial_height;
            }
            let info = PageInfo {
                width,
                height,
                resolution_x,
                resolution_y,
                lossless,
                refinement,
                default_pixel_value,
                combination_operator,
                requires_buffer,
                combination_operator_override,
                striped,
                stripe_size,
                height_unknown,
            };

            visitor.on_page_information(info)?;
        }
        16 => {
            require_payload(start, end, 7)?;
            // Dispatch pattern dictionary parameters and payload.
            let params = parse_pattern_dictionary_params(data, start);
            visitor.on_pattern_dictionary(
                params.mmr,
                params.pattern_width,
                params.pattern_height,
                params.max_pattern_index,
                params.template,
                header.number,
                data,
                start + 7,
                end,
            )?;
        }
        22 | 23 => {
            require_payload(start, end, REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 21)?;
            // Dispatch immediate halftone region parameters and payload.
            let params = parse_halftone_region_params(data, start);
            visitor.on_immediate_halftone_region(
                &params.region_info,
                params.mmr,
                params.template,
                params.enable_skip,
                params.combination_operator,
                params.default_pixel_value,
                params.grid_width,
                params.grid_height,
                params.grid_offset_x,
                params.grid_offset_y,
                params.grid_vector_x,
                params.grid_vector_y,
                &header.referred_to,
                data,
                start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 21,
                end,
            )?;
        }
        38 | 39 => {
            require_payload(start, end, REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 1)?;
            // Dispatch immediate generic region payload.
            use super::parser::read_generic_region;
            let generic_region = read_generic_region(data, start)?;
            visitor.on_immediate_generic_region(&generic_region, data, start, end)?;
        }
        42 | 43 => {
            require_payload(start, end, REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 1)?;
            // Dispatch immediate generic refinement region payload.
            let region_info = read_region_segment_information(data, start);
            visitor.on_immediate_generic_refinement_region(
                &region_info,
                &header.referred_to,
                data,
                start,
                end,
            )?;
        }
        4 => {
            require_payload(start, end, REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 6)?;
            // Dispatch intermediate text region payload.
            let params = parse_text_region_params(data, start);
            visitor.on_intermediate_text_region(
                &params.region_info,
                params.text_region_segment_flags,
                params.number_of_symbol_instances,
                &header.referred_to,
                data,
                start,
                end,
                header.number,
            )?;
        }
        20 => {
            require_payload(start, end, REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 21)?;
            // Dispatch intermediate halftone region payload.
            let params = parse_halftone_region_params(data, start);
            visitor.on_intermediate_halftone_region(
                &params.region_info,
                params.mmr,
                params.template,
                params.enable_skip,
                params.combination_operator,
                params.default_pixel_value,
                params.grid_width,
                params.grid_height,
                params.grid_offset_x,
                params.grid_offset_y,
                params.grid_vector_x,
                params.grid_vector_y,
                &header.referred_to,
                data,
                start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 21,
                end,
                header.number,
            )?;
        }
        36 => {
            require_payload(start, end, REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 1)?;
            // Dispatch intermediate generic region payload.
            use super::parser::read_generic_region;
            let generic_region = read_generic_region(data, start)?;
            visitor.on_intermediate_generic_region(
                &generic_region,
                &header.referred_to,
                data,
                start,
                end,
                header.number,
            )?;
        }
        40 => {
            require_payload(start, end, REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 1)?;
            // Dispatch intermediate generic refinement region payload.
            let region_info = read_region_segment_information(data, start);
            visitor.on_intermediate_generic_refinement_region(
                &region_info,
                &header.referred_to,
                data,
                start,
                end,
                header.number,
            )?;
        }
        49 => {
            // End-of-page marker does not carry payload.
        }
        50 => {
            require_payload(start, end, 4)?;
            // End-of-stripe carries the stripe height.
            let height = read_u32(data, start) as usize;
            visitor.on_end_of_stripe(height)?;
        }
        51 => {
            // End-of-file marker does not carry payload.
        }
        52 => {
            // Profiles are optional metadata and not required for decoding.
        }
        53 => {
            // Dispatch custom Huffman tables.
            visitor.on_tables(header.number, data, start, end)?;
        }
        62 => {
            // Extension segments are currently ignored.
        }
        _ => {} // Unknown segment types are skipped.
    }
    Ok(())
}

fn require_payload(start: usize, end: usize, required: usize) -> Result<(), Jbig2Error> {
    let available = end.saturating_sub(start);
    if available < required {
        return Err(Jbig2Error::insufficient_data(required, available).with_position(start));
    }
    Ok(())
}
