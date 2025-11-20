// Segment processing logic - dispatches segments to visitor callbacks

use crate::error::Jbig2Error;
use crate::visitor::SimpleSegmentVisitor;
use super::types::*;
use super::utils::*;
use super::parser::read_region_segment_information;

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
            
            eprintln!("Symbol Dict Segment {}:", header.number);
            eprintln!("  Flags: 0x{:04x}", dictionary_flags);
            eprintln!("  SDHUFF: {}", sdhuff);
            eprintln!("  SDREFAGG: {}", sdrefagg);
            eprintln!("  SDTEMPLATE: {}", sdtemplate);
            eprintln!("  Offset after flags: {}", start + 2);
            eprintln!("  Offset for counts: {}", offset);
            eprintln!("  SDNUMEXSYMS: {}", number_of_exported_symbols);
            eprintln!("  SDNUMNEWSYMS: {}", number_of_new_symbols);
            eprintln!("  Data start: {}", offset + 8);

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
            let mut pos = start;
            let text_region_segment_flags = read_u16(data, pos);
            pos += 2;
            let number_of_symbol_instances = read_u32_le(data, pos);
            pos += 4;
            let referred_size = if header.number <= 256 {
                1
            } else if header.number <= 65536 {
                2
            } else {
                4
            };
            pos += header.referred_to.len() * referred_size;
            let region_info = read_region_segment_information(data, pos);
            pos += REGION_SEGMENT_INFORMATION_FIELD_LENGTH;
            visitor.on_immediate_text_region(
                &region_info,
                text_region_segment_flags,
                number_of_symbol_instances,
                &header.referred_to,
                data,
                pos,
                end,
            )?;
        }
        48 => {
            // PageInformation  
            println!("Processing PageInformation segment {}", header.number);
            println!("PageInfo segment data starts at offset 0x{:04x} ({})", start, start);
            if end - start >= 19 {
                println!("PageInfo bytes: {:02x?}", &data[start..start+19]);
            }
            // Per JBIG2 spec and reference implementation: ALL fields use BIG-ENDIAN
            let mut width = read_u32(data, start);
            let mut height = read_u32(data, start + 4);
            let resolution_x = read_u32(data, start + 8);
            let resolution_y = read_u32(data, start + 12);
            let page_segment_flags = data[start + 16];
            println!(
                "Page info raw: width={}, height={}, xres={}, yres={} at segment {}",
                width, height, resolution_x, resolution_y, header.number
            );
            println!("Parsed from bytes: width=[{:02x} {:02x} {:02x} {:02x}], height=[{:02x} {:02x} {:02x} {:02x}]",
                data[start], data[start+1], data[start+2], data[start+3],
                data[start+4], data[start+5], data[start+6], data[start+7]);
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
            println!("Calling visitor.on_page_information from segment {}",
                header.number
            );
            visitor.on_page_information(info);
        }
        16 => {
            // PatternDictionary
            let pattern_dictionary_flags = data[start];
            let mmr = (pattern_dictionary_flags & 1) != 0;
            let template = ((pattern_dictionary_flags >> 1) & 3) as usize;
            let pattern_width = data[start + 1] as usize;
            let pattern_height = data[start + 2] as usize;
            let max_pattern_index = read_u32(data, start + 3) as usize;
            visitor.on_pattern_dictionary(
                mmr,
                pattern_width,
                pattern_height,
                max_pattern_index,
                template,
                header.number,
                data,
                start + 7,
                end,
            )?;
        }
        22 | 23 => {
            // ImmediateHalftoneRegion / ImmediateLosslessHalftoneRegion
            let region_info = read_region_segment_information(data, start);
            let halftone_region_flags = data[start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH];
            let mmr = (halftone_region_flags & 1) != 0;
            let template = ((halftone_region_flags >> 1) & 3) as usize;
            let enable_skip = (halftone_region_flags & 8) != 0;
            let combination_operator = ((halftone_region_flags >> 4) & 7) as usize;
            let default_pixel_value = (halftone_region_flags >> 7) & 1;
            let grid_width =
                read_u32(data, start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 1) as usize;
            let grid_height =
                read_u32(data, start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 5) as usize;
            let grid_offset_x =
                read_u32(data, start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 9) as i32;
            let grid_offset_y =
                read_u32(data, start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 13) as i32;
            let grid_vector_x =
                read_u16(data, start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 17) as i16;
            let grid_vector_y =
                read_u16(data, start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 19) as i16;
            visitor.on_immediate_halftone_region(
                &region_info,
                mmr,
                template,
                enable_skip,
                combination_operator,
                default_pixel_value,
                grid_width,
                grid_height,
                grid_offset_x,
                grid_offset_y,
                grid_vector_x,
                grid_vector_y,
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
            let region_info = read_region_segment_information(data, start);
            let text_region_segment_flags =
                read_u16(data, start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH);
            let number_of_symbol_instances =
                read_u32(data, start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 2);
            visitor.on_intermediate_text_region(
                &region_info,
                text_region_segment_flags,
                number_of_symbol_instances,
                &header.referred_to,
                data,
                start,
                end,
                header.number,
            )?;
        }
        20 => {
            // IntermediateHalftoneRegion
            let region_info = read_region_segment_information(data, start);
            let halftone_region_flags = data[start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH];
            let mmr = (halftone_region_flags & 1) != 0;
            let template = ((halftone_region_flags >> 1) & 3) as usize;
            let enable_skip = (halftone_region_flags & 8) != 0;
            let combination_operator = ((halftone_region_flags >> 4) & 7) as usize;
            let default_pixel_value = (halftone_region_flags >> 7) & 1;
            let grid_width =
                read_u32(data, start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 1) as usize;
            let grid_height =
                read_u32(data, start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 5) as usize;
            let grid_offset_x =
                read_u32(data, start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 9) as i32;
            let grid_offset_y =
                read_u32(data, start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 13) as i32;
            let grid_vector_x =
                read_u16(data, start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 17) as i16;
            let grid_vector_y =
                read_u16(data, start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 19) as i16;
            visitor.on_intermediate_halftone_region(
                &region_info,
                mmr,
                template,
                enable_skip,
                combination_operator,
                default_pixel_value,
                grid_width,
                grid_height,
                grid_offset_x,
                grid_offset_y,
                grid_vector_x,
                grid_vector_y,
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
            // Extension segment - ignore completely (as jbig2dec does)
            println!("Extension segment: ignoring as comment/metadata");
            // Don't process anything from extension segments
        }
        _ => {} // Unknown segment types
    }
    Ok(())
}
