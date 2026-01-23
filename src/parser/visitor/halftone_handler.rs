use crate::bitmap::Bitmap;
use crate::arithmetic::contexts::DecodingContext;
use crate::decoders::halftone::{ShiftedPattern, decode_halftone_region_with_shifted};
use crate::common::error::Jbig2Error;
use crate::document::PageInfo;
use crate::parser::segment::RegionInfo;
use std::collections::HashMap;
use std::sync::Arc;

use super::region_handlers::draw_bitmap;

/// Decode an immediate halftone region and draw it on the page.
#[allow(clippy::too_many_arguments)]
pub(super) fn on_immediate_halftone_region(
    current_page_info: &mut Option<PageInfo>,
    current_bitmap: &mut Option<Bitmap>,
    current_y: usize,
    patterns: &HashMap<u32, Vec<Bitmap>>,
    pattern_shifts: &HashMap<u32, Arc<Vec<ShiftedPattern>>>,
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
    if region_info.width == 0 || region_info.height == 0 {
        return Ok(());
    }
    // Prevent integer overflow when computing the bitmap buffer size.
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
            striped: false,
            stripe_size: 0,
            height_unknown: false,
        });
        let width = region_info.width.max(1) as usize;
        let height = region_info.height.max(1) as usize;
        *current_bitmap = Some(crate::bitmap::utils::create_initialized_bitmap(
            width, height, 0,
        ));
    }

    // Resolve the pattern dictionary referenced by this segment.
    if referred_to.is_empty() {
        return Ok(());
    }
    let pattern_segment = referred_to[0];
    let patterns_vec = if let Some(p) = patterns.get(&pattern_segment) {
        p.as_slice()
    } else {
        return Ok(());
    };
    let shifted_patterns = pattern_shifts.get(&pattern_segment).cloned();

    let slice = &data[start..end];
    let mut decoding_context = DecodingContext::new(slice, 0, slice.len());

    let params = crate::decoders::halftone::HalftoneRegionParams {
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

    let bitmap = decode_halftone_region_with_shifted(
        &params,
        shifted_patterns,
        &mut decoding_context,
    )?;

    draw_bitmap(
        current_page_info,
        current_bitmap,
        current_y,
        region_info,
        &bitmap,
    )?;
    Ok(())
}

/// Decode an intermediate halftone region and store it by segment number.
#[allow(clippy::too_many_arguments)]
pub(super) fn on_intermediate_halftone_region(
    patterns: &HashMap<u32, Vec<Bitmap>>,
    pattern_shifts: &HashMap<u32, Arc<Vec<ShiftedPattern>>>,
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
    // Resolve the pattern dictionary referenced by this segment.
    if referred_to.is_empty() {
        return Ok(());
    }
    let pattern_segment = referred_to[0];
    let patterns_vec = if let Some(p) = patterns.get(&pattern_segment) {
        p.as_slice()
    } else {
        return Ok(());
    };
    let shifted_patterns = pattern_shifts.get(&pattern_segment).cloned();

    let slice = &data[start..end];
    let mut decoding_context = DecodingContext::new(slice, 0, slice.len());

    let params = crate::decoders::halftone::HalftoneRegionParams {
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

    let bitmap = decode_halftone_region_with_shifted(
        &params,
        shifted_patterns,
        &mut decoding_context,
    )?;

    bitmaps.insert(segment_number, bitmap);
    Ok(())
}
