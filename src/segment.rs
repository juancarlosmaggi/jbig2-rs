use crate::error::Jbig2Error;
use crate::bitmap::Bitmap;

// Segment types from the JS
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

#[derive(Debug)]
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

pub struct Segment {
    pub header: SegmentHeader,
    pub data: Vec<u8>,
    pub start: usize,
    pub end: usize,
}

pub struct SimpleSegmentVisitor {
    pub current_page_info: Option<PageInfo>,
    pub buffer: Option<Vec<u8>>,
    pub symbols: std::collections::HashMap<u32, Vec<Bitmap>>,
    pub patterns: std::collections::HashMap<u32, Vec<Bitmap>>,
    pub custom_tables: std::collections::HashMap<u32, crate::huffman::HuffmanTable>,
}

#[derive(Debug, Clone)]
pub struct PageInfo {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub resolution_x: u32,
    pub resolution_y: u32,
    pub lossless: bool,
    pub refinement: bool,
    pub default_pixel_value: u8,
    pub combination_operator: u8,
    pub requires_buffer: bool,
    pub combination_operator_override: bool,
}

impl Default for SimpleSegmentVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl SimpleSegmentVisitor {
    pub fn new() -> Self {
        SimpleSegmentVisitor {
            current_page_info: None,
            buffer: None,
            symbols: std::collections::HashMap::new(),
            patterns: std::collections::HashMap::new(),
            custom_tables: std::collections::HashMap::new(),
        }
    }

    pub fn on_page_information(&mut self, info: PageInfo) {
        self.current_page_info = Some(info.clone());
        if let Some(height) = info.height {
            let row_size = ((info.width.unwrap_or(0) + 7) >> 3) as usize;
            let buffer = vec![if info.default_pixel_value != 0 { 0xff } else { 0 }; row_size * height as usize];
            self.buffer = Some(buffer);
        }
    }

    // Placeholder for other on_* methods
    pub fn on_immediate_generic_region(&mut self, _region: GenericRegion, _data: &[u8], _start: usize, _end: usize) {
        // TODO: implement
    }
}

#[derive(Debug)]
pub struct GenericRegion {
    pub info: RegionInfo,
    pub mmr: bool,
    pub template: usize,
    pub prediction: bool,
    pub at: Vec<(i8, i8)>,
}

#[derive(Debug)]
pub struct RegionInfo {
    pub width: u32,
    pub height: u32,
    pub x: u32,
    pub y: u32,
    pub combination_operator: u8,
}

fn read_u32(data: &[u8], pos: usize) -> u32 {
    ((data[pos] as u32) << 24) | ((data[pos + 1] as u32) << 16) | ((data[pos + 2] as u32) << 8) | (data[pos + 3] as u32)
}

pub fn read_segment_header(data: &[u8], start: usize) -> Result<SegmentHeader, Jbig2Error> {
    let mut pos = start;
    let number = read_u32(data, pos);
    pos += 4;
    let flags = data[pos];
    pos += 1;
    let segment_type = flags & 0x3f;
    let type_name = SEGMENT_TYPES[segment_type as usize].to_string();
    let deferred_non_retain = (flags & 0x80) != 0;
    let page_association_field_size = (flags & 0x40) != 0;
    let referred_flags = data[pos];
    pos += 1;
    let mut referred_to_count = ((referred_flags >> 5) & 7) as usize;
    let mut retain_bits = vec![referred_flags & 31];
    if referred_flags == 7 {
        referred_to_count = (read_u32(data, pos - 1) & 0x1fffffff) as usize;
        pos += 3;
        let bytes = (referred_to_count + 7) >> 3;
        retain_bits = data[pos..pos + bytes].to_vec();
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
        segment_type: segment_type as usize,
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

const REGION_SEGMENT_INFORMATION_FIELD_LENGTH: usize = 17;

fn read_generic_region(data: &[u8], start: usize) -> Result<GenericRegion, Jbig2Error> {
    let info = read_region_segment_information(data, start);
    let generic_region_segment_flags = data[start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH];
    let mmr = (generic_region_segment_flags & 1) != 0;
    let template = ((generic_region_segment_flags >> 1) & 3) as usize;
    let prediction = (generic_region_segment_flags & 8) != 0;
    let at_length = if template == 0 { 4 } else { 1 };
    let mut at = vec![];
    let mut pos = start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 1;
    for _ in 0..at_length {
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

pub fn process_segment(segment: &Segment, visitor: &mut SimpleSegmentVisitor) -> Result<(), Jbig2Error> {
    let header = &segment.header;
    let data = &segment.data;
    let start = segment.start;
    let end = segment.end;
    match header.segment_type {
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
                width: Some(width),
                height: Some(height),
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
        38 | 39 => { // ImmediateGenericRegion
            let generic_region = read_generic_region(data, start)?;
            let decoding_start = start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 1 + generic_region.at.len() * 2;
            visitor.on_immediate_generic_region(generic_region, data, decoding_start, end);
        }
        _ => {}
    }
    Ok(())
}