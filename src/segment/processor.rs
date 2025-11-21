// Segment processing logic - dispatches segments to visitor callbacks

use crate::error::Jbig2Error;
use crate::visitor::SimpleSegmentVisitor;
use super::types::*;
use super::utils::*;
use super::parser::{
    read_region_segment_information, 
    parse_halftone_region_params,
    parse_text_region_params,
    parse_pattern_dictionary_params,
};

pub fn process_segments<'a>(
    segments: &[Segment<'a>],
    visitor: &mut SimpleSegmentVisitor,
) -> Result<(), Jbig2Error> {
    let mut current_page = 0u32;
    let mut page_segments = Vec::new();
    for segment in segments {
        let page_association = segment.header.page_association;
        let is_global = page_association == 0;
        let is_page_info = segment.header.segment_type == 48;
        let is_extension = segment.header.segment_type == 62;
        let should_process = is_global
            || is_page_info
            || is_extension // Process extension segments immediately
            || (page_association == current_page && current_page > 0)
            || (page_association == 1 && current_page == 0);
        if !should_process {
            continue;
        }
        // Process extension segments immediately to extract page info
        if is_extension {
            process_segment(segment, visitor)?;
            continue;
        }
        page_segments.push(segment);
        if segment.header.segment_type == 48 {
            process_page_segments(&page_segments, visitor)?;
            page_segments.clear();
            current_page += 1;
        }
    }
    if !page_segments.is_empty() {
        process_page_segments(&page_segments, visitor)?;
    }
    Ok(())
}

fn process_page_segments<'a>(
    segments: &[&Segment<'a>],
    visitor: &mut SimpleSegmentVisitor,
) -> Result<(), Jbig2Error> {
    let mut retain_segments = Vec::new();
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
            let dictionary_flags = read_u16(data, start);
            
            // Parse all relevant flags
            let sdhuff = (dictionary_flags & 1) != 0;
            let sdrefagg = ((dictionary_flags >> 1) & 1) != 0;
            let sdtemplate = ((dictionary_flags >> 10) & 3) as usize;
            let sdrtemplate = ((dictionary_flags >> 12) & 1) != 0;
            
            let mut offset = start + 2;
            
            // AT pixels for direct coding (only if not Huffman)
            if !sdhuff {
                let sdat_bytes = if sdtemplate == 0 { 8 } else { 2 };
                offset += sdat_bytes;
            }
            
            // Refinement AT pixels (Table 18 in JBIG2 spec)
            if sdrefagg && !sdrtemplate {
                offset += 4;  // 4 bytes for refinement AT
            }
            
            // NOW read symbol counts (BIG-ENDIAN!)
            let number_of_exported_symbols = read_u32(data, offset);
            let number_of_new_symbols = read_u32(data, offset + 4);
            


            // Sanity check to catch parsing errors early
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
                start: offset + 8,  // Data starts after both counts
                end,
                at_pixels: Vec::new(),
                refinement_at_pixels: Vec::new(),
            };
            visitor.on_symbol_dictionary(&params)?;
        }
        6 | 7 => {
            // ImmediateTextRegion / ImmediateLosslessTextRegion
            let referred_size = if header.number <= 256 {
                1
            } else if header.number <= 65536 {
                2
            } else {
                4
            };
            
            let params = parse_text_region_params(data, start, referred_size, header.referred_to.len());
            let pos = start + 2 + 4 + header.referred_to.len() * referred_size + REGION_SEGMENT_INFORMATION_FIELD_LENGTH;
            
            visitor.on_immediate_text_region(
                &params.region_info,
                params.text_region_segment_flags,
                params.number_of_symbol_instances,
                &header.referred_to,
                data,
                pos,
                end,
            )?;
        }
        48 => {
            // PageInformation  

            // Per JBIG2 spec and reference implementation: ALL fields use BIG-ENDIAN
            let mut width = read_u32(data, start);
            let mut height = read_u32(data, start + 4);
            let resolution_x = read_u32(data, start + 8);
            let resolution_y = read_u32(data, start + 12);
            let page_segment_flags = data[start + 16];

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
            };

            visitor.on_page_information(info);
        }
        16 => {
            // PatternDictionary
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
            // ImmediateHalftoneRegion / ImmediateLosslessHalftoneRegion
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
            // ImmediateGenericRegion (lossless)
            use super::parser::read_generic_region;
            let generic_region = read_generic_region(data, start)?;
            visitor.on_immediate_generic_region(&generic_region, data, start, end)?;
        }
        42 | 43 => {
            // ImmediateGenericRefinementRegion / ImmediateLosslessGenericRefinementRegion
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
            // IntermediateTextRegion
            let params = parse_text_region_params(data, start, 0, 0);
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
            // IntermediateHalftoneRegion
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
            // IntermediateGenericRegion
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
            // IntermediateGenericRefinementRegion
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
        49 => { // EndOfPage
            // No action needed
        }
        50 => {
            // EndOfStripe
            let height = read_u32(data, start) as usize;
            visitor.on_end_of_stripe(height);
        }
        51 => { // EndOfFile
            // No action needed
        }
        52 => { // Profiles
            // Profile information - can be ignored for basic decoding
        }
        53 => {
            // Tables
            visitor.on_tables(header.number, data, start, end)?;
        }
        62 => {
            // Extension segment (ITU T.88 section 7.4.14)
            // Extension segments are used for vendor-specific features
            // and can be safely ignored for standard decoding
        }
        _ => {} // Unknown segment types
    }
    Ok(())
}
