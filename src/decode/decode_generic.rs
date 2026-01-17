use crate::bitmap::Bitmap;
use crate::contexts::DecodingContext;
use crate::decode::decode_mmr::decode_mmr_bitmap;
use crate::error::Jbig2Error;
use crate::reader::Reader;
use crate::validation;

const REUSED_CONTEXTS: [u16; 4] = [
    0x9b25, // 10011 0110010 0101
    0x0795, // 0011 110010 101
    0x00e5, // 001 11001 01
    0x0195, // 011001 0101
];

pub fn get_coding_template(index: usize) -> &'static [(i8, i8)] {
    match index {
        0 => &[
            (-1, -2),
            (0, -2),
            (1, -2),
            (-2, -1),
            (-1, -1),
            (0, -1),
            (1, -1),
            (2, -1),
            (-4, 0),
            (-3, 0),
            (-2, 0),
            (-1, 0),
        ],
        1 => &[
            (-1, -2),
            (0, -2),
            (1, -2),
            (2, -2),
            (-2, -1),
            (-1, -1),
            (0, -1),
            (1, -1),
            (2, -1),
            (-3, 0),
            (-2, 0),
            (-1, 0),
        ],
        2 => &[
            (-1, -2),
            (0, -2),
            (1, -2),
            (-2, -1),
            (-1, -1),
            (0, -1),
            (1, -1),
            (-2, 0),
            (-1, 0),
        ],
        3 => &[
            (-3, -1),
            (-2, -1),
            (-1, -1),
            (0, -1),
            (1, -1),
            (-4, 0),
            (-3, 0),
            (-2, 0),
            (-1, 0),
        ],
        _ => &[],
    }
}

/// Decode a generic region using the default template 0 fast path.
fn decode_bitmap_template0(
    width: usize,
    height: usize,
    decoding_context: &mut DecodingContext,
) -> Result<Bitmap, Jbig2Error> {
    let mut decoder = decoding_context.get_decoder();
    let mut contexts = decoding_context.get_contexts("GB");
    let mut bitmap = Bitmap::new(width, height);
    if width == 0 || height == 0 {
        return Ok(bitmap);
    }
    let rowstride = bitmap.stride;
    let padded_width = (width + 7) & !7;
    for y in 0..height {
        let row_start = y * rowstride;
        let (before, after) = bitmap.data.split_at_mut(row_start);
        let (row, _) = after.split_at_mut(rowstride);
        let line1 = if y >= 1 {
            Some(&before[(y - 1) * rowstride..y * rowstride])
        } else {
            None
        };
        let line2 = if y >= 2 {
            Some(&before[(y - 2) * rowstride..(y - 1) * rowstride])
        } else {
            None
        };

        let mut line_m1 = line1.map_or(0u32, |l| l[0] as u32);
        let mut line_m2 = line2.map_or(0u32, |l| (l[0] as u32) << 6);
        let mut context = (line_m1 & 0x7f0) | (line_m2 & 0xf800);

        for x in (0..padded_width).step_by(8) {
            let minor_width = if width - x > 8 { 8 } else { width - x };

            if let Some(line1_row) = line1 {
                let next = if x + 8 < width {
                    line1_row[(x >> 3) + 1] as u32
                } else {
                    0
                };
                line_m1 = (line_m1 << 8) | next;
            }

            if let Some(line2_row) = line2 {
                let next = if x + 8 < width {
                    line2_row[(x >> 3) + 1] as u32
                } else {
                    0
                };
                line_m2 = (line_m2 << 8) | (next << 6);
            }

            let mut result = 0u8;
            for x_minor in 0..minor_width {
                let bit = decoder.read_bit(contexts.as_mut(), context as usize)?;
                result |= (bit as u8) << (7 - x_minor);
                let line_m1_bit = ((line_m1 >> (7 - x_minor)) & 0x10) as u32;
                let line_m2_bit = ((line_m2 >> (7 - x_minor)) & 0x800) as u32;
                context =
                    ((context & 0x7bf7) << 1) | (bit as u32) | line_m1_bit | line_m2_bit;
            }
            row[x >> 3] = result;
        }
    }
    Ok(bitmap)
}

/// Inputs required to decode a generic region bitmap.
#[derive(Clone)]
pub struct DecodeBitmapParams<'a> {
    pub mmr: bool,
    pub width: usize,
    pub height: usize,
    pub template_index: usize,
    pub prediction: bool,
    pub skip: Option<&'a Bitmap>,
    pub at: Vec<(i8, i8)>,
}

