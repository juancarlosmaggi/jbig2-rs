// Segment header parsing and parameter extraction helpers.

use super::segment_params::{HalftoneRegionParams, PatternDictionaryParams, TextRegionParams};
use super::types::*;
use super::utils::*;
use crate::common::error::Jbig2Error;

/// Parse a segment header starting at `start`.
pub fn read_segment_header(
    data: &[u8],
    start: usize,
    has_file_header: bool,
) -> Result<SegmentHeader, Jbig2Error> {
    if data.len().saturating_sub(start) < 11 {
        return Err(
            Jbig2Error::insufficient_data(11, data.len().saturating_sub(start))
                .with_position(start),
        );
    }
    let mut pos = start;
    let number = read_u32(data, pos);
    pos += 4;
    let flags = data[pos];
    pos += 1;
    let segment_type = (flags & 0x3f) as usize;
    if segment_type >= SEGMENT_TYPES.len() {
        return Err(Jbig2Error::invalid_segment("segment type out of range").with_position(start));
    }
    let type_name = SEGMENT_TYPES[segment_type].to_string();
    let deferred_non_retain = (flags & 0x80) != 0;
    let page_association_field_size = (flags & 0x40) != 0;
    let _data_length_field_size = (flags & 0x04) != 0;
    let referred_to_count;
    let retain_bits;
    let rtscarf = data[pos];
    if (rtscarf & 0xE0) == 0xE0 {
        if data.len().saturating_sub(pos) < 4 {
            return Err(Jbig2Error::new(ERR_INSUFFICIENT_DATA));
        }
        let rtscarf_long = read_u32(data, pos);
        referred_to_count = (rtscarf_long & 0x1fffffff) as usize;
        pos += 4;
        let bytes = (referred_to_count + 1) / 8;
        if data.len().saturating_sub(pos) < bytes {
            return Err(Jbig2Error::new(ERR_INSUFFICIENT_DATA));
        }
        retain_bits = data[pos..pos + bytes].to_vec();
        pos += bytes;
    } else {
        referred_to_count = (rtscarf >> 5) as usize;
        retain_bits = vec![rtscarf & 0x1f];
        pos += 1;
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
    // Length field is always four bytes regardless of header size flags.
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

/// Read the fixed-size region information block.
pub fn read_region_segment_information(data: &[u8], start: usize) -> RegionInfo {
    RegionInfo {
        width: read_u32(data, start),
        height: read_u32(data, start + 4),
        x: read_u32(data, start + 8),
        y: read_u32(data, start + 12),
        combination_operator: data[start + 16] & 7,
    }
}

/// Parse a generic region segment header and its AT parameters.
pub fn read_generic_region(data: &[u8], start: usize) -> Result<GenericRegion, Jbig2Error> {
    let info = read_region_segment_information(data, start);
    let generic_region_segment_flags = data[start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH];
    let mmr = (generic_region_segment_flags & 1) != 0;
    let template = ((generic_region_segment_flags >> 1) & 3) as usize;
    let prediction = (generic_region_segment_flags & 8) != 0;
    // MMR-coded generic regions omit adaptive template parameters.
    let at_length = if mmr { 0 } else if template == 0 { 4 } else { 1 };
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

/// Parse halftone region parameters including grid setup.
pub fn parse_halftone_region_params(data: &[u8], start: usize) -> HalftoneRegionParams {
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

/// Parse text region parameters and symbol instance count.
pub fn parse_text_region_params(data: &[u8], start: usize) -> TextRegionParams {
    let region_info = read_region_segment_information(data, start);
    let mut pos = start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH;
    let text_region_segment_flags = read_u16(data, pos);
    pos += 2;

    let huffman = (text_region_segment_flags & 1) != 0;
    let refinement = (text_region_segment_flags & 2) != 0;
    let refinement_template = ((text_region_segment_flags >> 15) & 1) != 0;

    if huffman {
        pos += 2;
    } else if refinement && !refinement_template {
        pos += 4;
    }

    let number_of_symbol_instances = read_u32(data, pos);

    TextRegionParams {
        region_info,
        text_region_segment_flags,
        number_of_symbol_instances,
    }
}

/// Parse pattern dictionary parameters.
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

/// Parse segments from a stream in sequential or random-access mode.
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
        // Sequential mode parses headers and payloads in one pass.
        while pos < end {
            if pos + 11 > end {
                break;
            }

            match read_segment_header(data, pos, has_file_header) {
                Ok(segment_header) => {
                    // Move past the header to the segment data.
                    pos = segment_header.header_end;

                    // EOF segments have no payload.
                    if segment_header.segment_type == 51 {
                        segments.push(Segment {
                            header: segment_header,
                            data,
                            start: pos,
                            end: pos,
                        });
                        break;
                    }

                    // Record the data range for this segment.
                    let segment_start = pos;
                    let segment_end = (pos + segment_header.length).min(end);

                    segments.push(Segment {
                        header: segment_header,
                        data,
                        start: segment_start,
                        end: segment_end,
                    });

                    // Advance to the next segment header.
                    pos = segment_end;
                }
                Err(_) => break,
            }
        }
    } else {
        // Random-access mode parses headers first, then payloads.

        let mut headers = vec![];

        while pos < end {
            if pos + 11 > end {
                break;
            }

            match read_segment_header(data, pos, has_file_header) {
                Ok(segment_header) => {
                    // Move past the header to the directory entry.
                    pos = segment_header.header_end;

                    // Save the header for payload parsing later.
                    let is_eof = segment_header.segment_type == 51;
                    headers.push(segment_header);

                    // EOF indicates the end of the header directory.
                    if is_eof {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        // Parse each payload in the same order as the headers.

        let _data_area_start = pos;

        for header in headers.into_iter() {
            let segment_start = pos;
            let segment_end = pos + header.length;

            // Ensure the payload fits within the available data.
            if segment_end > end {
                return Err(Jbig2Error::new(ERR_OVERRUN));
            }

            segments.push(Segment {
                header,
                data,
                start: segment_start,
                end: segment_end,
            });

            // Advance to the next payload.
            pos = segment_end;
        }
    }

    Ok(segments)
}
