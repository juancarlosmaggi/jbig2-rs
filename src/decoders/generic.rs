use crate::arithmetic::contexts::DecodingContext;
use crate::bitmap::Bitmap;
use crate::common::error::Jbig2Error;
use crate::common::reader::Reader;
use crate::common::validation;
use crate::decoders::mmr::decode_mmr_bitmap;

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
    decoding_context: &mut DecodingContext<'_>,
) -> Result<Bitmap, Jbig2Error> {
    let mut decoder = decoding_context.get_decoder();
    let mut contexts = decoding_context.get_contexts("GB");
    let contexts = contexts.as_mut();
    // SAFETY: We use an uninitialized bitmap because this function guarantees
    // that every byte of the bitmap (including padding bits in the last byte of each row)
    // is overwritten before it is read.
    // The decoder writes byte-by-byte: `row[x >> 3] = result`.
    // Padding bits in `result` are zeroed.
    // Stride is equal to `padded_width / 8`, so there are no stride padding bytes.
    let mut bitmap = unsafe { Bitmap::uninit(width, height) };
    if width == 0 || height == 0 {
        return Ok(bitmap);
    }
    let rowstride = bitmap.stride;
    let padded_width = (width + 7) & !7;
    for y in 0..height {
        let row_start = y * rowstride;
        let (before, after) = bitmap.data.split_at_mut(row_start);
        let (row, _) = after.split_at_mut(rowstride);
        if y == 0 {
            let mut context = 0u32;
            for x in (0..padded_width).step_by(8) {
                let minor_width = if width - x > 8 { 8 } else { width - x };
                let mut result = 0u8;
                if minor_width == 8 {
                    // Unrolled loop for the common case.
                    let bit = decoder.read_bit(contexts, context as usize);
                    result |= bit << 7;
                    context = ((context & 0x7bf7) << 1) | (bit as u32);

                    let bit = decoder.read_bit(contexts, context as usize);
                    result |= bit << 6;
                    context = ((context & 0x7bf7) << 1) | (bit as u32);

                    let bit = decoder.read_bit(contexts, context as usize);
                    result |= bit << 5;
                    context = ((context & 0x7bf7) << 1) | (bit as u32);

                    let bit = decoder.read_bit(contexts, context as usize);
                    result |= bit << 4;
                    context = ((context & 0x7bf7) << 1) | (bit as u32);

                    let bit = decoder.read_bit(contexts, context as usize);
                    result |= bit << 3;
                    context = ((context & 0x7bf7) << 1) | (bit as u32);

                    let bit = decoder.read_bit(contexts, context as usize);
                    result |= bit << 2;
                    context = ((context & 0x7bf7) << 1) | (bit as u32);

                    let bit = decoder.read_bit(contexts, context as usize);
                    result |= bit << 1;
                    context = ((context & 0x7bf7) << 1) | (bit as u32);

                    let bit = decoder.read_bit(contexts, context as usize);
                    result |= bit;
                    context = ((context & 0x7bf7) << 1) | (bit as u32);
                } else {
                    for x_minor in 0..minor_width {
                        let bit = decoder.read_bit(contexts, context as usize);
                        result |= bit << (7 - x_minor);
                        context = ((context & 0x7bf7) << 1) | (bit as u32);
                    }
                }
                row[x >> 3] = result;
            }
            continue;
        }

        let line1_row = &before[(y - 1) * rowstride..y * rowstride];
        let mut line_m1 = line1_row[0] as u32;

        if y == 1 {
            let mut context = line_m1 & 0x7f0;
            for x in (0..padded_width).step_by(8) {
                let minor_width = if width - x > 8 { 8 } else { width - x };
                let next = if x + 8 < width {
                    line1_row[(x >> 3) + 1] as u32
                } else {
                    0
                };
                line_m1 = (line_m1 << 8) | next;

                let mut result = 0u8;
                if minor_width == 8 {
                    // Unrolled loop for the common full-byte case.
                    let bit = decoder.read_bit(contexts, context as usize);
                    result |= bit << 7;
                    context = ((context & 0x7bf7) << 1) | (bit as u32) | ((line_m1 >> 7) & 0x10);

                    let bit = decoder.read_bit(contexts, context as usize);
                    result |= bit << 6;
                    context = ((context & 0x7bf7) << 1) | (bit as u32) | ((line_m1 >> 6) & 0x10);

                    let bit = decoder.read_bit(contexts, context as usize);
                    result |= bit << 5;
                    context = ((context & 0x7bf7) << 1) | (bit as u32) | ((line_m1 >> 5) & 0x10);

                    let bit = decoder.read_bit(contexts, context as usize);
                    result |= bit << 4;
                    context = ((context & 0x7bf7) << 1) | (bit as u32) | ((line_m1 >> 4) & 0x10);

                    let bit = decoder.read_bit(contexts, context as usize);
                    result |= bit << 3;
                    context = ((context & 0x7bf7) << 1) | (bit as u32) | ((line_m1 >> 3) & 0x10);

                    let bit = decoder.read_bit(contexts, context as usize);
                    result |= bit << 2;
                    context = ((context & 0x7bf7) << 1) | (bit as u32) | ((line_m1 >> 2) & 0x10);

                    let bit = decoder.read_bit(contexts, context as usize);
                    result |= bit << 1;
                    context = ((context & 0x7bf7) << 1) | (bit as u32) | ((line_m1 >> 1) & 0x10);

                    let bit = decoder.read_bit(contexts, context as usize);
                    result |= bit;
                    context = ((context & 0x7bf7) << 1) | (bit as u32) | (line_m1 & 0x10);
                } else {
                    for x_minor in 0..minor_width {
                        let bit = decoder.read_bit(contexts, context as usize);
                        result |= bit << (7 - x_minor);
                        let line_m1_bit = (line_m1 >> (7 - x_minor)) & 0x10;
                        context = ((context & 0x7bf7) << 1) | (bit as u32) | line_m1_bit;
                    }
                }
                row[x >> 3] = result;
            }
            continue;
        }

        let line2_row = &before[(y - 2) * rowstride..(y - 1) * rowstride];
        let mut line_m2 = (line2_row[0] as u32) << 6;
        let mut context = (line_m1 & 0x7f0) | (line_m2 & 0xf800);

        for x in (0..padded_width).step_by(8) {
            let minor_width = if width - x > 8 { 8 } else { width - x };
            let next1 = if x + 8 < width {
                line1_row[(x >> 3) + 1] as u32
            } else {
                0
            };
            line_m1 = (line_m1 << 8) | next1;
            let next2 = if x + 8 < width {
                line2_row[(x >> 3) + 1] as u32
            } else {
                0
            };
            line_m2 = (line_m2 << 8) | (next2 << 6);

            let mut result = 0u8;
            if minor_width == 8 {
                // Unrolled loop for the common full-byte case (hot path for tall pages).
                let bit = decoder.read_bit(contexts, context as usize);
                result |= bit << 7;
                context = ((context & 0x7bf7) << 1)
                    | (bit as u32)
                    | ((line_m1 >> 7) & 0x10)
                    | ((line_m2 >> 7) & 0x800);

                let bit = decoder.read_bit(contexts, context as usize);
                result |= bit << 6;
                context = ((context & 0x7bf7) << 1)
                    | (bit as u32)
                    | ((line_m1 >> 6) & 0x10)
                    | ((line_m2 >> 6) & 0x800);

                let bit = decoder.read_bit(contexts, context as usize);
                result |= bit << 5;
                context = ((context & 0x7bf7) << 1)
                    | (bit as u32)
                    | ((line_m1 >> 5) & 0x10)
                    | ((line_m2 >> 5) & 0x800);

                let bit = decoder.read_bit(contexts, context as usize);
                result |= bit << 4;
                context = ((context & 0x7bf7) << 1)
                    | (bit as u32)
                    | ((line_m1 >> 4) & 0x10)
                    | ((line_m2 >> 4) & 0x800);

                let bit = decoder.read_bit(contexts, context as usize);
                result |= bit << 3;
                context = ((context & 0x7bf7) << 1)
                    | (bit as u32)
                    | ((line_m1 >> 3) & 0x10)
                    | ((line_m2 >> 3) & 0x800);

                let bit = decoder.read_bit(contexts, context as usize);
                result |= bit << 2;
                context = ((context & 0x7bf7) << 1)
                    | (bit as u32)
                    | ((line_m1 >> 2) & 0x10)
                    | ((line_m2 >> 2) & 0x800);

                let bit = decoder.read_bit(contexts, context as usize);
                result |= bit << 1;
                context = ((context & 0x7bf7) << 1)
                    | (bit as u32)
                    | ((line_m1 >> 1) & 0x10)
                    | ((line_m2 >> 1) & 0x800);

                let bit = decoder.read_bit(contexts, context as usize);
                result |= bit;
                context =
                    ((context & 0x7bf7) << 1) | (bit as u32) | (line_m1 & 0x10) | (line_m2 & 0x800);
            } else {
                for x_minor in 0..minor_width {
                    let bit = decoder.read_bit(contexts, context as usize);
                    result |= bit << (7 - x_minor);
                    let line_m1_bit = (line_m1 >> (7 - x_minor)) & 0x10;
                    let line_m2_bit = (line_m2 >> (7 - x_minor)) & 0x800;
                    context = ((context & 0x7bf7) << 1) | (bit as u32) | line_m1_bit | line_m2_bit;
                }
            }
            row[x >> 3] = result;
        }
    }
    Ok(bitmap)
}

