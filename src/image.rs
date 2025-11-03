use crate::error::Jbig2Error;
use crate::bitmap::{Bitmap, decode_bitmap, DecodingContext};
const SEGMENT_TYPES: [&str; 63] = [
    "SymbolDictionary", "", "", "", "IntermediateTextRegion", "", "ImmediateTextRegion",
    "ImmediateLosslessTextRegion", "", "", "", "", "", "", "", "", "PatternDictionary",
    "", "", "", "IntermediateHalftoneRegion", "", "ImmediateHalftoneRegion",
    "ImmediateLosslessHalftoneRegion", "", "", "", "", "", "", "", "", "", "", "", "",
    "IntermediateGenericRegion", "", "ImmediateGenericRegion", "ImmediateLosslessGenericRegion",
    "IntermediateGenericRefinementRegion", "", "ImmediateGenericRefinementRegion",
    "ImmediateLosslessGenericRefinementRegion", "", "", "", "", "PageInformation",
    "EndOfPage", "EndOfStripe", "EndOfFile", "Profiles", "Tables", "", "", "", "", "", "",
    "", "", "Extension",
];
fn read_u32(data: &[u8], pos: usize) -> u32 {
    ((data[pos] as u32) << 24) | ((data[pos + 1] as u32) << 16) | ((data[pos + 2] as u32) << 8) | (data[pos + 3] as u32)
}
fn read_u16(data: &[u8], pos: usize) -> u16 {
    ((data[pos] as u16) << 8) | (data[pos + 1] as u16)
}
#[derive(Clone)]
#[allow(dead_code)]
struct SegmentHeader {
    number: u32,
    segment_type: u8,
    type_name: String,
    deferred_non_retain: bool,
    retain_bits: Vec<u8>,
    referred_to: Vec<u32>,
    page_association: u32,
    length: u32,
    header_end: usize,
}
fn read_segment_header(data: &[u8], start: usize) -> Result<SegmentHeader, Jbig2Error> {
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
            2 => read_u16(data, pos) as u32,
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
    let length = read_u32(data, pos);
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
struct Segment {
    header: SegmentHeader,
    data: Vec<u8>,
    start: usize,
    end: usize,
}
fn read_segments(_header: &FileHeader, data: &[u8], start: usize, end: usize) -> Result<Vec<Segment>, Jbig2Error> {
    let mut segments = vec![];
    let mut position = start;
    while position < end {
        let segment_header = read_segment_header(data, position)?;
        position = segment_header.header_end;
        let segment_start = position;
        position += segment_header.length as usize;
        let segment_end = position;
        segments.push(Segment {
            header: segment_header,
            data: data.to_vec(),
            start: segment_start,
            end: segment_end,
        });
        if segments.last().unwrap().header.segment_type == 51 {
            break;
        }
    }
    Ok(segments)
}
#[allow(dead_code)]
struct FileHeader {
    random_access: bool,
    number_of_pages: Option<u32>,
}
#[derive(Clone)]
#[allow(dead_code)]
struct PageInfo {
    width: usize,
    height: usize,
    resolution_x: usize,
    resolution_y: usize,
    lossless: bool,
    refinement: bool,
    default_pixel_value: u8,
    combination_operator: u8,
    requires_buffer: bool,
    combination_operator_override: bool,
}
#[derive(Clone)]
struct RegionInfo {
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    combination_operator: u8,
}
const REGION_SEGMENT_INFORMATION_FIELD_LENGTH: usize = 17;
fn read_region_segment_information(data: &[u8], start: usize) -> RegionInfo {
    RegionInfo {
        width: read_u32(data, start) as usize,
        height: read_u32(data, start + 4) as usize,
        x: read_u32(data, start + 8) as usize,
        y: read_u32(data, start + 12) as usize,
        combination_operator: data[start + 16] & 7,
    }
}
#[derive(Clone)]
struct GenericRegion {
    info: RegionInfo,
    mmr: bool,
    template: usize,
    prediction: bool,
    at: Vec<(i8, i8)>,
}
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
struct SimpleSegmentVisitor {
    current_page_info: Option<PageInfo>,
    buffer: Option<Vec<u8>>,
}
impl SimpleSegmentVisitor {
    fn new() -> Self {
        SimpleSegmentVisitor {
            current_page_info: None,
            buffer: None,
        }
    }
    fn on_page_information(&mut self, info: PageInfo) {
        self.current_page_info = Some(info.clone());
        let row_size = (info.width + 7) >> 3;
        let buffer_size = row_size * info.height;
        let mut buffer = vec![0u8; buffer_size];
        if info.default_pixel_value != 0 {
            buffer.fill(0xff);
        }
        self.buffer = Some(buffer);
    }
    fn draw_bitmap(&mut self, region_info: &RegionInfo, bitmap: &Bitmap) {
        let page_info = self.current_page_info.as_ref().unwrap();
        let width = region_info.width;
        let height = region_info.height;
        let row_size = (page_info.width + 7) >> 3;
        let combination_operator = if page_info.combination_operator_override {
            region_info.combination_operator
        } else {
            page_info.combination_operator
        };
        let buffer = self.buffer.as_mut().unwrap();
        let mask0 = 128 >> (region_info.x & 7);
        let mut offset0 = region_info.y * row_size + (region_info.x >> 3);
        for i in 0..height {
            let mut mask = mask0;
            let mut offset = offset0;
            for _j in 0..width {
                let pixel = bitmap.get_pixel(offset, i);
                match combination_operator {
                    0 => { // OR
                        if pixel != 0 {
                            buffer[offset] |= mask;
                        }
                    }
                    2 => { // XOR
                        if pixel != 0 {
                            buffer[offset] ^= mask;
                        }
                    }
                    _ => {}
                }
                mask >>= 1;
                if mask == 0 {
                    mask = 128;
                    offset += 1;
                }
            }
            offset0 += row_size;
        }
    }
    fn on_immediate_generic_region(&mut self, region: &GenericRegion, data: &[u8], start: usize, end: usize) -> Result<(), Jbig2Error> {
        let region_info = &region.info;
        let mut decoding_context = DecodingContext::new(data.to_vec(), start, end);
        let bitmap = decode_bitmap(
            region.mmr,
            region_info.width,
            region_info.height,
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
fn process_segment(segment: &Segment, visitor: &mut SimpleSegmentVisitor) -> Result<(), Jbig2Error> {
    let header = &segment.header;
    let data = &segment.data;
    let start = segment.start;
    let end = segment.end;
    match header.segment_type {
        48 => { // PageInformation
            let width = read_u32(data, start) as usize;
            let height = read_u32(data, start + 4) as usize;
            let resolution_x = read_u32(data, start + 8) as usize;
            let resolution_y = read_u32(data, start + 12) as usize;
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
        38 | 39 => { // ImmediateGenericRegion
            let generic_region = read_generic_region(data, start)?;
            let decoding_start = start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 1 + generic_region.at.len() * 2;
            visitor.on_immediate_generic_region(&generic_region, data, decoding_start, end)?;
        }
        _ => {}
    }
    Ok(())
}
fn process_segments(segments: &[Segment], visitor: &mut SimpleSegmentVisitor) -> Result<(), Jbig2Error> {
    for segment in segments {
        process_segment(segment, visitor)?;
    }
    Ok(())
}
pub struct Jbig2Image {
    pub width: usize,
    pub height: usize,
}
impl Default for Jbig2Image {
    fn default() -> Self {
        Self::new()
    }
}
impl Jbig2Image {
    pub fn new() -> Self {
        Jbig2Image {
            width: 0,
            height: 0,
        }
    }
    pub fn parse_chunks(&mut self, _chunks: Vec<Jbig2Chunk>) -> Result<(), Jbig2Error> {
        // TODO: implement parseJbig2Chunks
        Err(Jbig2Error::new("parse_chunks not implemented"))
    }
    pub fn parse(&mut self, data: &[u8]) -> Result<Vec<u8>, Jbig2Error> {
        if data.len() < 8 || &data[0..8] != b"\x97\x4a\x42\x32\x0d\x0a\x1a\x0a" {
            return Err(Jbig2Error::new("invalid header"));
        }
        let mut pos = 8;
        let flags = data[pos];
        pos += 1;
        let random_access = (flags & 1) == 0;
        let number_of_pages = if (flags & 2) == 0 {
            let np = read_u32(data, pos);
            pos += 4;
            Some(np)
        } else {
            None
        };
        let header = FileHeader {
            random_access,
            number_of_pages,
        };
        let segments = read_segments(&header, data, pos, data.len())?;
        let mut visitor = SimpleSegmentVisitor::new();
        process_segments(&segments, &mut visitor)?;
        let buffer = visitor.buffer.ok_or(Jbig2Error::new("no buffer"))?;
        let page_info = visitor.current_page_info.ok_or(Jbig2Error::new("no page info"))?;
        let width = page_info.width;
        let height = page_info.height;
        let row_size = (width + 7) >> 3;
        let mut img_data = vec![0u8; width * height];
        let mut q = 0;
        for i in 0..height {
            let mut mask = 128u8;
            let mut buffer_pos = i * row_size;
            for _j in 0..width {
                if buffer[buffer_pos] & mask != 0 {
                    img_data[q] = 255;
                }
                mask >>= 1;
                if mask == 0 {
                    mask = 128;
                    buffer_pos += 1;
                }
                q += 1;
            }
        }
        self.width = width;
        self.height = height;
        Ok(img_data)
    }
}
pub struct Jbig2Chunk {
    pub data: Vec<u8>,
    pub start: usize,
    pub end: usize,
}