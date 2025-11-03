use crate::bitmap::Bitmap;
use crate::contexts::DecodingContext;
use crate::decode_generic::{DecodeBitmapParams, decode_bitmap};
use crate::error::Jbig2Error;

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

pub fn decode_halftone_region(
    params: &HalftoneRegionParams,
    decoding_context: &mut DecodingContext,
) -> Result<Bitmap, Jbig2Error> {
    if params.enable_skip {
        return Err(Jbig2Error::new("skip is not supported"));
    }
    if params.combination_operator != 0 {
        return Err(Jbig2Error::new("only OR combination operator is supported"));
    }
    // Prepare bitmap
    let mut region_bitmap = Bitmap::new(params.region_width, params.region_height);
    if params.default_pixel_value != 0 {
        for y in 0..params.region_height {
            for x in 0..params.region_width {
                region_bitmap.set_pixel(x, y, 1);
            }
        }
    }
    let number_of_patterns = params.patterns.len();
    if number_of_patterns == 0 {
        return Ok(region_bitmap);
    }
    let pattern0 = &params.patterns[0];
    let _pattern_width = pattern0.width;
    let pattern_height = pattern0.height;
    let bits_per_value = crate::core_utils::log2(number_of_patterns as u32) as usize;
    let at = if !params.mmr {
        let mut at_vec = vec![(if params.template <= 1 { 3i8 } else { 2i8 }, -1i8)];
        if params.template == 0 {
            at_vec.extend(vec![(-3i8, -1i8), (2i8, -2i8), (-2i8, -2i8)]);
        }
        at_vec
    } else {
        vec![]
    };
    // Gray-scale bit planes
    let mut gray_scale_bit_planes = Vec::new();
    for _ in (0..bits_per_value).rev() {
        let decode_params = DecodeBitmapParams {
            mmr: params.mmr,
            width: params.grid_width,
            height: params.grid_height,
            template_index: params.template,
            prediction: false,
            skip: None,
            at: at.clone(),
        };
        let bitmap = decode_bitmap(&decode_params, decoding_context)?;
        gray_scale_bit_planes.push(bitmap);
    }
    // Render patterns
    for mg in 0..params.grid_height {
        for ng in 0..params.grid_width {
            let mut bit = 0u8;
            let mut pattern_index = 0usize;
            for j in (0..bits_per_value).rev() {
                let plane_bit = gray_scale_bit_planes[j].get_pixel(ng, mg);
                bit ^= plane_bit;
                pattern_index |= (bit as usize) << j;
            }
            if pattern_index >= params.patterns.len() {
                continue;
            }
            let pattern_bitmap = &params.patterns[pattern_index];
            let x = (params.grid_offset_x
                + mg as i32 * params.grid_vector_y as i32
                + ng as i32 * params.grid_vector_x as i32)
                >> 8;
            let y = (params.grid_offset_y + mg as i32 * params.grid_vector_x as i32
                - ng as i32 * params.grid_vector_y as i32)
                >> 8;
            // Draw pattern
            if x >= 0
                && x + pattern_bitmap.width as i32 <= params.region_width as i32
                && y >= 0
                && y + pattern_bitmap.height as i32 <= params.region_height as i32
            {
                for i in 0..pattern_bitmap.height {
                    for j in 0..pattern_bitmap.width {
                        let src_pixel = pattern_bitmap.get_pixel(j, i);
                        let dst_pixel = region_bitmap
                            .get_pixel((x + j as i32) as usize, (y + i as i32) as usize);
                        let new_pixel = src_pixel | dst_pixel; // OR
                        region_bitmap.set_pixel(
                            (x + j as i32) as usize,
                            (y + i as i32) as usize,
                            new_pixel,
                        );
                    }
                }
            } else {
                // Handle partial patterns at edges
                for i in 0..pattern_height {
                    let region_y = y + i as i32;
                    if region_y < 0 || region_y >= params.region_height as i32 {
                        continue;
                    }
                    for j in 0..pattern_bitmap.width {
                        let region_x = x + j as i32;
                        if region_x >= 0 && region_x < params.region_width as i32 {
                            let src_pixel = pattern_bitmap.get_pixel(j, i);
                            let dst_pixel =
                                region_bitmap.get_pixel(region_x as usize, region_y as usize);
                            let new_pixel = src_pixel | dst_pixel; // OR
                            region_bitmap.set_pixel(
                                region_x as usize,
                                region_y as usize,
                                new_pixel,
                            );
                        }
                    }
                }
            }
        }
    }
    Ok(region_bitmap)
}