/// Decode a generic region using template 0 with a skip bitmap.
fn decode_bitmap_template0_with_skip(
    width: usize,
    height: usize,
    skip: &Bitmap,
    decoding_context: &mut DecodingContext<'_>,
) -> Result<Bitmap, Jbig2Error> {
    let mut decoder = decoding_context.get_decoder();
    let mut contexts = decoding_context.get_contexts("GB");
    let contexts = contexts.as_mut();
    // SAFETY: We use an uninitialized bitmap because this function guarantees
    // that every byte of the bitmap (including padding bits in the last byte of each row)
    // is overwritten before it is read.
    // The decoder writes byte-by-byte: `row[x >> 3] = result`.
    // Padding bits in `result` are zeroed.
    // Stride is equal to `padded_width / 8`, so there are no stride padding bytes.
    let mut bitmap = unsafe { Bitmap::uninit(width, height) };
    if width == 0 || height == 0 {
        return Ok(bitmap);
    }
    let rowstride = bitmap.stride;
    debug_assert_eq!(skip.stride, rowstride);
    let padded_width = (width + 7) & !7;
    for y in 0..height {
        let row_start = y * rowstride;
        let (before, after) = bitmap.data.split_at_mut(row_start);
        let (row, _) = after.split_at_mut(rowstride);
        let skip_row_start = y * skip.stride;
        let skip_row = &skip.data[skip_row_start..skip_row_start + skip.stride];

        if y == 0 {
            let mut context = 0u32;
            for x in (0..padded_width).step_by(8) {
                let minor_width = if width - x > 8 { 8 } else { width - x };
                let skip_byte = skip_row[x >> 3];
                if skip_byte == 0 {
                    let mut result = 0u8;
                    for x_minor in 0..minor_width {
                        let bit = decoder.read_bit(contexts, context as usize);
                        result |= bit << (7 - x_minor);
                        context = ((context & 0x7bf7) << 1) | (bit as u32);
                    }
                    row[x >> 3] = result;
                } else if skip_byte == 0xFF {
                    for _ in 0..minor_width {
                        context = (context & 0x7bf7) << 1;
                    }
                    row[x >> 3] = 0;
                } else {
                    let mut result = 0u8;
                    let mut skip_mask = 0x80u8;
                    for x_minor in 0..minor_width {
                        let skip_set = (skip_byte & skip_mask) != 0;
                        let bit = if skip_set {
                            0
                        } else {
                            decoder.read_bit(contexts, context as usize)
                        };
                        result |= bit << (7 - x_minor);
                        context = ((context & 0x7bf7) << 1) | (bit as u32);
                        skip_mask >>= 1;
                    }
                    row[x >> 3] = result;
                }
            }
            continue;
        }

        let line1_row = &before[(y - 1) * rowstride..y * rowstride];
        let mut line_m1 = line1_row[0] as u32;

        if y == 1 {
            let mut context = line_m1 & 0x7f0;
            for x in (0..padded_width).step_by(8) {
                let minor_width = if width - x > 8 { 8 } else { width - x };
                let next = if x + 8 < width {
                    line1_row[(x >> 3) + 1] as u32
                } else {
                    0
                };
                line_m1 = (line_m1 << 8) | next;

                let skip_byte = skip_row[x >> 3];
                if skip_byte == 0 {
                    let mut result = 0u8;
                    for x_minor in 0..minor_width {
                        let bit = decoder.read_bit(contexts, context as usize);
                        result |= bit << (7 - x_minor);
                        let line_m1_bit = (line_m1 >> (7 - x_minor)) & 0x10;
                        context = ((context & 0x7bf7) << 1) | (bit as u32) | line_m1_bit;
                    }
                    row[x >> 3] = result;
                } else if skip_byte == 0xFF {
                    for x_minor in 0..minor_width {
                        let line_m1_bit = (line_m1 >> (7 - x_minor)) & 0x10;
                        context = ((context & 0x7bf7) << 1) | line_m1_bit;
                    }
                    row[x >> 3] = 0;
                } else {
                    let mut result = 0u8;
                    let mut skip_mask = 0x80u8;
                    for x_minor in 0..minor_width {
                        let skip_set = (skip_byte & skip_mask) != 0;
                        let bit = if skip_set {
                            0
                        } else {
                            decoder.read_bit(contexts, context as usize)
                        };
                        result |= bit << (7 - x_minor);
                        let line_m1_bit = (line_m1 >> (7 - x_minor)) & 0x10;
                        context = ((context & 0x7bf7) << 1) | (bit as u32) | line_m1_bit;
                        skip_mask >>= 1;
                    }
                    row[x >> 3] = result;
                }
            }
            continue;
        }

        let line2_row = &before[(y - 2) * rowstride..(y - 1) * rowstride];
        let mut line_m2 = (line2_row[0] as u32) << 6;
        let mut context = (line_m1 & 0x7f0) | (line_m2 & 0xf800);

        for x in (0..padded_width).step_by(8) {
            let minor_width = if width - x > 8 { 8 } else { width - x };
            let next1 = if x + 8 < width {
                line1_row[(x >> 3) + 1] as u32
            } else {
                0
            };
            line_m1 = (line_m1 << 8) | next1;
            let next2 = if x + 8 < width {
                line2_row[(x >> 3) + 1] as u32
            } else {
                0
            };
            line_m2 = (line_m2 << 8) | (next2 << 6);

            let skip_byte = skip_row[x >> 3];
            if skip_byte == 0 {
                let mut result = 0u8;
                for x_minor in 0..minor_width {
                    let bit = decoder.read_bit(contexts, context as usize);
                    result |= bit << (7 - x_minor);
                    let line_m1_bit = (line_m1 >> (7 - x_minor)) & 0x10;
                    let line_m2_bit = (line_m2 >> (7 - x_minor)) & 0x800;
                    context = ((context & 0x7bf7) << 1) | (bit as u32) | line_m1_bit | line_m2_bit;
                }
                row[x >> 3] = result;
            } else if skip_byte == 0xFF {
                for x_minor in 0..minor_width {
                    let line_m1_bit = (line_m1 >> (7 - x_minor)) & 0x10;
                    let line_m2_bit = (line_m2 >> (7 - x_minor)) & 0x800;
                    context = ((context & 0x7bf7) << 1) | line_m1_bit | line_m2_bit;
                }
                row[x >> 3] = 0;
            } else {
                let mut result = 0u8;
                let mut skip_mask = 0x80u8;
                for x_minor in 0..minor_width {
                    let skip_set = (skip_byte & skip_mask) != 0;
                    let bit = if skip_set {
                        0
                    } else {
                        decoder.read_bit(contexts, context as usize)
                    };
                    result |= bit << (7 - x_minor);
                    let line_m1_bit = (line_m1 >> (7 - x_minor)) & 0x10;
                    let line_m2_bit = (line_m2 >> (7 - x_minor)) & 0x800;
                    context = ((context & 0x7bf7) << 1) | (bit as u32) | line_m1_bit | line_m2_bit;
                    skip_mask >>= 1;
                }
                row[x >> 3] = result;
            }
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
    pub at: &'a [(i8, i8)],
}

struct GenericDecodeTemplate {
    template_length: usize,
    template_x: [i8; 16],
    template_y: [i8; 16],
    changing_template_x: [i8; 16],
    changing_template_y: [i8; 16],
    changing_template_bit: [u16; 16],
    changing_entries_length: usize,
    reuse_mask: u16,
    sbb_left: usize,
    sbb_top: usize,
    sbb_right: usize,
}

fn build_generic_template(params: &DecodeBitmapParams<'_>) -> GenericDecodeTemplate {
    let mut template = [(0i8, 0i8); 16];
    let mut template_length = 0;

    for &item in get_coding_template(params.template_index) {
        if template_length < 16 {
            template[template_length] = item;
            template_length += 1;
        }
    }
    for &item in params.at {
        if template_length < 16 {
            template[template_length] = item;
            template_length += 1;
        }
    }

    template[..template_length].sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
    debug_assert!(template_length <= 16);

    let mut template_x = [0i8; 16];
    let mut template_y = [0i8; 16];
    let mut changing_template_x = [0i8; 16];
    let mut changing_template_y = [0i8; 16];
    let mut changing_template_bit = [0u16; 16];

    let mut changing_template_entries = [0usize; 16];
    let mut changing_entries_length = 0;

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
            changing_template_entries[changing_entries_length] = k;
            changing_entries_length += 1;
        }
    }

    for c in 0..changing_entries_length {
        let k = changing_template_entries[c];
        changing_template_x[c] = template[k].0;
        changing_template_y[c] = template[k].1;
        changing_template_bit[c] = 1 << (template_length - 1 - k);
    }

    let sbb_left = (-min_x) as usize;
    let sbb_top = (-min_y) as usize;
    let sbb_right = params.width.saturating_sub(max_x as usize);

    GenericDecodeTemplate {
        template_length,
        template_x,
        template_y,
        changing_template_x,
        changing_template_y,
        changing_template_bit,
        changing_entries_length,
        reuse_mask,
        sbb_left,
        sbb_top,
        sbb_right,
    }
}

