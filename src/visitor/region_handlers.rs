use crate::bitmap::Bitmap;
use crate::bitmap_utils;
use crate::contexts::DecodingContext;
use crate::decode::decode_generic::{DecodeBitmapParams, decode_bitmap};
use crate::error::Jbig2Error;
use crate::segment::{GenericRegion, PageInfo, RegionInfo, parse_at_parameters};
use std::collections::HashMap;

pub(super) const REGION_SEGMENT_INFORMATION_FIELD_LENGTH: usize = 17;

/// Composite a decoded region bitmap onto the current page bitmap.
pub(super) fn draw_bitmap(
    current_page_info: &Option<PageInfo>,
    current_bitmap: &mut Option<Bitmap>,
    _current_y: usize,
    region_info: &RegionInfo,
    src_bitmap: &Bitmap,
) -> Result<(), Jbig2Error> {
    let page_info = current_page_info
        .as_ref()
        .ok_or(Jbig2Error::new("no current page info"))?;
    let page_width = page_info.width as usize;
    let page_height = page_info.height as usize;
    let combo_op = if page_info.combination_operator_override {
        region_info.combination_operator
    } else {
        page_info.combination_operator
    };
    let dst = current_bitmap
        .as_mut()
        .ok_or(Jbig2Error::new("no current bitmap"))?;

    let reg_x = region_info.x as usize;
    let reg_y = region_info.y as usize;

    // Skip if the region lies entirely outside the page.
    if reg_x >= page_width || reg_y >= page_height {
        return Ok(());
    }

    dst.combine(src_bitmap, reg_x as isize, reg_y as isize, combo_op);
    Ok(())
}

/// Decode an immediate generic region and draw it on the page.
pub(super) fn on_immediate_generic_region(
    current_page_info: &mut Option<PageInfo>,
    current_bitmap: &mut Option<Bitmap>,
    current_y: usize,
    region: &GenericRegion,
    data: &[u8],
    start: usize,
    end: usize,
) -> Result<(), Jbig2Error> {
    let region_info = &region.info;
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
        *current_bitmap = Some(bitmap_utils::create_initialized_bitmap(width, height, 0));
    }

    let at_bytes = region.at.len() * 2;
    let decoding_start = start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 1 + at_bytes;
    if decoding_start > end {
        return Ok(());
    }

    let slice = &data[decoding_start..end];
    let mut decoding_context = DecodingContext::new(slice, 0, slice.len());

    let params = DecodeBitmapParams {
        mmr: region.mmr,
        width: region_info.width as usize,
        height: region_info.height as usize,
        template_index: region.template,
        prediction: region.prediction,
        skip: None,
        at: region.at.as_slice(),
    };

    let bitmap = decode_bitmap(&params, &mut decoding_context)?;
    draw_bitmap(
        current_page_info,
        current_bitmap,
        current_y,
        region_info,
        &bitmap,
    )?;
    Ok(())
}