/// Decode a generic region bitmap using either MMR or arithmetic coding.
pub fn decode_bitmap(
    params: &DecodeBitmapParams,
    decoding_context: &mut DecodingContext,
) -> Result<Bitmap, Jbig2Error> {
    // Early return for zero dimensions.
    if params.width == 0 || params.height == 0 {
        return Ok(Bitmap::new(params.width, params.height));
    }

    // Validate parameters before decoding.
    validation::validate_generic_decode_params(params.width, params.height, params.template_index)?;

    if params.mmr {
        let mut reader = Reader::new(
            decoding_context.data.clone(),
            decoding_context.start,
            decoding_context.end,
        );
        let bitmap = decode_mmr_bitmap(&mut reader, params.width, params.height, true)?;
        decoding_context.start = reader.get_position();
        return Ok(bitmap);
    }

    // Use an optimized path for the common template-0 case.
    if params.template_index == 0
        && params.skip.is_none()
        && !params.prediction
        && params.at.len() == 4
        && params.at[0].0 == 3
        && params.at[0].1 == -1
        && params.at[1].0 == -3
        && params.at[1].1 == -1
        && params.at[2].0 == 2
        && params.at[2].1 == -2
        && params.at[3].0 == -2
        && params.at[3].1 == -2
    {
        return decode_bitmap_template0(params.width, params.height, decoding_context);
    }

    let useskip = params.skip.is_some();
    // Build and sort the context template for reuse decisions.
    let template = get_coding_template(params.template_index)
        .iter()
        .cloned()
        .chain(params.at.clone())
        .collect::<Vec<_>>();
    let mut template = template;
    template.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
    let template_length = template.len();
    debug_assert!(template_length <= 16);
    let mut template_x = vec![0i8; template_length];
    let mut template_y = vec![0i8; template_length];
    let mut changing_template_entries = vec![];
    let mut reuse_mask = 0u16;
    let mut min_x = i8::MAX;
    let mut max_x = i8::MIN;
    let mut min_y = i8::MAX;
    for k in 0..template_length {
        template_x[k] = template[k].0;
        template_y[k] = template[k].1;
        min_x = min_x.min(template[k].0);
        max_x = max_x.max(template[k].0);
        min_y = min_y.min(template[k].1);
        if k < template_length - 1
            && template[k].1 == template[k + 1].1
            && template[k].0 == template[k + 1].0 - 1
        {
            reuse_mask |= 1 << (template_length - 1 - k);
        } else {
            changing_template_entries.push(k);
        }
    }
    let changing_entries_length = changing_template_entries.len();
    let mut changing_template_x = vec![0i8; changing_entries_length];
    let mut changing_template_y = vec![0i8; changing_entries_length];
    let mut changing_template_bit = vec![0u16; changing_entries_length];
    for c in 0..changing_entries_length {
        let k = changing_template_entries[c];
        changing_template_x[c] = template[k].0;
        changing_template_y[c] = template[k].1;
        changing_template_bit[c] = 1 << (template_length - 1 - k);
    }
    let sbb_left = (-min_x) as usize;
    let sbb_top = (-min_y) as usize;
    let sbb_right = params.width.saturating_sub(max_x as usize);
    let pseudo_pixel_context = REUSED_CONTEXTS[params.template_index];
    let mut bitmap = Bitmap::new(params.width, params.height);
    let mut decoder = decoding_context.get_decoder();
    let mut contexts = decoding_context.get_contexts("GB");
    let width_i32 = params.width as i32;
    let height_i32 = params.height as i32;
    let skip_stride = params.skip.map_or(0, |skip| skip.stride);
    let empty_skip: &[u8] = &[];
    let skip_data = params.skip.map_or(empty_skip, |skip| skip.data.as_slice());
    let safe_start = sbb_left.min(params.width);
    let safe_end = sbb_right.min(params.width);
    let mut ltp = 0i32;
    for i in 0..params.height {
        if params.prediction {
            let sltp = decoder.read_bit(contexts.as_mut(), pseudo_pixel_context as usize)? as i32;
            ltp ^= sltp;
            if ltp != 0 {
                let dst_start = i * bitmap.stride;
                if i == 0 {
                    bitmap.data[dst_start..dst_start + bitmap.stride].fill(0);
                } else {
                    let src_start = (i - 1) * bitmap.stride;
                    let (before, after) = bitmap.data.split_at_mut(dst_start);
                    let src_row = &before[src_start..src_start + bitmap.stride];
                    let dst_row = &mut after[0..bitmap.stride];
                    dst_row.copy_from_slice(src_row);
                }
                continue;
            }
        }
        let mut context_label: u16;
        if i < sbb_top || safe_start >= safe_end {
            for j in 0..params.width {
                let mut full = 0u16;
                let mut shift = template_length as i32 - 1;
                for k in 0..template_length {
                    let j0 = j as i32 + template_x[k] as i32;
                    if j0 >= 0 && j0 < width_i32 {
                        let i0 = i as i32 + template_y[k] as i32;
                        if i0 >= 0
                            && i0 < height_i32
                            && bitmap.get_pixel_unchecked(j0 as usize, i0 as usize) != 0
                        {
                            full |= 1 << shift;
                        }
                    }
                    shift -= 1;
                }
                context_label = full;
                let skip_hit = if useskip {
                    let byte_index = i * skip_stride + (j >> 3);
                    let mask = 1u8 << (7 - (j & 7));
                    (skip_data[byte_index] & mask) != 0
                } else {
                    false
                };
                let pixel = if skip_hit {
                    0
                } else {
                    decoder.read_bit(contexts.as_mut(), context_label as usize)?
                };
                bitmap.set_pixel_unchecked(j, i, pixel);
            }
            continue;
        }

        for j in 0..safe_start {
            let mut full = 0u16;
            let mut shift = template_length as i32 - 1;
            for k in 0..template_length {
                let j0 = j as i32 + template_x[k] as i32;
                if j0 >= 0 && j0 < width_i32 {
                    let i0 = i as i32 + template_y[k] as i32;
                    if i0 >= 0
                        && i0 < height_i32
                        && bitmap.get_pixel_unchecked(j0 as usize, i0 as usize) != 0
                    {
                        full |= 1 << shift;
                    }
                }
                shift -= 1;
            }
            context_label = full;
            let skip_hit = if useskip {
                let byte_index = i * skip_stride + (j >> 3);
                let mask = 1u8 << (7 - (j & 7));
                (skip_data[byte_index] & mask) != 0
            } else {
                false
            };
            let pixel = if skip_hit {
                0
            } else {
                decoder.read_bit(contexts.as_mut(), context_label as usize)?
            };
            bitmap.set_pixel_unchecked(j, i, pixel);
        }

        {
            let j = safe_start;
            let mut full = 0u16;
            let mut shift = template_length as i32 - 1;
            for k in 0..template_length {
                let j0 = (j as i32 + template_x[k] as i32) as usize;
                let i0 = (i as i32 + template_y[k] as i32) as usize;
                if bitmap.get_pixel_unchecked(j0, i0) != 0 {
                    full |= 1 << shift;
                }
                shift -= 1;
            }
            context_label = full;
            let skip_hit = if useskip {
                let byte_index = i * skip_stride + (j >> 3);
                let mask = 1u8 << (7 - (j & 7));
                (skip_data[byte_index] & mask) != 0
            } else {
                false
            };
            let pixel = if skip_hit {
                0
            } else {
                decoder.read_bit(contexts.as_mut(), context_label as usize)?
            };
            bitmap.set_pixel_unchecked(j, i, pixel);
        }

        for j in (safe_start + 1)..safe_end {
            context_label = (context_label << 1) & reuse_mask;
            for k in 0..changing_entries_length {
                let i0 = (i as i32 + changing_template_y[k] as i32) as usize;
                let j0 = (j as i32 + changing_template_x[k] as i32) as usize;
                if bitmap.get_pixel_unchecked(j0, i0) != 0 {
                    context_label |= changing_template_bit[k];
                }
            }
            let skip_hit = if useskip {
                let byte_index = i * skip_stride + (j >> 3);
                let mask = 1u8 << (7 - (j & 7));
                (skip_data[byte_index] & mask) != 0
            } else {
                false
            };
            let pixel = if skip_hit {
                0
            } else {
                decoder.read_bit(contexts.as_mut(), context_label as usize)?
            };
            bitmap.set_pixel_unchecked(j, i, pixel);
        }

        for j in safe_end..params.width {
            let mut full = 0u16;
            let mut shift = template_length as i32 - 1;
            for k in 0..template_length {
                let j0 = j as i32 + template_x[k] as i32;
                if j0 >= 0 && j0 < width_i32 {
                    let i0 = i as i32 + template_y[k] as i32;
                    if i0 >= 0
                        && i0 < height_i32
                        && bitmap.get_pixel_unchecked(j0 as usize, i0 as usize) != 0
                    {
                        full |= 1 << shift;
                    }
                }
                shift -= 1;
            }
            context_label = full;
            let skip_hit = if useskip {
                let byte_index = i * skip_stride + (j >> 3);
                let mask = 1u8 << (7 - (j & 7));
                (skip_data[byte_index] & mask) != 0
            } else {
                false
            };
            let pixel = if skip_hit {
                0
            } else {
                decoder.read_bit(contexts.as_mut(), context_label as usize)?
            };
            bitmap.set_pixel_unchecked(j, i, pixel);
        }
    }
    Ok(bitmap)
}
