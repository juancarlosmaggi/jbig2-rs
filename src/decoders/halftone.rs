#![allow(clippy::too_many_arguments)]
use crate::bitmap::Bitmap;
use crate::bitmap::utils as bitmap_utils;
use crate::arithmetic::contexts::DecodingContext;
use crate::decoders::generic::{DecodeBitmapParams, decode_bitmap};
use crate::common::error::Jbig2Error;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

const BIT_MASKS: [u8; 8] = [0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01];
const SHIFTED_PATTERN_CACHE_LIMIT: usize = 32;

thread_local! {
    static SHIFTED_PATTERN_CACHE: RefCell<HashMap<u64, Arc<Vec<ShiftedPattern>>>> =
        RefCell::new(HashMap::new());
}

struct ShiftedRows {
    stride: usize,
    data: Vec<u8>,
}

pub(crate) struct ShiftedPattern {
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
    if !has_black {
        let shifts = std::array::from_fn(|_| ShiftedRows {
            stride: 0,
            data: Vec::new(),
        });
        return ShiftedPattern { shifts, has_black };
    }
    let shifts = std::array::from_fn(|shift| build_shifted_rows(pattern, shift));
    ShiftedPattern { shifts, has_black }
}

pub(crate) fn build_shifted_patterns(patterns: &[Bitmap]) -> Arc<Vec<ShiftedPattern>> {
    Arc::new(patterns.iter().map(build_shifted_pattern).collect())
}

fn compute_patterns_hash(patterns: &[Bitmap]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    patterns.len().hash(&mut hasher);
    for pattern in patterns {
        pattern.width.hash(&mut hasher);
        pattern.height.hash(&mut hasher);
        pattern.stride.hash(&mut hasher);
        pattern.data.len().hash(&mut hasher);
        pattern.data.hash(&mut hasher);
    }
    hasher.finish()
}

fn get_shifted_patterns(patterns: &[Bitmap]) -> Arc<Vec<ShiftedPattern>> {
    let hash = compute_patterns_hash(patterns);
    SHIFTED_PATTERN_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(cached) = cache.get(&hash) {
            return Arc::clone(cached);
        }
        let shifted = build_shifted_patterns(patterns);
        if cache.len() >= SHIFTED_PATTERN_CACHE_LIMIT {
            cache.clear();
        }
        cache.insert(hash, Arc::clone(&shifted));
        shifted
    })
}

#[derive(Clone, Copy)]
struct HalftoneLayout {
    grid_width: usize,
    grid_height: usize,
    grid_vector_x: i64,
    grid_vector_y: i64,
    grid_offset_x: i64,
    grid_offset_y: i64,
    pattern_height_usize: usize,
    pattern_width: i64,
    pattern_height: i64,
    region_width: i64,
    region_height: i64,
}

#[inline(always)]
unsafe fn or_row_bytes_ptr(dst: *mut u8, src: *const u8, len: usize) {
    let mut idx = 0usize;
    unsafe {
        while idx + 8 <= len {
            let dst_ptr = dst.add(idx) as *mut u64;
            let src_ptr = src.add(idx) as *const u64;
            let dst_val = std::ptr::read_unaligned(dst_ptr);
            let src_val = std::ptr::read_unaligned(src_ptr);
            std::ptr::write_unaligned(dst_ptr, dst_val | src_val);
            idx += 8;
        }
        while idx < len {
            let dst_byte = dst.add(idx);
            *dst_byte |= *src.add(idx);
            idx += 1;
        }
    }
}

#[inline(always)]
fn place_halftone_pattern<const INSIDE: bool>(
    region_bitmap: &mut Bitmap,
    shifted_patterns: &[ShiftedPattern],
    patterns: &[Bitmap],
    layout: &HalftoneLayout,
    pattern_index: usize,
    x: i64,
    y: i64,
) {
    let shifted_pattern = &shifted_patterns[pattern_index];
    if !shifted_pattern.has_black {
        return;
    }
    let region_x = x >> 8;
    let region_y = y >> 8;
    let inside = if INSIDE {
        true
    } else {
        if region_x + layout.pattern_width <= 0
            || region_x >= layout.region_width
            || region_y + layout.pattern_height <= 0
            || region_y >= layout.region_height
        {
            return;
        }
        region_x >= 0
            && region_y >= 0
            && region_x + layout.pattern_width <= layout.region_width
            && region_y + layout.pattern_height <= layout.region_height
    };
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
        let mut dst_row_start = region_y_u * dst_stride + dst_byte_offset;
        let mut src_row_start = 0usize;
        unsafe {
            let dst_ptr = dst_data.as_mut_ptr();
            let src_ptr = src_data.as_ptr();
            for _ in 0..layout.pattern_height_usize {
                or_row_bytes_ptr(dst_ptr.add(dst_row_start), src_ptr.add(src_row_start), src_stride);
                dst_row_start += dst_stride;
                src_row_start += src_stride;
            }
        }
    } else {
        let pattern_bitmap = &patterns[pattern_index];
        region_bitmap.combine_or(pattern_bitmap, region_x as isize, region_y as isize);
    }
}