/// Decode an immediate generic refinement region and draw it on the page.
#[allow(clippy::too_many_arguments)]
pub(super) fn on_immediate_generic_refinement_region(
    current_page_info: &mut Option<PageInfo>,
    current_bitmap: &mut Option<Bitmap>,
    current_y: usize,
    bitmaps: &HashMap<u32, Bitmap>,
    region_info: &RegionInfo,
    referred_to: &[u32],
    data: &[u8],
    start: usize,
    end: usize,
) -> Result<(), Jbig2Error> {
    if current_page_info.is_none() {
        *current_page_info = Some(PageInfo {
            width: region_info.width,
            height: region_info.height,
            resolution_x: 0,
            resolution_y: 0,
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
        let width = region_info.width as usize;
        let height = region_info.height as usize;
        *current_bitmap = Some(bitmap_utils::create_initialized_bitmap(width, height, 0));
    }

    // Parse refinement flags and optional AT offsets.
    let mut pos = start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH;
    let generic_region_segment_flags = data[pos];
    pos += 1;
    let template = (generic_region_segment_flags & 1) as usize;
    let prediction = (generic_region_segment_flags & 2) != 0;
    let at_length = if template == 0 { 2 } else { 0 };
    let at = if at_length > 0 {
        parse_at_parameters(data, pos, at_length)?
    } else {
        Vec::new()
    };
    pos += at_length * 2;
    if pos > end {
        return Ok(());
    }

    // Resolve the reference bitmap from the first referred segment.
    if referred_to.is_empty() {
        return Ok(());
    }
    let ref_segment = referred_to[0];
    let reference_bitmap = if let Some(bm) = bitmaps.get(&ref_segment) {
        bm
    } else {
        return Ok(());
    };

    let slice = &data[pos..end];
    let mut decoding_context = DecodingContext::new(slice, 0, slice.len());

    let bitmap = crate::decode::decode_refinement::decode_refinement(
        &crate::decode::decode_refinement::RefinementParams {
            width: region_info.width as usize,
            height: region_info.height as usize,
            template_index: template,
            reference_bitmap,
            offset_x: 0,
            offset_y: 0,
            prediction,
            at,
        },
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

/// Decode an intermediate generic region and store it by segment number.
#[allow(clippy::too_many_arguments)]
pub(super) fn on_intermediate_generic_region(
    symbols: &HashMap<u32, Vec<Bitmap>>,
    patterns: &HashMap<u32, Vec<Bitmap>>,
    custom_tables: &HashMap<u32, crate::huffman::HuffmanTable>,
    bitmaps: &mut HashMap<u32, Bitmap>,
    region: &GenericRegion,
    referred_to: &[u32],
    data: &[u8],
    start: usize,
    end: usize,
    segment_number: u32,
) -> Result<(), Jbig2Error> {
    // Validate that referenced segments are already available.
    for &seg_id in referred_to {
        if !symbols.contains_key(&seg_id)
            && !patterns.contains_key(&seg_id)
            && !custom_tables.contains_key(&seg_id)
            && !bitmaps.contains_key(&seg_id)
        {
            return Err(Jbig2Error::new("referred segment not found"));
        }
    }

    let region_info = &region.info;

    let at_bytes = region.at.len() * 2;
    let decoding_start = start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 1 + at_bytes;
    if decoding_start > end {
        return Ok(());
    }

    let slice = &data[decoding_start..end];
    let mut decoding_context = DecodingContext::new(slice, 0, slice.len());

    let params = DecodeBitmapParams {
        mmr: region.mmr,
        width: region_info.width as usize,
        height: region_info.height as usize,
        template_index: region.template,
        prediction: region.prediction,
        skip: None,
        at: region.at.as_slice(),
    };

    let bitmap = decode_bitmap(&params, &mut decoding_context)?;

    bitmaps.insert(segment_number, bitmap);
    Ok(())
}

/// Decode an intermediate generic refinement region and store it by segment number.
pub(super) fn on_intermediate_generic_refinement_region(
    bitmaps: &mut HashMap<u32, Bitmap>,
    region_info: &RegionInfo,
    referred_to: &[u32],
    data: &[u8],
    start: usize,
    end: usize,
    segment_number: u32,
) -> Result<(), Jbig2Error> {
    // Parse refinement flags and optional AT offsets.
    let mut pos = start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH;
    let generic_region_segment_flags = data[pos];
    pos += 1;
    let template = (generic_region_segment_flags & 1) as usize;
    let prediction = (generic_region_segment_flags & 2) != 0;
    let at_length = if template == 0 { 2 } else { 0 };
    let at = if at_length > 0 && pos + at_length * 2 <= end {
        parse_at_parameters(data, pos, at_length)?
    } else {
        Vec::new()
    };
    pos += at_length * 2;
    if pos > end {
        return Ok(());
    }

    // Resolve the reference bitmap from the first referred segment.
    if referred_to.is_empty() {
        return Ok(());
    }
    let ref_segment = referred_to[0];
    let reference_bitmap = if let Some(bm) = bitmaps.get(&ref_segment) {
        bm
    } else {
        return Ok(());
    };

    let slice = &data[pos..end];
    let mut decoding_context = DecodingContext::new(slice, 0, slice.len());

    let bitmap = crate::decode::decode_refinement::decode_refinement(
        &crate::decode::decode_refinement::RefinementParams {
            width: region_info.width as usize,
            height: region_info.height as usize,
            template_index: template,
            reference_bitmap,
            offset_x: 0,
            offset_y: 0,
            prediction,
            at,
        },
        &mut decoding_context,
    )?;

    bitmaps.insert(segment_number, bitmap);
    Ok(())
}