/// Decode a generic region bitmap using either MMR or arithmetic coding.
pub fn decode_bitmap(
    params: &DecodeBitmapParams<'_>,
    decoding_context: &mut DecodingContext<'_>,
) -> Result<Bitmap, Jbig2Error> {
    // Early return for zero dimensions.
    if params.width == 0 || params.height == 0 {
        return Ok(Bitmap::new(params.width, params.height));
    }

    // Validate parameters before decoding.
    validation::validate_generic_decode_params(params.width, params.height, params.template_index)?;

    if params.mmr {
        let mut reader = Reader::new(
            decoding_context.data,
            decoding_context.start,
            decoding_context.end,
        );
        let bitmap = decode_mmr_bitmap(&mut reader, params.width, params.height, true)?;
        decoding_context.start = reader.get_position();
        return Ok(bitmap);
    }

    // Use an optimized path for the common template-0 case.
    if params.template_index == 0
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
        if let Some(skip) = params.skip {
            return decode_bitmap_template0_with_skip(
                params.width,
                params.height,
                skip,
                decoding_context,
            );
        }
        return decode_bitmap_template0(params.width, params.height, decoding_context);
    }

    let template = build_generic_template(params);
    if params.skip.is_some() {
        decode_bitmap_with_skip(params, decoding_context, &template)
    } else {
        decode_bitmap_no_skip(params, decoding_context, &template)
    }
}

