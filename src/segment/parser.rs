// Segment header and data parsing functions

use crate::error::Jbig2Error;
use super::types::*;
use super::utils::*;

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
    let _data_length_field_size = (flags & 0x04) != 0;
    let mut referred_to_count = (data[pos] >> 5) as usize;
    let mut retain_bits = vec![data[pos] & 0x07];
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

pub fn read_region_segment_information(data: &[u8], start: usize) -> RegionInfo {
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

/// Parameters parsed from halftone region segment
#[derive(Debug)]
pub struct HalftoneRegionParams {
    pub region_info: RegionInfo,
    pub mmr: bool,
    pub template: usize,
    pub enable_skip: bool,
    pub combination_operator: usize,
    pub default_pixel_value: u8,
    pub grid_width: usize,
    pub grid_height: usize,
    pub grid_offset_x: i32,
    pub grid_offset_y: i32,
    pub grid_vector_x: i16,
    pub grid_vector_y: i16,
}

/// Parse halftone region parameters (segments 20, 22, 23)
/// Extracts region info, flags, and grid parameters
pub fn parse_halftone_region_params(data: &[u8], start: usize) -> HalftoneRegionParams {
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
    
    HalftoneRegionParams {
        region_info,
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
    }
}

/// Parameters parsed from text region segment
#[derive(Debug)]
pub struct TextRegionParams {
    pub region_info: RegionInfo,
    pub text_region_segment_flags: u16,
    pub number_of_symbol_instances: u32,
}

/// Parse text region common parameters (segments 4, 6, 7)
/// Returns region info, flags, and symbol instance count
pub fn parse_text_region_params(data: &[u8], start: usize, referred_size: usize, referred_count: usize) -> TextRegionParams {
    let mut pos = start;
    let text_region_segment_flags = read_u16(data, pos);
    pos += 2;
    let number_of_symbol_instances = read_u32_le(data, pos);
    pos += 4;
    pos += referred_count * referred_size;
    let region_info = read_region_segment_information(data, pos);
    
    TextRegionParams {
        region_info,
        text_region_segment_flags,
        number_of_symbol_instances,
    }
}

/// Parameters parsed from pattern dictionary segment
#[derive(Debug)]
pub struct PatternDictionaryParams {
    pub mmr: bool,
    pub template: usize,
    pub pattern_width: usize,
    pub pattern_height: usize,
    pub max_pattern_index: usize,
}

/// Parse pattern dictionary parameters (segment 16)
pub fn parse_pattern_dictionary_params(data: &[u8], start: usize) -> PatternDictionaryParams {
    let pattern_dictionary_flags = data[start];
    let mmr = (pattern_dictionary_flags & 1) != 0;
    let template = ((pattern_dictionary_flags >> 1) & 3) as usize;
    let pattern_width = data[start + 1] as usize;
    let pattern_height = data[start + 2] as usize;
    let max_pattern_index = read_u32(data, start + 3) as usize;
    
    PatternDictionaryParams {
        mmr,
        template,
        pattern_width,
        pattern_height,
        max_pattern_index,
    }
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
    let mut pos = start;
    

    
    if sequential {
        // SEQUENTIAL MODE: Parse header and data together
        while pos < end {
            if pos + 11 > end {
                break;
            }
            
            match read_segment_header(data, pos, has_file_header) {
                Ok(segment_header) => {

                    
                    // Move past header
                    pos = segment_header.header_end;
                    
                    // EOF segment has no data
                    if segment_header.segment_type == 51 {
                        segments.push(Segment {
                            header: segment_header,
                            data,
                            start: pos,
                            end: pos,
                        });
                        break;
                    }
                    
                    // Calculate data range
                    let segment_start = pos;
                    let segment_end = (pos + segment_header.length).min(end);
                    
                    segments.push(Segment {
                        header: segment_header,
                        data,
                        start: segment_start,
                        end: segment_end,
                    });
                    
                    // Move to next segment
                    pos = segment_end;
                }
                Err(_) => break,
            }
        }
    } else {
        // RANDOM-ACCESS MODE: Two-phase parsing
        
        // PHASE 1: Parse ALL segment headers (directory)

        let mut headers = vec![];
        
        while pos < end {
            if pos + 11 > end {
                break;
            }
            
            match read_segment_header(data, pos, has_file_header) {
                Ok(segment_header) => {

                    
                    // Move past header
                    pos = segment_header.header_end;
                    
                    // Store header for phase 2
                    let is_eof = segment_header.segment_type == 51;
                    headers.push(segment_header);
                    
                    // EOF segment marks end of directory
                    if is_eof {

                        break;
                    }
                }
                Err(_) => break,
            }
        }
        

        
        // PHASE 2: Parse ALL segment data (in same order as headers)

        let _data_area_start = pos;
        
        for header in headers.into_iter() {
            let segment_start = pos;
            let segment_end = pos + header.length;
            

            
            // Validate we have enough data
            if segment_end > end {
                return Err(Jbig2Error::new(ERR_OVERRUN));
            }
            
            segments.push(Segment {
                header,
                data,
                start: segment_start,
                end: segment_end,
            });
            
            // Move to next segment's data
            pos = segment_end;
        }
    }
    

    Ok(segments)
}