#[inline(always)]
fn render_halftone_grid<const INSIDE: bool>(
    region_bitmap: &mut Bitmap,
    shifted_patterns: &[ShiftedPattern],
    patterns: &[Bitmap],
    gray_scale_bit_planes: &[Bitmap],
    bits_per_value: usize,
    layout: &HalftoneLayout,
) {
    let total_bytes = (layout.grid_width + 7) >> 3;
    let tail_bits = layout.grid_width & 7;
    let full_bytes = total_bytes.saturating_sub(usize::from(tail_bits != 0));
    let needs_clamp = patterns.len() != (1usize << bits_per_value);
    let max_pattern_index = patterns.len().saturating_sub(1);

    match bits_per_value {
        0 => render_halftone_grid_b0::<INSIDE>(
            region_bitmap,
            shifted_patterns,
            patterns,
            layout,
            full_bytes,
            tail_bits,
        ),
        1 => {
            if needs_clamp {
                render_halftone_grid_b1::<INSIDE, true>(
                    region_bitmap,
                    shifted_patterns,
                    patterns,
                    &gray_scale_bit_planes[0].data,
                    gray_scale_bit_planes[0].stride,
                    layout,
                    full_bytes,
                    tail_bits,
                    max_pattern_index,
                )
            } else {
                render_halftone_grid_b1::<INSIDE, false>(
                    region_bitmap,
                    shifted_patterns,
                    patterns,
                    &gray_scale_bit_planes[0].data,
                    gray_scale_bit_planes[0].stride,
                    layout,
                    full_bytes,
                    tail_bits,
                    max_pattern_index,
                )
            }
        }
        2 => {
            if needs_clamp {
                render_halftone_grid_b2::<INSIDE, true>(
                    region_bitmap,
                    shifted_patterns,
                    patterns,
                    &gray_scale_bit_planes[0].data,
                    &gray_scale_bit_planes[1].data,
                    gray_scale_bit_planes[0].stride,
                    layout,
                    full_bytes,
                    tail_bits,
                    max_pattern_index,
                )
            } else {
                render_halftone_grid_b2::<INSIDE, false>(
                    region_bitmap,
                    shifted_patterns,
                    patterns,
                    &gray_scale_bit_planes[0].data,
                    &gray_scale_bit_planes[1].data,
                    gray_scale_bit_planes[0].stride,
                    layout,
                    full_bytes,
                    tail_bits,
                    max_pattern_index,
                )
            }
        }
        3 => {
            if needs_clamp {
                render_halftone_grid_b3::<INSIDE, true>(
                    region_bitmap,
                    shifted_patterns,
                    patterns,
                    &gray_scale_bit_planes[0].data,
                    &gray_scale_bit_planes[1].data,
                    &gray_scale_bit_planes[2].data,
                    gray_scale_bit_planes[0].stride,
                    layout,
                    full_bytes,
                    tail_bits,
                    max_pattern_index,
                )
            } else {
                render_halftone_grid_b3::<INSIDE, false>(
                    region_bitmap,
                    shifted_patterns,
                    patterns,
                    &gray_scale_bit_planes[0].data,
                    &gray_scale_bit_planes[1].data,
                    &gray_scale_bit_planes[2].data,
                    gray_scale_bit_planes[0].stride,
                    layout,
                    full_bytes,
                    tail_bits,
                    max_pattern_index,
                )
            }
        }
        4 => {
            if needs_clamp {
                render_halftone_grid_b4::<INSIDE, true>(
                    region_bitmap,
                    shifted_patterns,
                    patterns,
                    &gray_scale_bit_planes[0].data,
                    &gray_scale_bit_planes[1].data,
                    &gray_scale_bit_planes[2].data,
                    &gray_scale_bit_planes[3].data,
                    gray_scale_bit_planes[0].stride,
                    layout,
                    full_bytes,
                    tail_bits,
                    max_pattern_index,
                )
            } else {
                render_halftone_grid_b4::<INSIDE, false>(
                    region_bitmap,
                    shifted_patterns,
                    patterns,
                    &gray_scale_bit_planes[0].data,
                    &gray_scale_bit_planes[1].data,
                    &gray_scale_bit_planes[2].data,
                    &gray_scale_bit_planes[3].data,
                    gray_scale_bit_planes[0].stride,
                    layout,
                    full_bytes,
                    tail_bits,
                    max_pattern_index,
                )
            }
        }
        _ => {
            if needs_clamp {
                render_halftone_grid_bn::<INSIDE, true>(
                    region_bitmap,
                    shifted_patterns,
                    patterns,
                    gray_scale_bit_planes,
                    bits_per_value,
                    gray_scale_bit_planes
                        .first()
                        .map(|plane| plane.stride)
                        .unwrap_or(0),
                    layout,
                    full_bytes,
                    tail_bits,
                    max_pattern_index,
                )
            } else {
                render_halftone_grid_bn::<INSIDE, false>(
                    region_bitmap,
                    shifted_patterns,
                    patterns,
                    gray_scale_bit_planes,
                    bits_per_value,
                    gray_scale_bit_planes
                        .first()
                        .map(|plane| plane.stride)
                        .unwrap_or(0),
                    layout,
                    full_bytes,
                    tail_bits,
                    max_pattern_index,
                )
            }
        }
    }
}

