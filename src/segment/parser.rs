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
    
    println!("read_segments: mode={}, start=0x{:04x}, end=0x{:04x}", 
        if sequential { "sequential" } else { "random-access" }, start, end);
    
    if sequential {
        // SEQUENTIAL MODE: Parse header and data together
        while pos < end {
            if pos + 11 > end {
                break;
            }
            
            match read_segment_header(data, pos, has_file_header) {
                Ok(segment_header) => {
                    println!("  Segment {}: type={}, header_end=0x{:04x}, length={}", 
                        segment_header.number, segment_header.segment_type, 
                        segment_header.header_end, segment_header.length);
                    
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
        println!("PHASE 1: Parsing segment directory");
        let mut headers = vec![];
        
        while pos < end {
            if pos + 11 > end {
                break;
            }
            
            match read_segment_header(data, pos, has_file_header) {
                Ok(segment_header) => {
                    println!("  Directory entry {}: type={}, header at 0x{:04x}-0x{:04x}, data_length={}", 
                        segment_header.number, segment_header.segment_type, 
                        pos, segment_header.header_end, segment_header.length);
                    
                    // Move past header
                    pos = segment_header.header_end;
                    
                    // Store header for phase 2
                    let is_eof = segment_header.segment_type == 51;
                    headers.push(segment_header);
                    
                    // EOF segment marks end of directory
                    if is_eof {
                        println!("  → End of directory (EOF segment)");
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        
        println!("Directory complete: {} segments, data area starts at 0x{:04x}", 
            headers.len(), pos);
        
        // PHASE 2: Parse ALL segment data (in same order as headers)
        println!("PHASE 2: Parsing segment data");
        let _data_area_start = pos;
        
        for (i, header) in headers.into_iter().enumerate() {
            let segment_start = pos;
            let segment_end = pos + header.length;
            
            println!("  Segment {} data: 0x{:04x}-0x{:04x} ({} bytes)", 
                i, segment_start, segment_end, header.length);
            
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
    
    println!("read_segments: parsed {} segments", segments.len());
    Ok(segments)
}
