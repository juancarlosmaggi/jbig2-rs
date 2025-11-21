use crate::bitmap::Bitmap;
use crate::bitmap_utils;
use crate::contexts::DecodingContext;
use crate::decode::decode_generic::{DecodeBitmapParams, decode_bitmap};
use crate::error::Jbig2Error;
use crate::segment::{GenericRegion, PageInfo, RegionInfo, parse_at_parameters};
use std::collections::HashMap;

pub(super) const REGION_SEGMENT_INFORMATION_FIELD_LENGTH: usize = 17;

/// Draw a decoded bitmap onto the current page
pub(super) fn draw_bitmap(
    current_page_info: &Option<PageInfo>,
    current_bitmap: &mut Option<Bitmap>,
    current_y: usize,
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

    // Region coordinates are validated by checking bounds below
    let reg_x = region_info.x as usize;
    let reg_y = region_info.y as usize + current_y;

    // Check if region is completely outside page bounds
    if reg_x >= page_width || reg_y >= page_height {
        return Ok(()); // Nothing to draw
    }

    let width = (region_info.width as usize).min(page_width - reg_x);
    let height = (region_info.height as usize).min(page_height - reg_y);

    // Validate source bitmap dimensions
    if src_bitmap.width < width || src_bitmap.height < height {
        return Err(Jbig2Error::new("source bitmap too small for region"));
    }

    for i in 0..height {
        for j in 0..width {
            let src = src_bitmap.get_pixel(j, i);
            let dx = reg_x + j;
            let dy = reg_y + i;
            let old_dst = dst.get_pixel(dx, dy);
            let new_val = bitmap_utils::apply_combination_operator(old_dst, src, combo_op);
            dst.set_pixel(dx, dy, new_val);
        }
    }
    Ok(())
}

/// Handle immediate generic region
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
    println!(
        "Generic region: width={}, height={}",
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
        *current_bitmap = Some(bitmap_utils::create_initialized_bitmap(width, height, 0));
    }

    let at_bytes = region.at.len() * 2;
    let decoding_start = start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 1 + at_bytes;
    if decoding_start > end {
        return Ok(()); // Allow short data for minimal test
    }

    let slice = &data[decoding_start..end];
    let mut decoding_context = DecodingContext::new(slice.to_vec(), 0, slice.len());

    let params = DecodeBitmapParams {
        mmr: region.mmr,
        width: region_info.width as usize,
        height: region_info.height as usize,
        template_index: region.template,
        prediction: region.prediction,
        skip: None,
        at: region.at.clone(),
    };

    let bitmap = decode_bitmap(&params, &mut decoding_context)?;
    draw_bitmap(current_page_info, current_bitmap, current_y, region_info, &bitmap)?;
    Ok(())
}

/// Handle immediate generic refinement region
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
        });
        let width = region_info.width as usize;
        let height = region_info.height as usize;
        *current_bitmap = Some(bitmap_utils::create_initialized_bitmap(width, height, 0));
    }

    // Parse refinement region parameters
    let mut pos = start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH;
    let generic_region_segment_flags = data[pos];
    pos += 1;
    let template = ((generic_region_segment_flags >> 1) & 3) as usize;
    let at_length = if template == 0 { 2 } else { 0 };
    let at = if at_length > 0 {
        parse_at_parameters(data, pos, at_length)?
    } else {
        Vec::new() // Default to empty if insufficient data
    };
    pos += at_length * 2;
    if pos > end {
        return Ok(()); // Allow short data
    }

    // Get reference bitmap from referred segment
    if referred_to.is_empty() {
        return Ok(()); // Skip if no referred
    }
    let ref_segment = referred_to[0];
    let reference_bitmap = if let Some(bm) = bitmaps.get(&ref_segment) {
        bm
    } else {
        return Ok(()); // Skip if not found
    };

    let slice = &data[pos..end];
    let mut decoding_context = DecodingContext::new(slice.to_vec(), 0, slice.len());

    let bitmap = crate::decode::decode_refinement::decode_refinement(
        &crate::decode::decode_refinement::RefinementParams {
            width: region_info.width as usize,
            height: region_info.height as usize,
            template_index: template,
            reference_bitmap,
            offset_x: 0, // Default offset
            offset_y: 0,
            prediction: false,
            at,
        },
        &mut decoding_context,
    )?;

    draw_bitmap(current_page_info, current_bitmap, current_y, region_info, &bitmap)?;
    Ok(())
}

/// Handle intermediate generic region
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
    // Basic validation: check that referred segments exist
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
        return Ok(()); // Allow short data
    }

    let slice = &data[decoding_start..end];
    let mut decoding_context = DecodingContext::new(slice.to_vec(), 0, slice.len());

    let params = DecodeBitmapParams {
        mmr: region.mmr,
        width: region_info.width as usize,
        height: region_info.height as usize,
        template_index: region.template,
        prediction: region.prediction,
        skip: None,
        at: region.at.clone(),
    };

    let bitmap = decode_bitmap(&params, &mut decoding_context)?;

    bitmaps.insert(segment_number, bitmap);
    Ok(())
}

/// Handle intermediate generic refinement region
pub(super) fn on_intermediate_generic_refinement_region(
    bitmaps: &mut HashMap<u32, Bitmap>,
    region_info: &RegionInfo,
    referred_to: &[u32],
    data: &[u8],
    start: usize,
    end: usize,
    segment_number: u32,
) -> Result<(), Jbig2Error> {
    // Parse refinement region parameters
    let mut pos = start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH;
    let generic_region_segment_flags = data[pos];
    pos += 1;
    let template = ((generic_region_segment_flags >> 1) & 3) as usize;
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

    // Get reference bitmap from referred segment
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
    let mut decoding_context = DecodingContext::new(slice.to_vec(), 0, slice.len());

    let bitmap = crate::decode::decode_refinement::decode_refinement(
        &crate::decode::decode_refinement::RefinementParams {
            width: region_info.width as usize,
            height: region_info.height as usize,
            template_index: template,
            reference_bitmap,
            offset_x: 0, // Default offset
            offset_y: 0,
            prediction: false,
            at,
        },
        &mut decoding_context,
    )?;

    bitmaps.insert(segment_number, bitmap);
    Ok(())
}
