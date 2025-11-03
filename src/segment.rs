use crate::error::Jbig2Error;
use crate::bitmap::{Bitmap, decode_bitmap, DecodingContext};
use std::collections::HashMap;

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

pub struct SimpleSegmentVisitor {
    pub current_page_info: Option<PageInfo>,
    pub bitmap: Option<Bitmap>,
    pub symbols: HashMap<u32, Vec<Bitmap>>,
    pub patterns: HashMap<u32, Vec<Bitmap>>,
    pub custom_tables: HashMap<u32, crate::huffman::HuffmanTable>,
}

#[derive(Debug, Clone)]
pub struct FileHeader {
    pub random_access: bool,
    pub number_of_pages: Option<u32>,
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
            bitmap: None,
            symbols: HashMap::new(),
            patterns: HashMap::new(),
            custom_tables: HashMap::new(),
        }
    }

    pub fn on_page_information(&mut self, info: PageInfo) {
        self.current_page_info = Some(info.clone());
        let width = info.width as usize;
        let height = info.height as usize;
        let mut bitmap = Bitmap::new(width, height);
        if info.default_pixel_value != 0 {
            for y in 0..height {
                for x in 0..width {
                    bitmap.set_pixel(x, y, 1);
                }
            }
        }
        self.bitmap = Some(bitmap);
    }

    pub fn draw_bitmap(&mut self, region_info: &RegionInfo, src_bitmap: &Bitmap) {
        let page_info = self.current_page_info.as_ref().unwrap();
        let page_width = page_info.width as usize;
        let page_height = page_info.height as usize;
        let combo_op = if page_info.combination_operator_override {
            region_info.combination_operator
        } else {
            page_info.combination_operator
        };
        let dst = self.bitmap.as_mut().unwrap();
        let reg_x = region_info.x as usize;
        let reg_y = region_info.y as usize;
        let width = region_info.width.min((page_width as u32 - reg_x as u32) as u32) as usize;
        let height = region_info.height.min((page_height as u32 - reg_y as u32) as u32) as usize;
        for i in 0..height {
            for j in 0..width {
                let src = src_bitmap.get_pixel(j, i);
                let dx = reg_x + j;
                let dy = reg_y + i;
                let old_dst = dst.get_pixel(dx, dy);
                let new_val = match combo_op {
                    0 => src, // replace
                    1 => old_dst | src, // OR
                    2 => old_dst & src, // AND
                    3 => old_dst ^ src, // XOR
                    4 => !(old_dst ^ src) & 1, // XNOR (bi-level)
                    _ => old_dst, // undefined: no-op
                };
                dst.set_pixel(dx, dy, new_val);
            }
        }
    }

    pub fn on_immediate_generic_region(&mut self, region: &GenericRegion, data: &[u8], start: usize, end: usize) -> Result<(), Jbig2Error> {
        let region_info = &region.info;
        let at_bytes = region.at.len() * 2;
        let decoding_start = start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 1 + at_bytes;
        if decoding_start >= end {
            return Err(Jbig2Error::new("insufficient data for generic region"));
        }
        let slice = &data[decoding_start..end];
        let mut decoding_context = DecodingContext::new(slice.to_vec(), 0, slice.len());
        let bitmap = decode_bitmap(
            region.mmr,
            region_info.width as usize,
            region_info.height as usize,
            region.template,
            region.prediction,
            None,
            region.at.clone(),
            &mut decoding_context,
        )?;
        self.draw_bitmap(region_info, &bitmap);
        Ok(())
    }
}

const REGION_SEGMENT_INFORMATION_FIELD_LENGTH: usize = 17;

pub fn read_u32(data: &[u8], pos: usize) -> u32 {
    ((data[pos] as u32) << 24) | ((data[pos + 1] as u32) << 16) | ((data[pos + 2] as u32) << 8) | (data[pos + 3] as u32)
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

pub fn read_segments<'a>(_header: &'a FileHeader, data: &'a [u8], start: usize, end: usize) -> Result<Vec<Segment<'a>>, Jbig2Error> {
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

pub fn process_segment<'a>(segment: &Segment<'a>, visitor: &mut SimpleSegmentVisitor) -> Result<(), Jbig2Error> {
    let header = &segment.header;
    let data = segment.data;
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
        38 | 39 => { // ImmediateGenericRegion (lossless)
            let generic_region = read_generic_region(data, start)?;
            visitor.on_immediate_generic_region(&generic_region, data, start, end)?;
        }
        _ => {} // TODO: Add text, halftone, etc.
    }
    Ok(())
}

pub fn process_segments<'a>(segments: &[Segment<'a>], visitor: &mut SimpleSegmentVisitor) -> Result<(), Jbig2Error> {
    for segment in segments {
        process_segment(segment, visitor)?;
    }
    Ok(())
}