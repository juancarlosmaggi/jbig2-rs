use crate::arithmetic::contexts::DecodingContext;
use crate::bitmap::Bitmap;
use crate::bitmap::utils as bitmap_utils;
use crate::common::error::Jbig2Error;
use crate::decoders::generic::{DecodeBitmapParams, decode_bitmap};
use crate::document::PageInfo;
use crate::parser::segment::{GenericRegion, RegionInfo, parse_at_parameters};
use std::collections::HashMap;

use super::{IntermediateResources, PageComposeTarget, SegmentSlice};

pub(super) const REGION_SEGMENT_INFORMATION_FIELD_LENGTH: usize = 17;

/// Composite a decoded region bitmap onto the current page bitmap.
pub(super) fn draw_bitmap(
    page: PageComposeTarget<'_>,
    region_info: &RegionInfo,
    src_bitmap: &Bitmap,
) -> Result<(), Jbig2Error> {
    let page_info = page
        .page_info
        .as_ref()
        .ok_or(Jbig2Error::new("no current page info"))?;
    let page_width = page_info.width as usize;
    let page_height = page_info.height as usize;
    let combo_op = if page_info.combination_operator_override {
        region_info.combination_operator
    } else {
        page_info.combination_operator
    };
    let dst = page
        .bitmap
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
    page: PageComposeTarget<'_>,
    region: &GenericRegion,
    bytes: SegmentSlice<'_>,
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
    ensure_page_for_region(
        page.page_info,
        page.bitmap,
        region_info,
        PageDefaults {
            resolution: 300,
            use_max_dimension: true,
        },
    );

    let at_bytes = region.at.len() * 2;
    let decoding_start = bytes.start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 1 + at_bytes;
    if decoding_start > bytes.end {
        return Ok(());
    }

    let slice = &bytes.data[decoding_start..bytes.end];
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
    draw_bitmap(page, region_info, &bitmap)
}

/// Decode an immediate generic refinement region and draw it on the page.
pub(super) fn on_immediate_generic_refinement_region(
    page: PageComposeTarget<'_>,
    bitmaps: &HashMap<u32, Bitmap>,
    region_info: &RegionInfo,
    referred_to: &[u32],
    bytes: SegmentSlice<'_>,
) -> Result<(), Jbig2Error> {
    ensure_page_for_region(
        page.page_info,
        page.bitmap,
        region_info,
        PageDefaults {
            resolution: 0,
            use_max_dimension: false,
        },
    );

    let Some(bitmap) = decode_generic_refinement(bitmaps, region_info, referred_to, bytes, true)?
    else {
        return Ok(());
    };

    draw_bitmap(page, region_info, &bitmap)
}

/// Decode an intermediate generic region and store it by segment number.
pub(super) fn on_intermediate_generic_region(
    resources: IntermediateResources<'_>,
    region: &GenericRegion,
    referred_to: &[u32],
    bytes: SegmentSlice<'_>,
    segment_number: u32,
) -> Result<(), Jbig2Error> {
    for &seg_id in referred_to {
        if !resources.symbols.contains_key(&seg_id)
            && !resources.patterns.contains_key(&seg_id)
            && !resources.custom_tables.contains_key(&seg_id)
            && !resources.bitmaps.contains_key(&seg_id)
        {
            return Err(Jbig2Error::new("referred segment not found"));
        }
    }

    let region_info = &region.info;

    let at_bytes = region.at.len() * 2;
    let decoding_start = bytes.start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 1 + at_bytes;
    if decoding_start > bytes.end {
        return Ok(());
    }

    let slice = &bytes.data[decoding_start..bytes.end];
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

    resources.bitmaps.insert(segment_number, bitmap);
    Ok(())
}

/// Decode an intermediate generic refinement region and store it by segment number.
pub(super) fn on_intermediate_generic_refinement_region(
    bitmaps: &mut HashMap<u32, Bitmap>,
    region_info: &RegionInfo,
    referred_to: &[u32],
    bytes: SegmentSlice<'_>,
    segment_number: u32,
) -> Result<(), Jbig2Error> {
    let Some(bitmap) = decode_generic_refinement(bitmaps, region_info, referred_to, bytes, false)?
    else {
        return Ok(());
    };
    bitmaps.insert(segment_number, bitmap);
    Ok(())
}

struct PageDefaults {
    resolution: u32,
    use_max_dimension: bool,
}

fn ensure_page_for_region(
    current_page_info: &mut Option<PageInfo>,
    current_bitmap: &mut Option<Bitmap>,
    region_info: &RegionInfo,
    defaults: PageDefaults,
) {
    if current_page_info.is_some() {
        return;
    }
    let width = if defaults.use_max_dimension {
        region_info.width.max(1)
    } else {
        region_info.width
    };
    let height = if defaults.use_max_dimension {
        region_info.height.max(1)
    } else {
        region_info.height
    };
    *current_page_info = Some(PageInfo {
        width,
        height,
        resolution_x: defaults.resolution,
        resolution_y: defaults.resolution,
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
    *current_bitmap = Some(bitmap_utils::create_initialized_bitmap(
        width as usize,
        height as usize,
        0,
    ));
}

fn decode_generic_refinement(
    bitmaps: &HashMap<u32, Bitmap>,
    region_info: &RegionInfo,
    referred_to: &[u32],
    bytes: SegmentSlice<'_>,
    require_at: bool,
) -> Result<Option<Bitmap>, Jbig2Error> {
    let data = bytes.data;
    let end = bytes.end;
    let mut pos = bytes.start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH;
    let generic_region_segment_flags = data[pos];
    pos += 1;
    let template = (generic_region_segment_flags & 1) as usize;
    let prediction = (generic_region_segment_flags & 2) != 0;
    let at_length = if template == 0 { 2 } else { 0 };
    let at = if at_length == 0 {
        Vec::new()
    } else if require_at || pos + at_length * 2 <= end {
        parse_at_parameters(data, pos, at_length)?
    } else {
        Vec::new()
    };
    pos += at_length * 2;
    if pos > end {
        return Ok(None);
    }

    if referred_to.is_empty() {
        return Ok(None);
    }
    let ref_segment = referred_to[0];
    let Some(reference_bitmap) = bitmaps.get(&ref_segment) else {
        return Ok(None);
    };

    let slice = &data[pos..end];
    let mut decoding_context = DecodingContext::new(slice, 0, slice.len());

    crate::decoders::refinement::decode_refinement(
        &crate::decoders::refinement::RefinementParams {
            width: region_info.width as usize,
            height: region_info.height as usize,
            template_index: template,
            reference_bitmap,
            offset_x: 0,
            offset_y: 0,
            prediction,
            at: at.as_slice(),
        },
        &mut decoding_context,
    )
    .map(Some)
}
