use crate::bitmap::Bitmap;
use crate::bitmap_utils;
use crate::contexts::DecodingContext;
use crate::decode::decode_generic::{DecodeBitmapParams, decode_bitmap};
use crate::error::Jbig2Error;

const BIT_MASKS: [u8; 8] = [0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01];

struct ShiftedRows {
    stride: usize,
    data: Vec<u8>,
}

struct ShiftedPattern {
    shifts: [ShiftedRows; 8],
    has_black: bool,
}

fn build_shifted_rows(pattern: &Bitmap, shift: usize) -> ShiftedRows {
    if pattern.width == 0 || pattern.height == 0 {
        return ShiftedRows {
            stride: 0,
            data: Vec::new(),
        };
    }

    let width = pattern.width;
    let height = pattern.height;
    let src_stride = pattern.stride;
    let dst_stride = (width + shift + 7) >> 3;
    let src_rem_bits = width & 7;
    let src_mask = if src_rem_bits == 0 {
        0xFF
    } else {
        0xFFu8 << (8 - src_rem_bits)
    };
    let total_bits = width + shift;
    let rem_bits = total_bits & 7;
    let last_mask = if rem_bits == 0 {
        0xFF
    } else {
        0xFFu8 << (8 - rem_bits)
    };

    let mut data = vec![0u8; dst_stride * height];

    for row in 0..height {
        let src_row_start = row * src_stride;
        let dst_row_start = row * dst_stride;
        let src_row = &pattern.data[src_row_start..src_row_start + src_stride];
        let dst_row = &mut data[dst_row_start..dst_row_start + dst_stride];

        if shift == 0 {
            dst_row.copy_from_slice(src_row);
        } else {
            let mut carry = 0u8;
            let mut dst_idx = 0usize;
            for (idx, &b0) in src_row.iter().enumerate() {
                let mut b = b0;
                if src_rem_bits != 0 && idx + 1 == src_stride {
                    b &= src_mask;
                }
                let out = (b >> shift) | carry;
                if dst_idx < dst_stride {
                    dst_row[dst_idx] = out;
                    dst_idx += 1;
                } else {
                    break;
                }
                carry = b << (8 - shift);
            }
            if dst_idx < dst_stride {
                dst_row[dst_idx] = carry;
            }
        }

        if rem_bits != 0 && dst_stride > 0 {
            dst_row[dst_stride - 1] &= last_mask;
        }
    }

    ShiftedRows { stride: dst_stride, data }
}

fn build_shifted_pattern(pattern: &Bitmap) -> ShiftedPattern {
    let has_black = pattern.data.iter().any(|&b| b != 0);
    let shifts = std::array::from_fn(|shift| build_shifted_rows(pattern, shift));
    ShiftedPattern { shifts, has_black }
}

fn or_row_bytes(dst: &mut [u8], src: &[u8]) {
    let len = dst.len().min(src.len());
    let mut idx = 0usize;
    unsafe {
        while idx + 8 <= len {
            let dst_ptr = dst.as_mut_ptr().add(idx) as *mut u64;
            let src_ptr = src.as_ptr().add(idx) as *const u64;
            let dst_val = std::ptr::read_unaligned(dst_ptr);
            let src_val = std::ptr::read_unaligned(src_ptr);
            std::ptr::write_unaligned(dst_ptr, dst_val | src_val);
            idx += 8;
        }
    }
    while idx < len {
        dst[idx] |= src[idx];
        idx += 1;
    }
}

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
    let pattern_width_usize = pattern0.width;
    let pattern_height_usize = pattern0.height;
    let pattern_width = pattern_width_usize as i64;
    let pattern_height = pattern_height_usize as i64;
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
            let (left, right) = gray_scale_bit_planes.split_at_mut(j + 1);
            let dst = &mut left[j].data;
            let src = &right[0].data;
            xor_plane_bytes(dst, src);
        }
    }
    // Render patterns into the output bitmap using the grid geometry.
    let patterns_len = params.patterns.len();
    let shifted_patterns: Vec<ShiftedPattern> = params
        .patterns
        .iter()
        .map(build_shifted_pattern)
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
            let bit_mask = BIT_MASKS[ng & 7];
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
            let shifted_pattern = &shifted_patterns[pattern_index];
            if !shifted_pattern.has_black {
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
            let inside = region_x >= 0
                && region_y >= 0
                && region_x + pattern_width <= region_width
                && region_y + pattern_height <= region_height;
            if inside {
                let region_x_u = region_x as usize;
                let region_y_u = region_y as usize;
                let shift = region_x_u & 7;
                let shifted_rows = &shifted_pattern.shifts[shift];
                let src_stride = shifted_rows.stride;
                let src_data = shifted_rows.data.as_slice();
                let dst_stride = region_bitmap.stride;
                let dst_byte_offset = region_x_u >> 3;
                let dst_data = &mut region_bitmap.data;
                for row in 0..pattern_height_usize {
                    let dst_row_start = (region_y_u + row) * dst_stride + dst_byte_offset;
                    let src_row_start = row * src_stride;
                    let dst_row = &mut dst_data[dst_row_start..dst_row_start + src_stride];
                    let src_row = &src_data[src_row_start..src_row_start + src_stride];
                    or_row_bytes(dst_row, src_row);
                }
            } else {
                let pattern_bitmap = &params.patterns[pattern_index];
                region_bitmap.combine_or(pattern_bitmap, region_x as isize, region_y as isize);
            }
            x += grid_vector_x;
            y -= grid_vector_y;
        }
    }
    Ok(region_bitmap)
}

fn xor_plane_bytes(dst: &mut [u8], src: &[u8]) {
    let len = dst.len().min(src.len());
    let mut idx = 0usize;
    unsafe {
        while idx + 8 <= len {
            let dst_ptr = dst.as_mut_ptr().add(idx) as *mut u64;
            let src_ptr = src.as_ptr().add(idx) as *const u64;
            let dst_val = std::ptr::read_unaligned(dst_ptr);
            let src_val = std::ptr::read_unaligned(src_ptr);
            std::ptr::write_unaligned(dst_ptr, dst_val ^ src_val);
            idx += 8;
        }
    }
    while idx < len {
        dst[idx] ^= src[idx];
        idx += 1;
    }
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