#[inline(always)]
fn render_halftone_grid_b0<const INSIDE: bool>(
    region_bitmap: &mut Bitmap,
    shifted_patterns: &[ShiftedPattern],
    patterns: &[Bitmap],
    layout: &HalftoneLayout,
    full_bytes: usize,
    tail_bits: usize,
) {
    let mut base_x = layout.grid_offset_x;
    let mut base_y = layout.grid_offset_y;
    for _ in 0..layout.grid_height {
        let mut x = base_x;
        let mut y = base_y;
        for _ in 0..full_bytes {
            for _ in 0..8 {
                place_halftone_pattern::<INSIDE>(
                    region_bitmap,
                    shifted_patterns,
                    patterns,
                    layout,
                    0,
                    x,
                    y,
                );
                x += layout.grid_vector_x;
                y -= layout.grid_vector_y;
            }
        }
        if tail_bits != 0 {
            for _ in 0..tail_bits {
                place_halftone_pattern::<INSIDE>(
                    region_bitmap,
                    shifted_patterns,
                    patterns,
                    layout,
                    0,
                    x,
                    y,
                );
                x += layout.grid_vector_x;
                y -= layout.grid_vector_y;
            }
        }
        base_x += layout.grid_vector_y;
        base_y += layout.grid_vector_x;
    }
}

#[inline(always)]
fn render_halftone_grid_b1<const INSIDE: bool, const CLAMP: bool>(
    region_bitmap: &mut Bitmap,
    shifted_patterns: &[ShiftedPattern],
    patterns: &[Bitmap],
    plane0: &[u8],
    plane_stride: usize,
    layout: &HalftoneLayout,
    full_bytes: usize,
    tail_bits: usize,
    max_pattern_index: usize,
) {
    let mut base_x = layout.grid_offset_x;
    let mut base_y = layout.grid_offset_y;
    let mut row_offset = 0usize;
    for _ in 0..layout.grid_height {
        let mut x = base_x;
        let mut y = base_y;
        let plane0_row = &plane0[row_offset..];
        for p0_byte in plane0_row.iter().take(full_bytes) {
            let mut p0 = *p0_byte;
            for _ in 0..8 {
                let mut pattern_index = (p0 >> 7) as usize;
                if CLAMP && pattern_index > max_pattern_index {
                    pattern_index = max_pattern_index;
                }
                place_halftone_pattern::<INSIDE>(
                    region_bitmap,
                    shifted_patterns,
                    patterns,
                    layout,
                    pattern_index,
                    x,
                    y,
                );
                p0 <<= 1;
                x += layout.grid_vector_x;
                y -= layout.grid_vector_y;
            }
        }
        if tail_bits != 0 {
            let mut p0 = plane0_row[full_bytes];
            for _ in 0..tail_bits {
                let mut pattern_index = (p0 >> 7) as usize;
                if CLAMP && pattern_index > max_pattern_index {
                    pattern_index = max_pattern_index;
                }
                place_halftone_pattern::<INSIDE>(
                    region_bitmap,
                    shifted_patterns,
                    patterns,
                    layout,
                    pattern_index,
                    x,
                    y,
                );
                p0 <<= 1;
                x += layout.grid_vector_x;
                y -= layout.grid_vector_y;
            }
        }
        base_x += layout.grid_vector_y;
        base_y += layout.grid_vector_x;
        row_offset += plane_stride;
    }
}