fn decode_bitmap_no_skip(
    params: &DecodeBitmapParams<'_>,
    decoding_context: &mut DecodingContext<'_>,
    template: &GenericDecodeTemplate,
) -> Result<Bitmap, Jbig2Error> {
    let pseudo_pixel_context = REUSED_CONTEXTS[params.template_index];
    let mut bitmap = Bitmap::new(params.width, params.height);
    let mut decoder = decoding_context.get_decoder();
    let mut contexts = decoding_context.get_contexts("GB");
    let contexts = contexts.as_mut();
    let width_i32 = params.width as i32;
    let height_i32 = params.height as i32;
    let safe_start = template.sbb_left.min(params.width);
    let safe_end = template.sbb_right.min(params.width);
    let template_length = template.template_length;
    let template_x = &template.template_x;
    let template_y = &template.template_y;
    let changing_entries_length = template.changing_entries_length;
    let changing_template_x = &template.changing_template_x;
    let changing_template_y = &template.changing_template_y;
    let changing_template_bit = &template.changing_template_bit;
    let reuse_mask = template.reuse_mask;
    let mut ltp = 0i32;

    let mut template_offsets = [0usize; 16];
    let mut template_is_valid = [false; 16];
    let mut changing_template_offsets = [0usize; 16];

    for i in 0..params.height {
        if params.prediction {
            let sltp = decoder.read_bit(contexts, pseudo_pixel_context as usize) as i32;
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

        let row_start_index = unsafe { bitmap.get_row_start_index_unchecked(i) };

        for k in 0..template_length {
            let i0 = i as i32 + template_y[k] as i32;
            if i0 >= 0 && i0 < height_i32 {
                template_is_valid[k] = true;
                template_offsets[k] = unsafe { bitmap.get_row_start_index_unchecked(i0 as usize) };
            } else {
                template_is_valid[k] = false;
            }
        }

        let mut context_label: u16;
        if i < template.sbb_top || safe_start >= safe_end {
            for j in 0..params.width {
                let mut full = 0u16;
                let mut shift = template_length as i32 - 1;
                for k in 0..template_length {
                    let j0 = j as i32 + template_x[k] as i32;
                    if j0 >= 0 && j0 < width_i32 && template_is_valid[k] {
                        if unsafe {
                            bitmap.get_pixel_at_index_unchecked(template_offsets[k], j0 as usize)
                        } != 0
                        {
                            full |= 1 << shift;
                        }
                    }
                    shift -= 1;
                }
                context_label = full;
                let pixel = decoder.read_bit(contexts, context_label as usize);
                unsafe {
                    bitmap.set_pixel_at_index_unchecked(row_start_index, j, pixel);
                }
            }
            continue;
        }

        for k in 0..changing_entries_length {
            let i0 = (i as i32 + changing_template_y[k] as i32) as usize;
            changing_template_offsets[k] = unsafe { bitmap.get_row_start_index_unchecked(i0) };
        }

        for j in 0..safe_start {
            let mut full = 0u16;
            let mut shift = template_length as i32 - 1;
            for k in 0..template_length {
                let j0 = j as i32 + template_x[k] as i32;
                if j0 >= 0 && j0 < width_i32 && template_is_valid[k] {
                    if unsafe {
                        bitmap.get_pixel_at_index_unchecked(template_offsets[k], j0 as usize)
                    } != 0
                    {
                        full |= 1 << shift;
                    }
                }
                shift -= 1;
            }
            context_label = full;
            let pixel = decoder.read_bit(contexts, context_label as usize);
            unsafe {
                bitmap.set_pixel_at_index_unchecked(row_start_index, j, pixel);
            }
        }

        {
            let j = safe_start;
            let mut full = 0u16;
            let mut shift = template_length as i32 - 1;
            for k in 0..template_length {
                let j0 = (j as i32 + template_x[k] as i32) as usize;
                if unsafe { bitmap.get_pixel_at_index_unchecked(template_offsets[k], j0) } != 0 {
                    full |= 1 << shift;
                }
                shift -= 1;
            }
            context_label = full;
            let pixel = decoder.read_bit(contexts, context_label as usize);
            unsafe {
                bitmap.set_pixel_at_index_unchecked(row_start_index, j, pixel);
            }
        }

        let stride = bitmap.stride;
        let data_ptr = bitmap.data.as_mut_ptr();
        let mut context_row_ptrs = [std::ptr::null::<u8>(); 16];
        for k in 0..changing_entries_length {
            let i0 = (i as i32 + changing_template_y[k] as i32) as usize;
            context_row_ptrs[k] = unsafe { data_ptr.add(i0 * stride) };
        }
        let dst_row_ptr = unsafe { data_ptr.add(i * stride) };

        let mut j = safe_start + 1;
        let limit = safe_end;

        // Split entries into static (previous rows) and dynamic (current row).
        let mut static_count = 0;
        let mut static_bits = [0u16; 16];
        let mut static_x = [0i8; 16];
        let mut static_ptrs = [std::ptr::null::<u8>(); 16];

        let mut dynamic_count = 0;
        let mut dynamic_bits = [0u16; 16];
        let mut dynamic_x = [0i8; 16];
        let mut dynamic_ptrs = [std::ptr::null::<u8>(); 16];

        for k in 0..changing_entries_length {
            if changing_template_y[k] != 0 {
                static_bits[static_count] = changing_template_bit[k];
                static_x[static_count] = changing_template_x[k];
                static_ptrs[static_count] = context_row_ptrs[k];
                static_count += 1;
            } else {
                dynamic_bits[dynamic_count] = changing_template_bit[k];
                dynamic_x[dynamic_count] = changing_template_x[k];
                dynamic_ptrs[dynamic_count] = context_row_ptrs[k];
                dynamic_count += 1;
            }
        }

        // Optimization: process 56 pixels at a time using u64 registers for static entries.
        let chunk_size = 56;
        let safe_limit = (stride * 8).saturating_sub(128);
        let chunk_limit = limit.min(safe_limit);

        while j < chunk_limit {
            let mut static_words = [0u64; 16];
            for k in 0..static_count {
                let j0 = (j as i32 + static_x[k] as i32) as usize;
                let byte_offset = j0 >> 3;
                let bit_offset = j0 & 7;
                let ptr = unsafe { static_ptrs[k].add(byte_offset) };
                let val = u64::from_be_bytes(unsafe { *(ptr as *const [u8; 8]) });
                static_words[k] = val << bit_offset;
            }

            for _ in 0..chunk_size {
                context_label = (context_label << 1) & reuse_mask;

                // Process static entries (fast path)
                for k in 0..static_count {
                    if (static_words[k] as i64) < 0 {
                        context_label |= static_bits[k];
                    }
                    static_words[k] <<= 1;
                }

                // Process dynamic entries (slow path)
                for k in 0..dynamic_count {
                    let j0 = (j as i32 + dynamic_x[k] as i32) as usize;
                    let val = unsafe { *dynamic_ptrs[k].add(j0 >> 3) };
                    if (val >> (7 - (j0 & 7))) & 1 != 0 {
                        context_label |= dynamic_bits[k];
                    }
                }

                let pixel = decoder.read_bit(contexts, context_label as usize);
                let byte_idx = j >> 3;
                let bit_idx = 7 - (j & 7);
                if pixel != 0 {
                    unsafe { *dst_row_ptr.add(byte_idx) |= 1 << bit_idx };
                } else {
                    unsafe { *dst_row_ptr.add(byte_idx) &= !(1 << bit_idx) };
                }
                j += 1;
            }
        }

        for j in j..limit {
            context_label = (context_label << 1) & reuse_mask;
            for k in 0..changing_entries_length {
                let j0 = (j as i32 + changing_template_x[k] as i32) as usize;
                let val = unsafe { *context_row_ptrs[k].add(j0 >> 3) };
                if (val >> (7 - (j0 & 7))) & 1 != 0 {
                    context_label |= changing_template_bit[k];
                }
            }
            let pixel = decoder.read_bit(contexts, context_label as usize);
            let byte_idx = j >> 3;
            let bit_idx = 7 - (j & 7);
            if pixel != 0 {
                unsafe { *dst_row_ptr.add(byte_idx) |= 1 << bit_idx };
            } else {
                unsafe { *dst_row_ptr.add(byte_idx) &= !(1 << bit_idx) };
            }
        }

        for j in safe_end..params.width {
            context_label = (context_label << 1) & reuse_mask;
            for k in 0..changing_entries_length {
                let j0_i32 = j as i32 + changing_template_x[k] as i32;
                if j0_i32 >= 0 && j0_i32 < width_i32 {
                    let j0 = j0_i32 as usize;
                    let val = unsafe { *context_row_ptrs[k].add(j0 >> 3) };
                    if (val >> (7 - (j0 & 7))) & 1 != 0 {
                        context_label |= changing_template_bit[k];
                    }
                }
            }
            let pixel = decoder.read_bit(contexts, context_label as usize);
            unsafe {
                bitmap.set_pixel_at_index_unchecked(row_start_index, j, pixel);
            }
        }
    }
    Ok(bitmap)
}

fn decode_bitmap_with_skip(
    params: &DecodeBitmapParams<'_>,
    decoding_context: &mut DecodingContext<'_>,
    template: &GenericDecodeTemplate,
) -> Result<Bitmap, Jbig2Error> {
    let skip = params
        .skip
        .expect("decode_bitmap_with_skip requires skip bitmap");
    let skip_stride = skip.stride;
    let skip_data = skip.data.as_slice();
    let pseudo_pixel_context = REUSED_CONTEXTS[params.template_index];
    let mut bitmap = Bitmap::new(params.width, params.height);
    let mut decoder = decoding_context.get_decoder();
    let mut contexts = decoding_context.get_contexts("GB");
    let contexts = contexts.as_mut();
    let width_i32 = params.width as i32;
    let height_i32 = params.height as i32;
    let safe_start = template.sbb_left.min(params.width);
    let safe_end = template.sbb_right.min(params.width);
    let template_length = template.template_length;
    let template_x = &template.template_x;
    let template_y = &template.template_y;
    let changing_entries_length = template.changing_entries_length;
    let changing_template_x = &template.changing_template_x;
    let changing_template_y = &template.changing_template_y;
    let changing_template_bit = &template.changing_template_bit;
    let reuse_mask = template.reuse_mask;
    let mut ltp = 0i32;

    let mut template_offsets = [0usize; 16];
    let mut template_is_valid = [false; 16];
    let mut changing_template_offsets = [0usize; 16];

    for i in 0..params.height {
        if params.prediction {
            let sltp = decoder.read_bit(contexts, pseudo_pixel_context as usize) as i32;
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
        let skip_row_start = i * skip_stride;
        let skip_row = &skip_data[skip_row_start..skip_row_start + skip_stride];

        let row_start_index = unsafe { bitmap.get_row_start_index_unchecked(i) };

        for k in 0..template_length {
            let i0 = i as i32 + template_y[k] as i32;
            if i0 >= 0 && i0 < height_i32 {
                template_is_valid[k] = true;
                template_offsets[k] = unsafe { bitmap.get_row_start_index_unchecked(i0 as usize) };
            } else {
                template_is_valid[k] = false;
            }
        }

        let mut context_label: u16;
        if i < template.sbb_top || safe_start >= safe_end {
            for j in 0..params.width {
                let mut full = 0u16;
                let mut shift = template_length as i32 - 1;
                for k in 0..template_length {
                    let j0 = j as i32 + template_x[k] as i32;
                    if j0 >= 0 && j0 < width_i32 && template_is_valid[k] {
                        if unsafe {
                            bitmap.get_pixel_at_index_unchecked(template_offsets[k], j0 as usize)
                        } != 0
                        {
                            full |= 1 << shift;
                        }
                    }
                    shift -= 1;
                }
                context_label = full;
                let byte_index = j >> 3;
                let mask = 1u8 << (7 - (j & 7));
                let pixel = if (skip_row[byte_index] & mask) != 0 {
                    0
                } else {
                    decoder.read_bit(contexts, context_label as usize)
                };
                unsafe {
                    bitmap.set_pixel_at_index_unchecked(row_start_index, j, pixel);
                }
            }
            continue;
        }

        for k in 0..changing_entries_length {
            let i0 = (i as i32 + changing_template_y[k] as i32) as usize;
            changing_template_offsets[k] = unsafe { bitmap.get_row_start_index_unchecked(i0) };
        }

        for j in 0..safe_start {
            let mut full = 0u16;
            let mut shift = template_length as i32 - 1;
            for k in 0..template_length {
                let j0 = j as i32 + template_x[k] as i32;
                if j0 >= 0 && j0 < width_i32 && template_is_valid[k] {
                    if unsafe {
                        bitmap.get_pixel_at_index_unchecked(template_offsets[k], j0 as usize)
                    } != 0
                    {
                        full |= 1 << shift;
                    }
                }
                shift -= 1;
            }
            context_label = full;
            let byte_index = j >> 3;
            let mask = 1u8 << (7 - (j & 7));
            let pixel = if (skip_row[byte_index] & mask) != 0 {
                0
            } else {
                decoder.read_bit(contexts, context_label as usize)
            };
            unsafe {
                bitmap.set_pixel_at_index_unchecked(row_start_index, j, pixel);
            }
        }

        {
            let j = safe_start;
            let mut full = 0u16;
            let mut shift = template_length as i32 - 1;
            for k in 0..template_length {
                let j0 = (j as i32 + template_x[k] as i32) as usize;
                if unsafe { bitmap.get_pixel_at_index_unchecked(template_offsets[k], j0) } != 0 {
                    full |= 1 << shift;
                }
                shift -= 1;
            }
            context_label = full;
            let byte_index = j >> 3;
            let mask = 1u8 << (7 - (j & 7));
            let pixel = if (skip_row[byte_index] & mask) != 0 {
                0
            } else {
                decoder.read_bit(contexts, context_label as usize)
            };
            unsafe {
                bitmap.set_pixel_at_index_unchecked(row_start_index, j, pixel);
            }
        }

        let stride = bitmap.stride;
        let data_ptr = bitmap.data.as_mut_ptr();
        let mut context_row_ptrs = [std::ptr::null::<u8>(); 16];
        for k in 0..changing_entries_length {
            let i0 = (i as i32 + changing_template_y[k] as i32) as usize;
            context_row_ptrs[k] = unsafe { data_ptr.add(i0 * stride) };
        }
        let dst_row_ptr = unsafe { data_ptr.add(i * stride) };

        for j in (safe_start + 1)..safe_end {
            context_label = (context_label << 1) & reuse_mask;
            for k in 0..changing_entries_length {
                let j0 = (j as i32 + changing_template_x[k] as i32) as usize;
                let val = unsafe { *context_row_ptrs[k].add(j0 >> 3) };
                if (val >> (7 - (j0 & 7))) & 1 != 0 {
                    context_label |= changing_template_bit[k];
                }
            }
            let byte_idx = j >> 3;
            let bit_idx = 7 - (j & 7);
            let mask = 1u8 << bit_idx;
            let pixel = if (skip_row[byte_idx] & mask) != 0 {
                0
            } else {
                decoder.read_bit(contexts, context_label as usize)
            };
            if pixel != 0 {
                unsafe { *dst_row_ptr.add(byte_idx) |= mask };
            } else {
                unsafe { *dst_row_ptr.add(byte_idx) &= !mask };
            }
        }

        for j in safe_end..params.width {
            context_label = (context_label << 1) & reuse_mask;
            for k in 0..changing_entries_length {
                let j0_i32 = j as i32 + changing_template_x[k] as i32;
                if j0_i32 >= 0 && j0_i32 < width_i32 {
                    let j0 = j0_i32 as usize;
                    let val = unsafe { *context_row_ptrs[k].add(j0 >> 3) };
                    if (val >> (7 - (j0 & 7))) & 1 != 0 {
                        context_label |= changing_template_bit[k];
                    }
                }
            }
            let byte_index = j >> 3;
            let mask = 1u8 << (7 - (j & 7));
            let pixel = if (skip_row[byte_index] & mask) != 0 {
                0
            } else {
                decoder.read_bit(contexts, context_label as usize)
            };
            unsafe {
                bitmap.set_pixel_at_index_unchecked(row_start_index, j, pixel);
            }
        }
    }
    Ok(bitmap)
}
