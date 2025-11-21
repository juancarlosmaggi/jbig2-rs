use crate::bitmap::Bitmap;
use crate::contexts::DecodingContext;
use crate::decode::decode_halftone::decode_halftone_region;
use crate::error::Jbig2Error;
use crate::segment::{PageInfo, RegionInfo};
use std::collections::HashMap;

use super::region_handlers::draw_bitmap;

/// Handle immediate halftone region
#[allow(clippy::too_many_arguments)]
pub(super) fn on_immediate_halftone_region(
    current_page_info: &mut Option<PageInfo>,
    current_bitmap: &mut Option<Bitmap>,
    current_y: usize,
    patterns: &HashMap<u32, Vec<Bitmap>>,
    region_info: &RegionInfo,
    mmr: bool,
    template: usize,
    enable_skip: bool,
    combination_operator: usize,
    default_pixel_value: u8,
    grid_width: usize,
    grid_height: usize,
    grid_offset_x: i32,
    grid_offset_y: i32,
    grid_vector_x: i16,
    grid_vector_y: i16,
    referred_to: &[u32],
    data: &[u8],
    start: usize,
    end: usize,
) -> Result<(), Jbig2Error> {
    println!(
        "Halftone region: width={}, height={}",
        region_info.width, region_info.height
    );
    if region_info.width == 0 || region_info.height == 0 {
        return Ok(());
    }
    // Check for overflow matching jbig2dec: height > INT32_MAX / stride
    let stride = ((region_info.width - 1) / 8) + 1;
    if region_info.height > (i32::MAX as u32) / stride {
        return Err(Jbig2Error::new("bitmap size causes integer overflow"));
    }

    if current_page_info.is_none() {
        *current_page_info = Some(PageInfo {
            width: region_info.width.max(1),
            height: region_info.height.max(1),
            resolution_x: 300,
            resolution_y: 300,
            lossless: true,
            refinement: false,
            default_pixel_value: 0,
            combination_operator: 0, // OR
            requires_buffer: false,
            combination_operator_override: false,
        });
        let width = region_info.width.max(1) as usize;
        let height = region_info.height.max(1) as usize;
        *current_bitmap = Some(crate::bitmap_utils::create_initialized_bitmap(width, height, 0));
    }

    // Get patterns from referred segment
    if referred_to.is_empty() {
        return Ok(()); // Skip if no referred
    }
    let pattern_segment = referred_to[0];
    let patterns_vec = if let Some(p) = patterns.get(&pattern_segment) {
        p.clone()
    } else {
        return Ok(()); // Skip if not found
    };

    let slice = &data[start..end];
    let mut decoding_context = DecodingContext::new(slice.to_vec(), 0, slice.len());

    let params = crate::decode::decode_halftone::HalftoneRegionParams {
        mmr,
        patterns: patterns_vec,
        template,
        region_width: region_info.width as usize,
        region_height: region_info.height as usize,
        default_pixel_value,
        enable_skip,
        combination_operator,
        grid_width,
        grid_height,
        grid_offset_x,
        grid_offset_y,
        grid_vector_x,
        grid_vector_y,
    };

    let bitmap = decode_halftone_region(&params, &mut decoding_context)?;

    draw_bitmap(current_page_info, current_bitmap, current_y, region_info, &bitmap)?;
    Ok(())
}

/// Handle intermediate halftone region
#[allow(clippy::too_many_arguments)]
pub(super) fn on_intermediate_halftone_region(
    patterns: &HashMap<u32, Vec<Bitmap>>,
    bitmaps: &mut HashMap<u32, Bitmap>,
    region_info: &RegionInfo,
    mmr: bool,
    template: usize,
    enable_skip: bool,
    combination_operator: usize,
    default_pixel_value: u8,
    grid_width: usize,
    grid_height: usize,
    grid_offset_x: i32,
    grid_offset_y: i32,
    grid_vector_x: i16,
    grid_vector_y: i16,
    referred_to: &[u32],
    data: &[u8],
    start: usize,
    end: usize,
    segment_number: u32,
) -> Result<(), Jbig2Error> {
    // Get patterns from referred segment
    if referred_to.is_empty() {
        return Ok(());
    }
    let pattern_segment = referred_to[0];
    let patterns_vec = if let Some(p) = patterns.get(&pattern_segment) {
        p.clone()
    } else {
        return Ok(());
    };

    let slice = &data[start..end];
    let mut decoding_context = DecodingContext::new(slice.to_vec(), 0, slice.len());

    let params = crate::decode::decode_halftone::HalftoneRegionParams {
        mmr,
        patterns: patterns_vec,
        template,
        region_width: region_info.width as usize,
        region_height: region_info.height as usize,
        default_pixel_value,
        enable_skip,
        combination_operator,
        grid_width,
        grid_height,
        grid_offset_x,
        grid_offset_y,
        grid_vector_x,
        grid_vector_y,
    };

    let bitmap = decode_halftone_region(&params, &mut decoding_context)?;

    bitmaps.insert(segment_number, bitmap);
    Ok(())
}
