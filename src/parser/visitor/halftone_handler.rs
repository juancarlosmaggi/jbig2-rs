use crate::arithmetic::contexts::DecodingContext;
use crate::bitmap::Bitmap;
use crate::common::error::Jbig2Error;
use crate::decoders::halftone::{ShiftedPattern, decode_halftone_region_with_shifted};
use crate::parser::segment::{HalftoneRegionParams, RegionInfo};
use std::collections::HashMap;
use std::sync::Arc;

use super::region_handlers::draw_bitmap;
use super::{PageComposeTarget, SegmentSlice};

/// Decode an immediate halftone region and draw it on the page.
pub(super) fn on_immediate_halftone_region(
    page: PageComposeTarget<'_>,
    patterns: &HashMap<u32, Vec<Bitmap>>,
    pattern_shifts: &HashMap<u32, Arc<Vec<ShiftedPattern>>>,
    params: &HalftoneRegionParams,
    referred_to: &[u32],
    bytes: SegmentSlice<'_>,
) -> Result<(), Jbig2Error> {
    let region_info = &params.region_info;
    if region_info.width == 0 || region_info.height == 0 {
        return Ok(());
    }
    // Prevent integer overflow when computing the bitmap buffer size.
    let stride = ((region_info.width - 1) / 8) + 1;
    if region_info.height > (i32::MAX as u32) / stride {
        return Err(Jbig2Error::new("bitmap size causes integer overflow"));
    }

    ensure_page_for_region(page.page_info, page.bitmap, region_info);

    let Some(bitmap) =
        decode_parsed_halftone_region(patterns, pattern_shifts, params, referred_to, bytes)?
    else {
        return Ok(());
    };
    draw_bitmap(page, region_info, &bitmap)
}

/// Decode an intermediate halftone region and store it by segment number.
pub(super) fn on_intermediate_halftone_region(
    patterns: &HashMap<u32, Vec<Bitmap>>,
    pattern_shifts: &HashMap<u32, Arc<Vec<ShiftedPattern>>>,
    bitmaps: &mut HashMap<u32, Bitmap>,
    params: &HalftoneRegionParams,
    referred_to: &[u32],
    bytes: SegmentSlice<'_>,
    segment_number: u32,
) -> Result<(), Jbig2Error> {
    let Some(bitmap) =
        decode_parsed_halftone_region(patterns, pattern_shifts, params, referred_to, bytes)?
    else {
        return Ok(());
    };
    bitmaps.insert(segment_number, bitmap);
    Ok(())
}

fn ensure_page_for_region(
    current_page_info: &mut Option<crate::document::PageInfo>,
    current_bitmap: &mut Option<Bitmap>,
    region_info: &RegionInfo,
) {
    if current_page_info.is_some() {
        return;
    }
    *current_page_info = Some(crate::document::PageInfo {
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
    *current_bitmap = Some(crate::bitmap::utils::create_initialized_bitmap(
        region_info.width.max(1) as usize,
        region_info.height.max(1) as usize,
        0,
    ));
}

fn decode_parsed_halftone_region(
    patterns: &HashMap<u32, Vec<Bitmap>>,
    pattern_shifts: &HashMap<u32, Arc<Vec<ShiftedPattern>>>,
    params: &HalftoneRegionParams,
    referred_to: &[u32],
    bytes: SegmentSlice<'_>,
) -> Result<Option<Bitmap>, Jbig2Error> {
    if referred_to.is_empty() {
        return Ok(None);
    }
    let pattern_segment = referred_to[0];
    let patterns_vec = if let Some(p) = patterns.get(&pattern_segment) {
        p.as_slice()
    } else {
        return Ok(None);
    };
    let shifted_patterns = pattern_shifts.get(&pattern_segment).cloned();

    let slice = bytes.as_slice();
    let mut decoding_context = DecodingContext::new(slice, 0, slice.len());

    let decode_params = crate::decoders::halftone::HalftoneRegionParams {
        mmr: params.mmr,
        patterns: patterns_vec,
        template: params.template,
        region_width: params.region_info.width as usize,
        region_height: params.region_info.height as usize,
        default_pixel_value: params.default_pixel_value,
        enable_skip: params.enable_skip,
        combination_operator: params.combination_operator,
        grid_width: params.grid_width,
        grid_height: params.grid_height,
        grid_offset_x: params.grid_offset_x,
        grid_offset_y: params.grid_offset_y,
        grid_vector_x: params.grid_vector_x,
        grid_vector_y: params.grid_vector_y,
    };

    decode_halftone_region_with_shifted(&decode_params, shifted_patterns, &mut decoding_context)
        .map(Some)
}
