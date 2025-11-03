use crate::error::Jbig2Error;
use crate::visitor::SimpleSegmentVisitor;

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

#[derive(Debug)]
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

#[derive(Debug, Clone)]
pub struct FileHeader {
    pub random_access: bool,
    pub number_of_pages: Option<u32>,
}

const REGION_SEGMENT_INFORMATION_FIELD_LENGTH: usize = 17;

pub fn read_u32(data: &[u8], pos: usize) -> u32 {
    ((data[pos] as u32) << 24) | ((data[pos + 1] as u32) << 16) | ((data[pos + 2] as u32) << 8) | (data[pos + 3] as u32)
}

pub fn read_u16(data: &[u8], pos: usize) -> u16 {
    ((data[pos] as u16) << 8) | (data[pos + 1] as u16)
}

pub fn read_segment_header(data: &[u8], start: usize) -> Result<SegmentHeader, Jbig2Error> {
    let mut pos = start;
    let number = read_u32(data, pos);
    pos += 4;
    let flags = data[pos];
    pos += 1;
    let segment_type = (flags & 0x3f) as usize;
    let type_name = SEGMENT_TYPES[segment_type].to_string();
    let deferred_non_retain = (flags & 0x80) != 0;
    let page_association_field_size = (flags & 0x40) != 0;
    let referred_flags = data[pos];
    pos += 1;
    let mut referred_to_count = ((referred_flags >> 5) & 7) as usize;
    let mut retain_bits = vec![referred_flags & 31];
    if referred_flags == 7 {
        let extended_count = read_u32(data, pos - 1) & 0x1fffffff;
        referred_to_count = extended_count as usize;
        pos += 3;
        let bytes = (referred_to_count + 7) >> 3;
        retain_bits = data[pos..pos + bytes.min(data.len() - pos)].to_vec();
        pos += bytes;
    } else if referred_flags == 5 || referred_flags == 6 {
        return Err(Jbig2Error::new("invalid referred-to flags"));
    }
    let mut referred_to_segment_number_size = 4;
    if number <= 256 {
        referred_to_segment_number_size = 1;
    } else if number <= 65536 {
        referred_to_segment_number_size = 2;
    }
    let mut referred_to = vec![];
    for _ in 0..referred_to_count {
        let num = match referred_to_segment_number_size {
            1 => data[pos] as u32,
            2 => ((data[pos] as u16) << 8 | data[pos + 1] as u16) as u32,
            _ => read_u32(data, pos),
        };
        referred_to.push(num);
        pos += referred_to_segment_number_size;
    }
    let page_association = if page_association_field_size {
        let pa = read_u32(data, pos);
        pos += 4;
        pa
    } else {
        let pa = data[pos] as u32;
        pos += 1;
        pa
    };
    let length = read_u32(data, pos) as usize;
    pos += 4;
    if length == 0xffffffff {
        return Err(Jbig2Error::new("unknown segment length not supported"));
    }
    Ok(SegmentHeader {
        number,
        segment_type,
        type_name,
        deferred_non_retain,
        retain_bits,
        referred_to,
        page_association,
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
    let mut at = vec![];
    let mut pos = start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 1;
    for _ in 0..at_length {
        if pos + 1 >= data.len() {
            return Err(Jbig2Error::new("insufficient data for AT flags"));
        }
        let x = data[pos] as i8;
        let y = data[pos + 1] as i8;
        at.push((x, y));
        pos += 2;
    }
    Ok(GenericRegion {
        info,
        mmr,
        template,
        prediction,
        at,
    })
}

pub fn read_segments<'a>(data: &'a [u8], start: usize, end: usize) -> Result<Vec<Segment<'a>>, Jbig2Error> {
    let mut segments = vec![];
    let mut position = start;
    while position < end {
        let segment_header = read_segment_header(data, position)?;
        position = segment_header.header_end;
        let segment_start = position;
        position += segment_header.length;
        if position > end {
            return Err(Jbig2Error::new("segment overruns data"));
        }
        let segment_end = position;
        segments.push(Segment {
            header: segment_header,
            data,
            start: segment_start,
            end: segment_end,
        });
        if segments.last().unwrap().header.segment_type == 51 { // EndOfFile
            break;
        }
    }
    Ok(segments)
}

pub fn process_segments<'a>(segments: &[Segment<'a>], visitor: &mut SimpleSegmentVisitor) -> Result<(), Jbig2Error> {
    for segment in segments {
        process_segment(segment, visitor)?;
    }
    Ok(())
}

pub fn process_segment<'a>(segment: &Segment<'a>, visitor: &mut SimpleSegmentVisitor) -> Result<(), Jbig2Error> {
    let header = &segment.header;
    let data = segment.data;
    let start = segment.start;
    let end = segment.end;
    match header.segment_type {
        0 => { // SymbolDictionary
            let dictionary_flags = read_u16(data, start);
            let number_of_exported_symbols = read_u32(data, start + 2);
            let number_of_new_symbols = read_u32(data, start + 6);
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
        6 | 7 => { // ImmediateTextRegion / ImmediateLosslessTextRegion
            let region_info = read_region_segment_information(data, start);
            let text_region_segment_flags = read_u16(data, start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH);
            let number_of_symbol_instances = read_u32(data, start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 2);
            visitor.on_immediate_text_region(
                &region_info,
                text_region_segment_flags,
                number_of_symbol_instances,
                &header.referred_to,
                data,
                start,
                end,
            )?;
        }
        48 => { // PageInformation
            let width = read_u32(data, start);
            let height = read_u32(data, start + 4);
            let resolution_x = read_u32(data, start + 8);
            let resolution_y = read_u32(data, start + 12);
            let page_segment_flags = data[start + 16];
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
        16 => { // PatternDictionary
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
        22 | 23 => { // ImmediateHalftoneRegion / ImmediateLosslessHalftoneRegion
            let region_info = read_region_segment_information(data, start);
            let halftone_region_flags = data[start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH];
            let mmr = (halftone_region_flags & 1) != 0;
            let template = ((halftone_region_flags >> 1) & 3) as usize;
            let enable_skip = (halftone_region_flags & 8) != 0;
            let combination_operator = ((halftone_region_flags >> 4) & 7) as usize;
            let default_pixel_value = (halftone_region_flags >> 7) & 1;
            let grid_width = read_u32(data, start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 1) as usize;
            let grid_height = read_u32(data, start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 5) as usize;
            let grid_offset_x = read_u32(data, start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 9) as i32;
            let grid_offset_y = read_u32(data, start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 13) as i32;
            let grid_vector_x = read_u16(data, start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 17) as i16;
            let grid_vector_y = read_u16(data, start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 19) as i16;
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
        38 | 39 => { // ImmediateGenericRegion (lossless)
            let generic_region = read_generic_region(data, start)?;
            visitor.on_immediate_generic_region(&generic_region, data, start, end)?;
        }
        53 => { // Tables
            visitor.on_tables(header.number, data, start, end)?;
        }
        _ => {} // TODO: Add refinement regions, etc.
    }
    Ok(())
}