#[inline(always)]
fn render_halftone_grid_b2<const INSIDE: bool, const CLAMP: bool>(
    region_bitmap: &mut Bitmap,
    shifted_patterns: &[ShiftedPattern],
    patterns: &[Bitmap],
    plane0: &[u8],
    plane1: &[u8],
    plane_stride: usize,
    layout: &HalftoneLayout,
    full_bytes: usize,
    tail_bits: usize,
    max_pattern_index: usize,
) {
    let mut base_x = layout.grid_offset_x;
    let mut base_y = layout.grid_offset_y;
    let mut row_offset = 0usize;
    for _ in 0..layout.grid_height {
        let mut x = base_x;
        let mut y = base_y;
        let plane0_row = &plane0[row_offset..];
        let plane1_row = &plane1[row_offset..];
        for byte_index in 0..full_bytes {
            let mut p0 = plane0_row[byte_index];
            let mut p1 = plane1_row[byte_index];
            for _ in 0..8 {
                let mut pattern_index = ((p0 >> 7) | ((p1 >> 7) << 1)) as usize;
                if CLAMP && pattern_index > max_pattern_index {
                    pattern_index = max_pattern_index;
                }
                place_halftone_pattern::<INSIDE>(
                    region_bitmap,
                    shifted_patterns,
                    patterns,
                    layout,
                    pattern_index,
                    x,
                    y,
                );
                p0 <<= 1;
                p1 <<= 1;
                x += layout.grid_vector_x;
                y -= layout.grid_vector_y;
            }
        }
        if tail_bits != 0 {
            let mut p0 = plane0_row[full_bytes];
            let mut p1 = plane1_row[full_bytes];
            for _ in 0..tail_bits {
                let mut pattern_index = ((p0 >> 7) | ((p1 >> 7) << 1)) as usize;
                if CLAMP && pattern_index > max_pattern_index {
                    pattern_index = max_pattern_index;
                }
                place_halftone_pattern::<INSIDE>(
                    region_bitmap,
                    shifted_patterns,
                    patterns,
                    layout,
                    pattern_index,
                    x,
                    y,
                );
                p0 <<= 1;
                p1 <<= 1;
                x += layout.grid_vector_x;
                y -= layout.grid_vector_y;
            }
        }
        base_x += layout.grid_vector_y;
        base_y += layout.grid_vector_x;
        row_offset += plane_stride;
    }
}

#[inline(always)]
fn render_halftone_grid_b3<const INSIDE: bool, const CLAMP: bool>(
    region_bitmap: &mut Bitmap,
    shifted_patterns: &[ShiftedPattern],
    patterns: &[Bitmap],
    plane0: &[u8],
    plane1: &[u8],
    plane2: &[u8],
    plane_stride: usize,
    layout: &HalftoneLayout,
    full_bytes: usize,
    tail_bits: usize,
    max_pattern_index: usize,
) {
    let mut base_x = layout.grid_offset_x;
    let mut base_y = layout.grid_offset_y;
    let mut row_offset = 0usize;
    for _ in 0..layout.grid_height {
        let mut x = base_x;
        let mut y = base_y;
        let plane0_row = &plane0[row_offset..];
        let plane1_row = &plane1[row_offset..];
        let plane2_row = &plane2[row_offset..];
        for byte_index in 0..full_bytes {
            let mut p0 = plane0_row[byte_index];
            let mut p1 = plane1_row[byte_index];
            let mut p2 = plane2_row[byte_index];
            for _ in 0..8 {
                let mut pattern_index =
                    ((p0 >> 7) | ((p1 >> 7) << 1) | ((p2 >> 7) << 2)) as usize;
                if CLAMP && pattern_index > max_pattern_index {
                    pattern_index = max_pattern_index;
                }
                place_halftone_pattern::<INSIDE>(
                    region_bitmap,
                    shifted_patterns,
                    patterns,
                    layout,
                    pattern_index,
                    x,
                    y,
                );
                p0 <<= 1;
                p1 <<= 1;
                p2 <<= 1;
                x += layout.grid_vector_x;
                y -= layout.grid_vector_y;
            }
        }
        if tail_bits != 0 {
            let mut p0 = plane0_row[full_bytes];
            let mut p1 = plane1_row[full_bytes];
            let mut p2 = plane2_row[full_bytes];
            for _ in 0..tail_bits {
                let mut pattern_index =
                    ((p0 >> 7) | ((p1 >> 7) << 1) | ((p2 >> 7) << 2)) as usize;
                if CLAMP && pattern_index > max_pattern_index {
                    pattern_index = max_pattern_index;
                }
                place_halftone_pattern::<INSIDE>(
                    region_bitmap,
                    shifted_patterns,
                    patterns,
                    layout,
                    pattern_index,
                    x,
                    y,
                );
                p0 <<= 1;
                p1 <<= 1;
                p2 <<= 1;
                x += layout.grid_vector_x;
                y -= layout.grid_vector_y;
            }
        }
        base_x += layout.grid_vector_y;
        base_y += layout.grid_vector_x;
        row_offset += plane_stride;
    }
}

