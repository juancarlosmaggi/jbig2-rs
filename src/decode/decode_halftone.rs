use crate::bitmap::Bitmap;
use crate::bitmap_utils;
use crate::contexts::DecodingContext;
use crate::decode::decode_generic::{DecodeBitmapParams, decode_bitmap};
use crate::error::Jbig2Error;

/// Inputs needed to decode a halftone region.
#[derive(Clone)]
pub struct HalftoneRegionParams<'a> {
    pub mmr: bool,
    pub patterns: &'a [Bitmap],
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
    params: &HalftoneRegionParams<'_>,
    decoding_context: &mut DecodingContext<'_>,
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
    let pattern_width = pattern0.width as i64;
    let pattern_height = pattern0.height as i64;
    let bits_per_value = crate::core_utils::log2(number_of_patterns as u32) as usize;
    const HALFTONE_AT_TEMPLATE_0_1: [(i8, i8); 4] =
        [(3, -1), (-3, -1), (2, -2), (-2, -2)];
    const HALFTONE_AT_TEMPLATE_2_3: [(i8, i8); 1] = [(2, -1)];
    let at: &[(i8, i8)] = if params.mmr {
        &[]
    } else if params.template <= 1 {
        &HALFTONE_AT_TEMPLATE_0_1
    } else {
        &HALFTONE_AT_TEMPLATE_2_3
    };
    // Build a skip bitmap from the grid geometry when enabled.
    let skip_bitmap = if params.enable_skip && !params.mmr {
        let region_width = params.region_width as i64;
        let region_height = params.region_height as i64;
        if params.grid_width == 0
            || params.grid_height == 0
            || grid_fully_inside_region(
                params,
                region_width,
                region_height,
                pattern_width,
                pattern_height,
            )
        {
            None
        } else {
        let mut skip = Bitmap::new(params.grid_width, params.grid_height);
        let grid_vector_x = params.grid_vector_x as i64;
        let grid_vector_y = params.grid_vector_y as i64;
        let grid_offset_x = params.grid_offset_x as i64;
        let grid_offset_y = params.grid_offset_y as i64;
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
        }
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
            at,
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
    let pattern_has_black: Vec<bool> = params
        .patterns
        .iter()
        .map(|pattern| pattern.data.iter().any(|&b| b != 0))
        .collect();
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
    let plane0 = if bits_per_value > 0 {
        gray_scale_bit_planes[0].data.as_slice()
    } else {
        &[]
    };
    let plane1 = if bits_per_value > 1 {
        gray_scale_bit_planes[1].data.as_slice()
    } else {
        &[]
    };
    let plane2 = if bits_per_value > 2 {
        gray_scale_bit_planes[2].data.as_slice()
    } else {
        &[]
    };
    let plane3 = if bits_per_value > 3 {
        gray_scale_bit_planes[3].data.as_slice()
    } else {
        &[]
    };
    for mg in 0..params.grid_height {
        let base_x = grid_offset_x + mg as i64 * grid_vector_y;
        let base_y = grid_offset_y + mg as i64 * grid_vector_x;
        let mut x = base_x;
        let mut y = base_y;
        let row_offset = mg * plane_stride;
        let plane0_row = if bits_per_value > 0 {
            &plane0[row_offset..]
        } else {
            &[]
        };
        let plane1_row = if bits_per_value > 1 {
            &plane1[row_offset..]
        } else {
            &[]
        };
        let plane2_row = if bits_per_value > 2 {
            &plane2[row_offset..]
        } else {
            &[]
        };
        let plane3_row = if bits_per_value > 3 {
            &plane3[row_offset..]
        } else {
            &[]
        };
        for ng in 0..params.grid_width {
            let byte_index = ng >> 3;
            let bit_mask = 1u8 << (7 - (ng & 7));
            let mut pattern_index = match bits_per_value {
                0 => 0usize,
                1 => ((plane0_row[byte_index] & bit_mask) != 0) as usize,
                2 => {
                    ((plane0_row[byte_index] & bit_mask) != 0) as usize
                        | (((plane1_row[byte_index] & bit_mask) != 0) as usize) << 1
                }
                3 => {
                    ((plane0_row[byte_index] & bit_mask) != 0) as usize
                        | (((plane1_row[byte_index] & bit_mask) != 0) as usize) << 1
                        | (((plane2_row[byte_index] & bit_mask) != 0) as usize) << 2
                }
                4 => {
                    ((plane0_row[byte_index] & bit_mask) != 0) as usize
                        | (((plane1_row[byte_index] & bit_mask) != 0) as usize) << 1
                        | (((plane2_row[byte_index] & bit_mask) != 0) as usize) << 2
                        | (((plane3_row[byte_index] & bit_mask) != 0) as usize) << 3
                }
                _ => {
                    let mut pattern_index = 0usize;
                    for (j, plane) in gray_scale_bit_planes.iter().enumerate() {
                        if (plane.data[row_offset + byte_index] & bit_mask) != 0 {
                            pattern_index |= 1usize << j;
                        }
                    }
                    pattern_index
                }
            };
            if pattern_index >= patterns_len {
                pattern_index = patterns_len.saturating_sub(1);
            }
            if !pattern_has_black[pattern_index] {
                x += grid_vector_x;
                y -= grid_vector_y;
                continue;
            }
            let region_x = x >> 8;
            let region_y = y >> 8;
            if region_x + pattern_width <= 0
                || region_x >= region_width
                || region_y + pattern_height <= 0
                || region_y >= region_height
            {
                x += grid_vector_x;
                y -= grid_vector_y;
                continue;
            }
            let pattern_bitmap = &params.patterns[pattern_index];
            region_bitmap.combine_or(pattern_bitmap, region_x as isize, region_y as isize);
            x += grid_vector_x;
            y -= grid_vector_y;
        }
    }
    Ok(region_bitmap)
}

fn grid_fully_inside_region(
    params: &HalftoneRegionParams<'_>,
    region_width: i64,
    region_height: i64,
    pattern_width: i64,
    pattern_height: i64,
) -> bool {
    let last_mg = (params.grid_height.saturating_sub(1)) as i64;
    let last_ng = (params.grid_width.saturating_sub(1)) as i64;
    let grid_vector_x = params.grid_vector_x as i64;
    let grid_vector_y = params.grid_vector_y as i64;
    let grid_offset_x = params.grid_offset_x as i64;
    let grid_offset_y = params.grid_offset_y as i64;

    let x00 = grid_offset_x;
    let y00 = grid_offset_y;
    let x01 = grid_offset_x + last_ng * grid_vector_x;
    let y01 = grid_offset_y - last_ng * grid_vector_y;
    let x10 = grid_offset_x + last_mg * grid_vector_y;
    let y10 = grid_offset_y + last_mg * grid_vector_x;
    let x11 = grid_offset_x + last_mg * grid_vector_y + last_ng * grid_vector_x;
    let y11 = grid_offset_y + last_mg * grid_vector_x - last_ng * grid_vector_y;

    let xs = [x00 >> 8, x01 >> 8, x10 >> 8, x11 >> 8];
    let ys = [y00 >> 8, y01 >> 8, y10 >> 8, y11 >> 8];

    let min_x = *xs.iter().min().unwrap_or(&0);
    let max_x = *xs.iter().max().unwrap_or(&0);
    let min_y = *ys.iter().min().unwrap_or(&0);
    let max_y = *ys.iter().max().unwrap_or(&0);

    min_x >= 0
        && min_y >= 0
        && max_x + pattern_width <= region_width
        && max_y + pattern_height <= region_height
}
