use crate::bitmap::Bitmap;
use crate::bitmap_utils;
use crate::contexts::DecodingContext;
use crate::decode::decode_generic::{DecodeBitmapParams, decode_bitmap};
use crate::error::Jbig2Error;

/// Inputs needed to decode a halftone region.
#[derive(Clone)]
pub struct HalftoneRegionParams {
    pub mmr: bool,
    pub patterns: Vec<Bitmap>,
    pub template: usize,
    pub region_width: usize,
    pub region_height: usize,
    pub default_pixel_value: u8,
    pub enable_skip: bool,
    pub combination_operator: usize,
    pub grid_width: usize,
    pub grid_height: usize,
    pub grid_offset_x: i32,
    pub grid_offset_y: i32,
    pub grid_vector_x: i16,
    pub grid_vector_y: i16,
}

/// Decode a halftone region bitmap from the supplied parameters and context.
pub fn decode_halftone_region(
    params: &HalftoneRegionParams,
    decoding_context: &mut DecodingContext,
) -> Result<Bitmap, Jbig2Error> {
    if params.combination_operator != 0 {
        return Err(Jbig2Error::new("only OR combination operator is supported"));
    }
    // Initialize the output bitmap.
    let mut region_bitmap = bitmap_utils::create_initialized_bitmap(
        params.region_width,
        params.region_height,
        params.default_pixel_value,
    );
    let number_of_patterns = params.patterns.len();
    if number_of_patterns == 0 {
        return Ok(region_bitmap);
    }
    let pattern0 = &params.patterns[0];
    let pattern_width = pattern0.width;
    let pattern_height = pattern0.height;
    let bits_per_value = crate::core_utils::log2(number_of_patterns as u32) as usize;
    let at = if !params.mmr {
        let mut at_vec = vec![(if params.template <= 1 { 3i8 } else { 2i8 }, -1i8)];
        if params.template <= 1 {
            at_vec.extend(vec![(-3i8, -1i8), (2i8, -2i8), (-2i8, -2i8)]);
        }
        at_vec
    } else {
        vec![]
    };
    // Build a skip bitmap from the grid geometry when enabled.
    let skip_bitmap = if params.enable_skip && !params.mmr {
        let mut skip = Bitmap::new(params.grid_width, params.grid_height);
        let grid_vector_x = params.grid_vector_x as i64;
        let grid_vector_y = params.grid_vector_y as i64;
        let grid_offset_x = params.grid_offset_x as i64;
        let grid_offset_y = params.grid_offset_y as i64;
        let region_width = params.region_width as i64;
        let region_height = params.region_height as i64;
        let pattern_width = pattern_width as i64;
        let pattern_height = pattern_height as i64;
        for mg in 0..params.grid_height {
            let base_x = grid_offset_x + mg as i64 * grid_vector_y;
            let base_y = grid_offset_y + mg as i64 * grid_vector_x;
            let mut x = base_x;
            let mut y = base_y;
            for ng in 0..params.grid_width {
                let region_x = x >> 8;
                let region_y = y >> 8;
                let outside = region_x + pattern_width <= 0
                    || region_x >= region_width
                    || region_y + pattern_height <= 0
                    || region_y >= region_height;
                if outside {
                    skip.set_pixel(ng, mg, 1);
                }
                x += grid_vector_x;
                y -= grid_vector_y;
            }
        }
        Some(skip)
    } else {
        None
    };
    // Decode gray-scale bit planes from MSB to LSB, then gray-decode with XOR.
    let mut gray_scale_bit_planes = vec![Bitmap::new(0, 0); bits_per_value];
    for j in (0..bits_per_value).rev() {
        let decode_params = DecodeBitmapParams {
            mmr: params.mmr,
            width: params.grid_width,
            height: params.grid_height,
            template_index: params.template,
            prediction: false,
            skip: skip_bitmap.as_ref(),
            at: at.clone(),
        };
        let bitmap = decode_bitmap(&decode_params, decoding_context)?;
        gray_scale_bit_planes[j] = bitmap;
        if j + 1 < bits_per_value {
            for idx in 0..gray_scale_bit_planes[j].data.len() {
                gray_scale_bit_planes[j].data[idx] ^= gray_scale_bit_planes[j + 1].data[idx];
            }
        }
    }
    // Render patterns into the output bitmap using the grid geometry.
    let patterns_len = params.patterns.len();
    let grid_vector_x = params.grid_vector_x as i64;
    let grid_vector_y = params.grid_vector_y as i64;
    let grid_offset_x = params.grid_offset_x as i64;
    let grid_offset_y = params.grid_offset_y as i64;
    let region_width = params.region_width as i64;
    let region_height = params.region_height as i64;
    let plane_stride = if bits_per_value > 0 {
        gray_scale_bit_planes[0].stride
    } else {
        0
    };
    for mg in 0..params.grid_height {
        let base_x = grid_offset_x + mg as i64 * grid_vector_y;
        let base_y = grid_offset_y + mg as i64 * grid_vector_x;
        let mut x = base_x;
        let mut y = base_y;
        let row_offset = mg * plane_stride;
        for ng in 0..params.grid_width {
            let mut pattern_index = 0usize;
            if bits_per_value > 0 {
                let byte_index = row_offset + (ng >> 3);
                let bit_mask = 1u8 << (7 - (ng & 7));
                for (j, plane) in gray_scale_bit_planes.iter().enumerate() {
                    if (plane.data[byte_index] & bit_mask) != 0 {
                        pattern_index |= 1usize << j;
                    }
                }
            }
            if pattern_index >= patterns_len {
                pattern_index = patterns_len.saturating_sub(1);
            }
            let pattern_bitmap = &params.patterns[pattern_index];
            let region_x = x >> 8;
            let region_y = y >> 8;
            let pattern_width = pattern_bitmap.width as i64;
            let pattern_height = pattern_bitmap.height as i64;
            if region_x + pattern_width <= 0
                || region_x >= region_width
                || region_y + pattern_height <= 0
                || region_y >= region_height
            {
                x += grid_vector_x;
                y -= grid_vector_y;
                continue;
            }
            region_bitmap.combine(pattern_bitmap, region_x as isize, region_y as isize, 0);
            x += grid_vector_x;
            y -= grid_vector_y;
        }
    }
    Ok(region_bitmap)
}
