use crate::arithmetic::ArithmeticDecoder;
use crate::bitmap::Bitmap;
use crate::contexts::DecodingContext;
use crate::error::Jbig2Error;
const REFINEMENT_REUSED_CONTEXTS: [u16; 2] = [
    0x0100, // TPGRON start context for template 0
    0x0040, // TPGRON start context for template 1
];

const CODING_TEMPLATE_0: [(i8, i8); 3] = [(-1, 0), (1, -1), (0, -1)];
const REFERENCE_TEMPLATE_0: [(i8, i8); 8] = [
    (1, 1),
    (0, 1),
    (-1, 1),
    (1, 0),
    (0, 0),
    (-1, 0),
    (1, -1),
    (0, -1),
];
const CODING_TEMPLATE_1: [(i8, i8); 4] = [(-1, 0), (1, -1), (0, -1), (-1, -1)];
const REFERENCE_TEMPLATE_1: [(i8, i8); 6] =
    [(1, 1), (0, 1), (1, 0), (0, 0), (-1, 0), (0, -1)];
const MAX_CODING_TEMPLATE_LEN: usize = 4;
const MAX_REFERENCE_TEMPLATE_LEN: usize = 9;

/// Coding and reference templates for refinement decoding.
#[derive(Clone)]
pub struct RefinementTemplate {
    pub coding: Vec<(i8, i8)>,
    pub reference: Vec<(i8, i8)>,
}
/// Return the refinement template set for the given index.
pub fn get_refinement_template(index: usize) -> RefinementTemplate {
    match index {
        0 => RefinementTemplate {
            coding: CODING_TEMPLATE_0.to_vec(),
            reference: REFERENCE_TEMPLATE_0.to_vec(),
        },
        1 => RefinementTemplate {
            coding: CODING_TEMPLATE_1.to_vec(),
            reference: REFERENCE_TEMPLATE_1.to_vec(),
        },
        _ => RefinementTemplate {
            coding: vec![],
            reference: vec![],
        },
    }
}

fn fill_template(
    dst_x: &mut [i32],
    dst_y: &mut [i32],
    base: &[(i8, i8)],
    extra: Option<(i8, i8)>,
) -> (usize, i32, i32, i32, i32) {
    let mut len = 0usize;
    let mut min_x = i32::MAX;
    let mut max_x = i32::MIN;
    let mut min_y = i32::MAX;
    let mut max_y = i32::MIN;
    for &(x, y) in base {
        let x = x as i32;
        let y = y as i32;
        dst_x[len] = x;
        dst_y[len] = y;
        len += 1;
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
    }
    if let Some((x, y)) = extra {
        let x = x as i32;
        let y = y as i32;
        dst_x[len] = x;
        dst_y[len] = y;
        len += 1;
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
    }
    if len == 0 {
        min_x = 0;
        max_x = 0;
        min_y = 0;
        max_y = 0;
    }
    (len, min_x, max_x, min_y, max_y)
}