#[inline(always)]
fn render_halftone_grid_b4<const INSIDE: bool, const CLAMP: bool>(
    region_bitmap: &mut Bitmap,
    shifted_patterns: &[ShiftedPattern],
    patterns: &[Bitmap],
    plane0: &[u8],
    plane1: &[u8],
    plane2: &[u8],
    plane3: &[u8],
    plane_stride: usize,
    layout: &HalftoneLayout,
    full_bytes: usize,
    tail_bits: usize,
    max_pattern_index: usize,
) {
    let mut base_x = layout.grid_offset_x;
    let mut base_y = layout.grid_offset_y;
    let mut row_offset = 0usize;
    for _ in 0..layout.grid_height {
        let mut x = base_x;
        let mut y = base_y;
        let plane0_row = &plane0[row_offset..];
        let plane1_row = &plane1[row_offset..];
        let plane2_row = &plane2[row_offset..];
        let plane3_row = &plane3[row_offset..];
        for byte_index in 0..full_bytes {
            let mut p0 = plane0_row[byte_index];
            let mut p1 = plane1_row[byte_index];
            let mut p2 = plane2_row[byte_index];
            let mut p3 = plane3_row[byte_index];
            for _ in 0..8 {
                let mut pattern_index = ((p0 >> 7)
                    | ((p1 >> 7) << 1)
                    | ((p2 >> 7) << 2)
                    | ((p3 >> 7) << 3)) as usize;
                if CLAMP && pattern_index > max_pattern_index {
                    pattern_index = max_pattern_index;
                }
                place_halftone_pattern::<INSIDE>(
                    region_bitmap,
                    shifted_patterns,
                    patterns,
                    layout,
                    pattern_index,
                    x,
                    y,
                );
                p0 <<= 1;
                p1 <<= 1;
                p2 <<= 1;
                p3 <<= 1;
                x += layout.grid_vector_x;
                y -= layout.grid_vector_y;
            }
        }
        if tail_bits != 0 {
            let mut p0 = plane0_row[full_bytes];
            let mut p1 = plane1_row[full_bytes];
            let mut p2 = plane2_row[full_bytes];
            let mut p3 = plane3_row[full_bytes];
            for _ in 0..tail_bits {
                let mut pattern_index = ((p0 >> 7)
                    | ((p1 >> 7) << 1)
                    | ((p2 >> 7) << 2)
                    | ((p3 >> 7) << 3)) as usize;
                if CLAMP && pattern_index > max_pattern_index {
                    pattern_index = max_pattern_index;
                }
                place_halftone_pattern::<INSIDE>(
                    region_bitmap,
                    shifted_patterns,
                    patterns,
                    layout,
                    pattern_index,
                    x,
                    y,
                );
                p0 <<= 1;
                p1 <<= 1;
                p2 <<= 1;
                p3 <<= 1;
                x += layout.grid_vector_x;
                y -= layout.grid_vector_y;
            }
        }
        base_x += layout.grid_vector_y;
        base_y += layout.grid_vector_x;
        row_offset += plane_stride;
    }
}

