use crate::error::Jbig2Error;
use crate::visitor::SimpleSegmentVisitor;
pub const ERR_INSUFFICIENT_DATA: &str = "insufficient data";
pub const ERR_INVALID_SEGMENT: &str = "invalid segment";
pub const ERR_OVERRUN: &str = "segment overruns data";
pub const ERR_MISMATCH: &str = "data mismatch";
pub const ERR_UNKNOWN_LENGTH: &str = "invalid unknown segment length";
pub const SEGMENT_TYPES: [&str; 63] = [
    "SymbolDictionary",
    "",
    "",
    "",
    "IntermediateTextRegion",
    "",
    "ImmediateTextRegion",
    "ImmediateLosslessTextRegion",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "PatternDictionary",
    "",
    "",
    "",
    "IntermediateHalftoneRegion",
    "",
    "ImmediateHalftoneRegion",
    "ImmediateLosslessHalftoneRegion",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "IntermediateGenericRegion",
    "",
    "ImmediateGenericRegion",
    "ImmediateLosslessGenericRegion",
    "IntermediateGenericRefinementRegion",
    "",
    "ImmediateGenericRefinementRegion",
    "ImmediateLosslessGenericRefinementRegion",
    "",
    "",
    "",
    "",
    "PageInformation",
    "EndOfPage",
    "EndOfStripe",
    "EndOfFile",
    "Profiles",
    "Tables",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "Extension",
];
#[derive(Debug, Clone)]
pub struct SegmentHeader {
    pub number: u32,
    pub segment_type: usize,
    pub type_name: String,
    pub deferred_non_retain: bool,
    pub retain_bits: Vec<u8>,
    pub referred_to: Vec<u32>,
    pub page_association: u32,
    pub length: usize,
    pub header_end: usize,
}
#[derive(Clone)]
pub struct Segment<'a> {
    pub header: SegmentHeader,
    pub data: &'a [u8],
    pub start: usize,
    pub end: usize,
}
#[derive(Debug, Clone)]
pub struct PageInfo {
    pub width: u32,
    pub height: u32,
    pub resolution_x: u32,
    pub resolution_y: u32,
    pub lossless: bool,
    pub refinement: bool,
    pub default_pixel_value: u8,
    pub combination_operator: u8,
    pub requires_buffer: bool,
    pub combination_operator_override: bool,
}
#[derive(Debug, Clone)]
pub struct RegionInfo {
    pub width: u32,
    pub height: u32,
    pub x: u32,
    pub y: u32,
    pub combination_operator: u8,
}
#[derive(Debug)]
pub struct GenericRegion {
    pub info: RegionInfo,
    pub mmr: bool,
    pub template: usize,
    pub prediction: bool,
    pub at: Vec<(i8, i8)>,
}
#[derive(Clone)]
pub struct SymbolDictionaryParams<'a> {
    pub dictionary_flags: u16,
    pub number_of_exported_symbols: u32,
    pub number_of_new_symbols: u32,
    pub current_segment: u32,
    pub referred_segments: &'a [u32],
    pub data: &'a [u8],
    pub start: usize,
    pub end: usize,
}
pub fn read_u32(data: &[u8], pos: usize) -> u32 {
    ((data[pos] as u32) << 24)
        | ((data[pos + 1] as u32) << 16)
        | ((data[pos + 2] as u32) << 8)
        | (data[pos + 3] as u32)
}
pub fn read_u32_le(data: &[u8], pos: usize) -> u32 {
    (data[pos] as u32)
        | ((data[pos + 1] as u32) << 8)
        | ((data[pos + 2] as u32) << 16)
        | ((data[pos + 3] as u32) << 24)
}
pub fn read_u16_le(data: &[u8], pos: usize) -> u16 {
    (data[pos] as u16) | ((data[pos + 1] as u16) << 8)
}
pub fn parse_at_parameters(
    data: &[u8],
    mut pos: usize,
    at_length: usize,
) -> Result<Vec<(i8, i8)>, Jbig2Error> {
    let mut at = vec![];
    for _ in 0..at_length {
        if pos + 1 >= data.len() {
            return Err(Jbig2Error::new("insufficient data for AT flags"));
        }
        let x = data[pos] as i8;
        let y = data[pos + 1] as i8;
        at.push((x, y));
        pos += 2;
    }
    Ok(at)
}
pub fn read_u16(data: &[u8], pos: usize) -> u16 {
    ((data[pos] as u16) << 8) | (data[pos + 1] as u16)
}
pub fn read_segment_header(
    data: &[u8],
    start: usize,
    has_file_header: bool,
) -> Result<SegmentHeader, Jbig2Error> {
    if data.len().saturating_sub(start) < 11 {
        return Err(Jbig2Error::new(ERR_INSUFFICIENT_DATA));
    }
    let mut pos = start;
    let number = read_u32(data, pos);
    pos += 4;
    let flags = data[pos];
    pos += 1;
    let segment_type = (flags & 0x3f) as usize;
    if segment_type >= SEGMENT_TYPES.len() {
        return Err(Jbig2Error::new(ERR_INVALID_SEGMENT));
    }
    let type_name = SEGMENT_TYPES[segment_type].to_string();
    let deferred_non_retain = (flags & 0x80) != 0;
    let page_association_field_size = (flags & 0x40) != 0;
    let data_length_field_size = (flags & 0x04) != 0;
    let mut referred_to_count = (data[pos] & 0x1f) as usize;
    let mut retain_bits = vec![data[pos] >> 5];
    pos += 1;
    if referred_to_count == 31 {
        if data.len().saturating_sub(pos) < 3 {
            return Err(Jbig2Error::new(ERR_INSUFFICIENT_DATA));
        }
        referred_to_count = read_u32(data, pos) as usize & 0x1fffffff;
        pos += 3;
        let bytes = (referred_to_count + 7) >> 3;
        if data.len().saturating_sub(pos) < bytes {
            return Err(Jbig2Error::new(ERR_INSUFFICIENT_DATA));
        }
        retain_bits = data[pos..pos + bytes].to_vec();
        pos += bytes;
    }
    let referred_to_segment_number_size = if number <= 256 {
        1
    } else if number <= 65536 {
        2
    } else {
        4
    };
    let mut referred_to = vec![];
    for _ in 0..referred_to_count {
        if data.len().saturating_sub(pos) < referred_to_segment_number_size {
            return Err(Jbig2Error::new(ERR_INSUFFICIENT_DATA));
        }
        let num = match referred_to_segment_number_size {
            1 => data[pos] as u32,
            2 => ((data[pos] as u32) << 8) | (data[pos + 1] as u32),
            4 => read_u32(data, pos),
            _ => return Err(Jbig2Error::new("invalid referred segment number size")),
        };
        referred_to.push(num);
        pos += referred_to_segment_number_size;
    }
    let pa = if page_association_field_size {
        if data.len().saturating_sub(pos) < 4 {
            return Err(Jbig2Error::new(ERR_INSUFFICIENT_DATA));
        }
        let pa = read_u32(data, pos);
        pos += 4;
        pa
    } else {
        if data.len().saturating_sub(pos) < 1 {
            return Err(Jbig2Error::new(ERR_INSUFFICIENT_DATA));
        }
        let pa = data[pos] as u32;
        pos += 1;
        pa
    };
    if data.len().saturating_sub(pos) < 4 {
        return Err(Jbig2Error::new(ERR_INSUFFICIENT_DATA));
    }
    // Length field is ALWAYS 4 bytes
    let length_u32 = read_u32(data, pos);
    pos += 4;
    let mut length = length_u32 as usize;
    if length_u32 == 0xffffffff {
        if segment_type == 38 || segment_type == 39 {
            if data.len().saturating_sub(pos) < REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 1 {
                return Err(Jbig2Error::new(ERR_INSUFFICIENT_DATA));
            }
            let region_info = read_region_segment_information(data, pos);
            let generic_flags = data[pos + REGION_SEGMENT_INFORMATION_FIELD_LENGTH];
            let mmr = (generic_flags & 1) != 0;
            if mmr && !has_file_header {
                length = data.len() - pos;
            } else {
                let height = region_info.height;
                let mut search_pattern: Vec<u8> = if mmr { vec![] } else { vec![0xff, 0xac] };
                search_pattern.push(((height >> 24) & 0xff) as u8);
                search_pattern.push(((height >> 16) & 0xff) as u8);
                search_pattern.push(((height >> 8) & 0xff) as u8);
                search_pattern.push((height & 0xff) as u8);
                let search_pattern_length = search_pattern.len();
                let mut found = false;
                for i in pos..data.len().saturating_sub(search_pattern_length) + 1 {
                    if data[i..i + search_pattern_length] == search_pattern[..] {
                        length = i + search_pattern_length - pos;
                        found = true;
                        break;
                    }
                }
                if !found {
                    return Err(Jbig2Error::new(ERR_UNKNOWN_LENGTH));
                }
            }
        } else {
            return Err(Jbig2Error::new(ERR_UNKNOWN_LENGTH));
        }
    }
    println!("Segment header at {}: number={}, type={}, flags={:02x}, page_assoc={}, length={}, header_end={}", 
        start, number, segment_type, flags, pa, length, pos);
    if segment_type == 48 || segment_type == 62 {
        println!("  -> Segment type {} at 0x{:04x}, data will start at 0x{:04x}", segment_type, start, pos);
    }
    Ok(SegmentHeader {
        number,
        segment_type,
        type_name,
        deferred_non_retain,
        retain_bits,
        referred_to,
        page_association: pa,
        length,
        header_end: pos,
    })
}
fn read_region_segment_information(data: &[u8], start: usize) -> RegionInfo {
    RegionInfo {
        width: read_u32(data, start),
        height: read_u32(data, start + 4),
        x: read_u32(data, start + 8),
        y: read_u32(data, start + 12),
        combination_operator: data[start + 16] & 7,
    }
}
pub fn read_generic_region(data: &[u8], start: usize) -> Result<GenericRegion, Jbig2Error> {
    let info = read_region_segment_information(data, start);
    let generic_region_segment_flags = data[start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH];
    let mmr = (generic_region_segment_flags & 1) != 0;
    let template = ((generic_region_segment_flags >> 1) & 3) as usize;
    let prediction = (generic_region_segment_flags & 8) != 0;
    let at_length = if template == 0 { 4 } else { 1 };
    let pos = start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 1;
    let at = parse_at_parameters(data, pos, at_length)?;
    Ok(GenericRegion {
        info,
        mmr,
        template,
        prediction,
        at,
    })
}
pub fn read_segments<'a>(
    data: &'a [u8],
    start: usize,
    end: usize,
    sequential: bool,
    _data_start: usize,
    has_file_header: bool,
) -> Result<Vec<Segment<'a>>, Jbig2Error> {
    let mut segments = vec![];
    let mut headers = vec![];
    let mut pos = start;
    println!("read_segments: start={}, end={}, data[start..start+20]={:02x?}", start, end, &data[start..(start+20).min(end)]);
    while pos < end {
        if pos + 11 > end {
            break;
        }
        let header_start = pos;
        match read_segment_header(data, pos, has_file_header) {
            Ok(segment_header) => {
                if segment_header.segment_type == 51 {
                    segments.push(Segment {
                        header: segment_header.clone(),
                        data,
                        start: pos,
                        end: segment_header.header_end,
                    });
                    break;
                }
                // Extension segments: trust the length field
                if segment_header.segment_type == 62 {
                    if sequential {
                        let ext_data_start = segment_header.header_end;
                        let ext_data_end = ext_data_start + segment_header.length;
                        
                        // Add extension segment
                        segments.push(Segment {
                            header: segment_header.clone(),
                            data,
                            start: ext_data_start,
                            end: ext_data_end,
                        });
                        
                        // Move position to after this segment's data
                        pos = ext_data_end;
                        continue;
                    } else {
                        headers.push(segment_header.clone());
                        pos = segment_header.header_end;
                    }
                } else {
                    headers.push(segment_header.clone());
                    pos = segment_header.header_end;
                }
                if sequential {
                    let segment_start = segment_header.header_end;
                    let segment_end = (segment_start + segment_header.length).min(end);
                    let next_pos = segment_end;
                    segments.push(Segment {
                        header: segment_header,
                        data,
                        start: segment_start,
                        end: segment_end,
                    });
                    pos = next_pos;
                }
            }
            Err(_) => break,
        }
    }
    if !sequential {
        let mut cumulative_length = 0;
        for header in &headers {
            cumulative_length += header.length;
        }
        if pos + cumulative_length > end {
            return Err(Jbig2Error::new(ERR_OVERRUN));
        }
        let mut data_pos = pos;
        for header in headers {
            let segment_start = data_pos;
            data_pos += header.length;
            let segment_end = data_pos;
            segments.push(Segment {
                header,
                data,
                start: segment_start,
                end: segment_end,
            });
        }
    }
    Ok(segments)
}
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
            let number_of_exported_symbols = read_u32_le(data, start + 2);
            let number_of_new_symbols = read_u32_le(data, start + 6);
            let params = SymbolDictionaryParams {
                dictionary_flags,
                number_of_exported_symbols,
                number_of_new_symbols,
                current_segment: header.number,
                referred_segments: &header.referred_to,
                data,
                start: start + 10,
                end,
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
const REGION_SEGMENT_INFORMATION_FIELD_LENGTH: usize = 17;