fn clamp_range(start: i32, end: i32, limit: i32) -> (usize, usize) {
    let mut s = start;
    let mut e = end;
    if s < 0 {
        s = 0;
    }
    if e < 0 {
        e = 0;
    }
    if s > limit {
        s = limit;
    }
    if e > limit {
        e = limit;
    }
    let s_usize = s as usize;
    let e_usize = e as usize;
    if e_usize < s_usize {
        (s_usize, s_usize)
    } else {
        (s_usize, e_usize)
    }
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn decode_refinement_range_slow(
    bitmap: &mut Bitmap,
    reference_bitmap: &Bitmap,
    decoder: &mut ArithmeticDecoder<'_>,
    contexts: &mut [i8],
    row: usize,
    start: usize,
    end: usize,
    use_prediction: bool,
    offset_x: i32,
    offset_y: i32,
    width_i32: i32,
    reference_width_i32: i32,
    reference_height_i32: i32,
    coding_template_length: usize,
    coding_template_x: &[i32],
    coding_template_y: &[i32],
    reference_template_length: usize,
    reference_template_x: &[i32],
    reference_template_y: &[i32],
) -> Result<(), Jbig2Error> {
    for j in start..end {
        let mut context_label = 0u16;
        let mut implicit = None;
        if use_prediction {
            let i_ref = j as i32 - offset_x;
            let j_ref = row as i32 - offset_y;
            let get_ref = |x: i32, y: i32| -> u8 {
                if x < 0 || y < 0 || x >= reference_width_i32 || y >= reference_height_i32 {
                    0
                } else {
                    reference_bitmap.get_pixel_unchecked(x as usize, y as usize)
                }
            };
            let m = get_ref(i_ref, j_ref);
            if get_ref(i_ref - 1, j_ref - 1) == m
                && get_ref(i_ref, j_ref - 1) == m
                && get_ref(i_ref + 1, j_ref - 1) == m
                && get_ref(i_ref - 1, j_ref) == m
                && get_ref(i_ref + 1, j_ref) == m
                && get_ref(i_ref - 1, j_ref + 1) == m
                && get_ref(i_ref, j_ref + 1) == m
                && get_ref(i_ref + 1, j_ref + 1) == m
            {
                implicit = Some(m);
            }
        }
        if let Some(pixel) = implicit {
            bitmap.set_pixel_unchecked(j, row, pixel);
            continue;
        }
        for k in 0..coding_template_length {
            let i0 = row as i32 + coding_template_y[k];
            let j0 = j as i32 + coding_template_x[k];
            if i0 >= 0 && j0 >= 0 && j0 < width_i32 {
                let bit = bitmap.get_pixel_unchecked(j0 as usize, i0 as usize) as u16;
                if bit != 0 {
                    context_label |= 1 << k;
                }
            }
        }
        for k in 0..reference_template_length {
            let i0 = row as i32 + reference_template_y[k] - offset_y;
            let j0 = j as i32 + reference_template_x[k] - offset_x;
            if i0 >= 0
                && i0 < reference_height_i32
                && j0 >= 0
                && j0 < reference_width_i32
            {
                let bit = reference_bitmap.get_pixel_unchecked(j0 as usize, i0 as usize) as u16;
                if bit != 0 {
                    context_label |= 1 << (coding_template_length + k);
                }
            }
        }
        let pixel = decoder.read_bit(contexts, context_label as usize)?;
        bitmap.set_pixel_unchecked(j, row, pixel);
    }
    Ok(())
}
/// Inputs required to decode a refinement region.
#[derive(Clone)]
pub struct RefinementParams<'a> {
    pub width: usize,
    pub height: usize,
    pub template_index: usize,
    pub reference_bitmap: &'a Bitmap,
    pub offset_x: i32,
    pub offset_y: i32,
    pub prediction: bool,
    pub at: &'a [(i8, i8)],
}
/// Decode a refinement bitmap using the reference bitmap and context templates.
pub fn decode_refinement<'a>(
    params: &RefinementParams<'a>,
    decoding_context: &mut DecodingContext<'_>,
) -> Result<Bitmap, Jbig2Error> {
    // Validate template index.
    if params.template_index > 1 {
        return Err(Jbig2Error::new("invalid refinement template index"));
    }
    // Validate AT parameters.
    if params.template_index == 0 && params.at.len() < 2 {
        return Err(Jbig2Error::new("template 0 requires 2 AT parameters"));
    }

    let (coding_base, reference_base) = if params.template_index == 0 {
        (&CODING_TEMPLATE_0[..], &REFERENCE_TEMPLATE_0[..])
    } else {
        (&CODING_TEMPLATE_1[..], &REFERENCE_TEMPLATE_1[..])
    };
    let extra_coding = if params.template_index == 0 {
        Some(params.at[0])
    } else {
        None
    };
    let extra_reference = if params.template_index == 0 {
        Some(params.at[1])
    } else {
        None
    };
    let mut coding_template_x = [0i32; MAX_CODING_TEMPLATE_LEN];
    let mut coding_template_y = [0i32; MAX_CODING_TEMPLATE_LEN];
    let (coding_template_length, coding_min_x, coding_max_x, coding_min_y, coding_max_y) =
        fill_template(
            &mut coding_template_x,
            &mut coding_template_y,
            coding_base,
            extra_coding,
        );
    let mut reference_template_x = [0i32; MAX_REFERENCE_TEMPLATE_LEN];
    let mut reference_template_y = [0i32; MAX_REFERENCE_TEMPLATE_LEN];
    let (reference_template_length, ref_min_x, ref_max_x, ref_min_y, ref_max_y) = fill_template(
        &mut reference_template_x,
        &mut reference_template_y,
        reference_base,
        extra_reference,
    );
    let reference_width = params.reference_bitmap.width;
    let reference_height = params.reference_bitmap.height;
    let reference_width_i32 = reference_width as i32;
    let reference_height_i32 = reference_height as i32;
    let width_i32 = params.width as i32;
    let height_i32 = params.height as i32;
    let offset_x = params.offset_x;
    let offset_y = params.offset_y;
    let start_context = REFINEMENT_REUSED_CONTEXTS[params.template_index];
    let safe_x_start = 0
        .max(-coding_min_x)
        .max(offset_x - ref_min_x);
    let safe_x_end = width_i32
        .min(width_i32 - coding_max_x)
        .min(offset_x + reference_width_i32 - ref_max_x);
    let safe_y_start = 0
        .max(-coding_min_y)
        .max(offset_y - ref_min_y);
    let safe_y_end = height_i32
        .min(height_i32 - coding_max_y)
        .min(offset_y + reference_height_i32 - ref_max_y);
    let (safe_x_start, safe_x_end) = clamp_range(safe_x_start, safe_x_end, width_i32);
    let (safe_y_start, safe_y_end) = clamp_range(safe_y_start, safe_y_end, height_i32);
    let mut contexts = decoding_context.get_contexts("GR");
    let contexts = contexts.as_mut();
    let mut decoder = decoding_context.get_decoder();
    let mut bitmap = Bitmap::new(params.width, params.height);
    let mut ltp = 0i32;
    for i in 0..params.height {
        if params.prediction {
            let sltp = decoder.read_bit(contexts, start_context as usize)? as i32;
            ltp ^= sltp;
        }
        let use_prediction = params.prediction && ltp != 0;
        let row_safe = i >= safe_y_start && i < safe_y_end && safe_x_start < safe_x_end;

        if row_safe {
            decode_refinement_range_slow(
                &mut bitmap,
                params.reference_bitmap,
                &mut decoder,
                contexts,
                i,
                0,
                safe_x_start,
                use_prediction,
                offset_x,
                offset_y,
                width_i32,
                reference_width_i32,
                reference_height_i32,
                coding_template_length,
                &coding_template_x,
                &coding_template_y,
                reference_template_length,
                &reference_template_x,
                &reference_template_y,
            )?;

            if use_prediction {
                for j in safe_x_start..safe_x_end {
                    let mut context_label = 0u16;
                    let mut implicit = None;
                    let i_ref = j as i32 - offset_x;
                    let j_ref = i as i32 - offset_y;
                    let get_ref = |x: i32, y: i32| -> u8 {
                        if x < 0
                            || y < 0
                            || x >= reference_width_i32
                            || y >= reference_height_i32
                        {
                            0
                        } else {
                            params
                                .reference_bitmap
                                .get_pixel_unchecked(x as usize, y as usize)
                        }
                    };
                    let m = get_ref(i_ref, j_ref);
                    if get_ref(i_ref - 1, j_ref - 1) == m
                        && get_ref(i_ref, j_ref - 1) == m
                        && get_ref(i_ref + 1, j_ref - 1) == m
                        && get_ref(i_ref - 1, j_ref) == m
                        && get_ref(i_ref + 1, j_ref) == m
                        && get_ref(i_ref - 1, j_ref + 1) == m
                        && get_ref(i_ref, j_ref + 1) == m
                        && get_ref(i_ref + 1, j_ref + 1) == m
                    {
                        implicit = Some(m);
                    }
                    if let Some(pixel) = implicit {
                        bitmap.set_pixel_unchecked(j, i, pixel);
                        continue;
                    }
                    for k in 0..coding_template_length {
                        let i0 = (i as i32 + coding_template_y[k]) as usize;
                        let j0 = (j as i32 + coding_template_x[k]) as usize;
                        let bit = bitmap.get_pixel_unchecked(j0, i0) as u16;
                        if bit != 0 {
                            context_label |= 1 << k;
                        }
                    }
                    for k in 0..reference_template_length {
                        let i0 = (i as i32 + reference_template_y[k] - offset_y) as usize;
                        let j0 = (j as i32 + reference_template_x[k] - offset_x) as usize;
                        let bit = params.reference_bitmap.get_pixel_unchecked(j0, i0) as u16;
                        if bit != 0 {
                            context_label |= 1 << (coding_template_length + k);
                        }
                    }
                    let pixel = decoder.read_bit(contexts, context_label as usize)?;
                    bitmap.set_pixel_unchecked(j, i, pixel);
                }
            } else {
                for j in safe_x_start..safe_x_end {
                    let mut context_label = 0u16;
                    for k in 0..coding_template_length {
                        let i0 = (i as i32 + coding_template_y[k]) as usize;
                        let j0 = (j as i32 + coding_template_x[k]) as usize;
                        let bit = bitmap.get_pixel_unchecked(j0, i0) as u16;
                        if bit != 0 {
                            context_label |= 1 << k;
                        }
                    }
                    for k in 0..reference_template_length {
                        let i0 = (i as i32 + reference_template_y[k] - offset_y) as usize;
                        let j0 = (j as i32 + reference_template_x[k] - offset_x) as usize;
                        let bit = params.reference_bitmap.get_pixel_unchecked(j0, i0) as u16;
                        if bit != 0 {
                            context_label |= 1 << (coding_template_length + k);
                        }
                    }
                    let pixel = decoder.read_bit(contexts, context_label as usize)?;
                    bitmap.set_pixel_unchecked(j, i, pixel);
                }
            }

            decode_refinement_range_slow(
                &mut bitmap,
                params.reference_bitmap,
                &mut decoder,
                contexts,
                i,
                safe_x_end,
                params.width,
                use_prediction,
                offset_x,
                offset_y,
                width_i32,
                reference_width_i32,
                reference_height_i32,
                coding_template_length,
                &coding_template_x,
                &coding_template_y,
                reference_template_length,
                &reference_template_x,
                &reference_template_y,
            )?;
        } else {
            decode_refinement_range_slow(
                &mut bitmap,
                params.reference_bitmap,
                &mut decoder,
                contexts,
                i,
                0,
                params.width,
                use_prediction,
                offset_x,
                offset_y,
                width_i32,
                reference_width_i32,
                reference_height_i32,
                coding_template_length,
                &coding_template_x,
                &coding_template_y,
                reference_template_length,
                &reference_template_x,
                &reference_template_y,
            )?;
        }
    }
    Ok(bitmap)
}