#[inline(always)]
fn render_halftone_grid_bn<const INSIDE: bool, const CLAMP: bool>(
    region_bitmap: &mut Bitmap,
    shifted_patterns: &[ShiftedPattern],
    patterns: &[Bitmap],
    gray_scale_bit_planes: &[Bitmap],
    bits_per_value: usize,
    plane_stride: usize,
    layout: &HalftoneLayout,
    full_bytes: usize,
    tail_bits: usize,
    max_pattern_index: usize,
) {
    let mut base_x = layout.grid_offset_x;
    let mut base_y = layout.grid_offset_y;
    let mut row_offset = 0usize;
    for _ in 0..layout.grid_height {
        let mut x = base_x;
        let mut y = base_y;
        for byte_index in 0..full_bytes {
            for bit_mask in &BIT_MASKS {
                let mut pattern_index = 0usize;
                for (j, plane) in gray_scale_bit_planes.iter().enumerate().take(bits_per_value) {
                    if (plane.data[row_offset + byte_index] & bit_mask) != 0 {
                        pattern_index |= 1usize << j;
                    }
                }
                if CLAMP && pattern_index > max_pattern_index {
                    pattern_index = max_pattern_index;
                }
                place_halftone_pattern::<INSIDE>(
                    region_bitmap,
                    shifted_patterns,
                    patterns,
                    layout,
                    pattern_index,
                    x,
                    y,
                );
                x += layout.grid_vector_x;
                y -= layout.grid_vector_y;
            }
        }
        if tail_bits != 0 {
            let byte_index = full_bytes;
            for bit_mask in BIT_MASKS.iter().take(tail_bits) {
                let mut pattern_index = 0usize;
                for (j, plane) in gray_scale_bit_planes.iter().enumerate().take(bits_per_value) {
                    if (plane.data[row_offset + byte_index] & bit_mask) != 0 {
                        pattern_index |= 1usize << j;
                    }
                }
                if CLAMP && pattern_index > max_pattern_index {
                    pattern_index = max_pattern_index;
                }
                place_halftone_pattern::<INSIDE>(
                    region_bitmap,
                    shifted_patterns,
                    patterns,
                    layout,
                    pattern_index,
                    x,
                    y,
                );
                x += layout.grid_vector_x;
                y -= layout.grid_vector_y;
            }
        }
        base_x += layout.grid_vector_y;
        base_y += layout.grid_vector_x;
        row_offset += plane_stride;
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
    decode_halftone_region_with_shifted(params, None, decoding_context)
}

pub(crate) fn decode_halftone_region_with_shifted(
    params: &HalftoneRegionParams<'_>,
    shifted_patterns: Option<Arc<Vec<ShiftedPattern>>>,
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
    let bits_per_value = crate::common::utils::log2(number_of_patterns as u32) as usize;
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
    let grid_inside = params.grid_width > 0
        && params.grid_height > 0
        && grid_fully_inside_region(
            params,
            params.region_width as i64,
            params.region_height as i64,
            pattern_width,
            pattern_height,
        );
    // Build a skip bitmap from the grid geometry when enabled.
    let skip_bitmap = if params.enable_skip && !params.mmr {
        let region_width = params.region_width as i64;
        let region_height = params.region_height as i64;
        if params.grid_width == 0 || params.grid_height == 0 || grid_inside {
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
    let shifted_patterns = shifted_patterns.unwrap_or_else(|| get_shifted_patterns(params.patterns));
    let region_width = params.region_width as i64;
    let region_height = params.region_height as i64;

    let layout = HalftoneLayout {
        grid_width: params.grid_width,
        grid_height: params.grid_height,
        grid_vector_x: params.grid_vector_x as i64,
        grid_vector_y: params.grid_vector_y as i64,
        grid_offset_x: params.grid_offset_x as i64,
        grid_offset_y: params.grid_offset_y as i64,
        pattern_height_usize,
        pattern_width,
        pattern_height,
        region_width,
        region_height,
    };
    if grid_inside {
        render_halftone_grid::<true>(
            &mut region_bitmap,
            shifted_patterns.as_ref(),
            params.patterns,
            &gray_scale_bit_planes,
            bits_per_value,
            &layout,
        );
    } else {
        render_halftone_grid::<false>(
            &mut region_bitmap,
            shifted_patterns.as_ref(),
            params.patterns,
            &gray_scale_bit_planes,
            bits_per_value,
            &layout,
        );
    }
    Ok(region_bitmap)
}

#[inline(always)]
fn xor_plane_bytes(dst: &mut [u8], src: &[u8]) {
    debug_assert_eq!(dst.len(), src.len());
    let len = dst.len();
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
