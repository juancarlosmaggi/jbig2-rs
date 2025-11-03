use crate::bitmap::Bitmap;
use crate::contexts::DecodingContext;
use crate::decode_mmr::decode_mmr_bitmap;
use crate::error::Jbig2Error;
use crate::reader::Reader;
const OLD_PIXEL_MASK: u16 = 0x7bf7;
const REUSED_CONTEXTS: [u16; 4] = [
    0x9b25, // 10011 0110010 0101
    0x0795, // 0011 110010 101
    0x00e5, // 001 11001 01
    0x0195, // 011001 0101
];
pub fn get_coding_template(index: usize) -> &'static [(i8, i8)] {
    match index {
        0 => &[
            (-1, -2), (0, -2), (1, -2), (-2, -1), (-1, -1), (0, -1), (1, -1), (2, -1), (-4, 0), (-3, 0), (-2, 0), (-1, 0),
        ],
        1 => &[
            (-1, -2), (0, -2), (1, -2), (2, -2), (-2, -1), (-1, -1), (0, -1), (1, -1), (2, -1), (-3, 0), (-2, 0), (-1, 0),
        ],
        2 => &[
            (-1, -2), (0, -2), (1, -2), (-2, -1), (-1, -1), (0, -1), (1, -1), (-2, 0), (-1, 0),
        ],
        3 => &[
            (-3, -1), (-2, -1), (-1, -1), (0, -1), (1, -1), (-4, 0), (-3, 0), (-2, 0), (-1, 0),
        ],
        _ => &[],
    }
}
fn decode_bitmap_template0(width: usize, height: usize, decoding_context: &mut DecodingContext) -> Result<Bitmap, Jbig2Error> {
    let mut decoder = decoding_context.get_decoder();
    let mut contexts = decoding_context.get_contexts("GB");
    let mut bitmap = Bitmap::new(width, height);
    for i in 0..height {
        let mut context_label = 0u16;
        if i >= 2 {
            let row2_y = i - 2;
            context_label |= (bitmap.get_pixel(0, row2_y) as u16) << 13;
            context_label |= (bitmap.get_pixel(1, row2_y) as u16) << 12;
            context_label |= (bitmap.get_pixel(2, row2_y) as u16) << 11;
        }
        if i >= 1 {
            let row1_y = i - 1;
            context_label |= (bitmap.get_pixel(0, row1_y) as u16) << 7;
            context_label |= (bitmap.get_pixel(1, row1_y) as u16) << 6;
            context_label |= (bitmap.get_pixel(2, row1_y) as u16) << 5;
            context_label |= (bitmap.get_pixel(3, row1_y) as u16) << 4;
        }
        for j in 0..width {
            let pixel = decoder.read_bit(contexts.as_mut(), context_label as usize);
            bitmap.set_pixel(j, i, pixel);
            let row2_contrib = if i >= 2 && j + 3 < width { (bitmap.get_pixel(j + 3, i - 2) as u16) << 11 } else { 0 };
            let row1_contrib = if i >= 1 && j + 4 < width { (bitmap.get_pixel(j + 4, i - 1) as u16) << 4 } else { 0 };
            context_label = ((context_label & OLD_PIXEL_MASK) << 1) | row2_contrib | row1_contrib | (pixel as u16);
        }
    }
    Ok(bitmap)
}
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
pub fn decode_bitmap(params: &DecodeBitmapParams, decoding_context: &mut DecodingContext) -> Result<Bitmap, Jbig2Error> {
    // Validate bitmap dimensions
    if params.width == 0 || params.height == 0 {
        return Err(Jbig2Error::new("invalid bitmap dimensions: width and height must be positive"));
    }
    if params.width > 65535 || params.height > 65535 {
        return Err(Jbig2Error::new("bitmap dimensions too large"));
    }

    if params.mmr {
        let mut reader = Reader::new(
            decoding_context.data.clone(),
            decoding_context.start,
            decoding_context.end,
        );
        return decode_mmr_bitmap(&mut reader, params.width, params.height, false);
    }
    // Use optimized version for the most common case
    if params.template_index == 0 && params.skip.is_none() && !params.prediction && params.at.len() == 4 &&
        params.at[0].0 == 3 && params.at[0].1 == -1 &&
        params.at[1].0 == -3 && params.at[1].1 == -1 &&
        params.at[2].0 == 2 && params.at[2].1 == -2 &&
        params.at[3].0 == -2 && params.at[3].1 == -2 {
        return decode_bitmap_template0(params.width, params.height, decoding_context);
    }
    let useskip = params.skip.is_some();
    let template = get_coding_template(params.template_index).iter().cloned().chain(params.at.clone()).collect::<Vec<_>>();
    let mut template = template;
    template.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
    let template_length = template.len();
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
        if k < template_length - 1 && template[k].1 == template[k + 1].1 && template[k].0 == template[k + 1].0 - 1 {
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
    let sbb_right = params.width - max_x as usize;
    let pseudo_pixel_context = REUSED_CONTEXTS[params.template_index];
    let mut bitmap = Bitmap::new(params.width, params.height);
    let mut decoder = decoding_context.get_decoder();
    let mut contexts = decoding_context.get_contexts("GB");
    let mut ltp = 0i32;
    for i in 0..params.height {
        if params.prediction && i > 0 {
            let sltp = decoder.read_bit(contexts.as_mut(), pseudo_pixel_context as usize) as i32;
            ltp ^= sltp;
            if ltp != 0 {
                let src_start = (i - 1) * bitmap.stride;
                let dst_start = i * bitmap.stride;
                let (before, after) = bitmap.data.split_at_mut(dst_start);
                let src_row = &before[src_start..src_start + bitmap.stride];
                let dst_row = &mut after[0..bitmap.stride];
                dst_row.copy_from_slice(src_row);
                continue;
            }
        }
        for j in 0..params.width {
            if useskip && params.skip.unwrap().get_pixel(j, i) != 0 {
                continue;
            }
            let context_label = if j >= sbb_left && j < sbb_right && i >= sbb_top {
                let mut context_label = 0u16;
                context_label = (context_label << 1) & reuse_mask;
                for k in 0..changing_entries_length {
                    let i0 = i as i32 + changing_template_y[k] as i32;
                    let j0 = j as i32 + changing_template_x[k] as i32;
                    if i0 >= 0 && i0 < params.height as i32 && j0 >= 0 && j0 < params.width as i32 && bitmap.get_pixel(j0 as usize, i0 as usize) != 0 {
                        context_label |= changing_template_bit[k];
                    }
                }
                context_label
            } else {
                let mut context_label = 0u16;
                let mut shift = template_length - 1;
                for k in 0..template_length {
                    let j0 = j as i32 + template_x[k] as i32;
                    if j0 >= 0 && j0 < params.width as i32 {
                        let i0 = i as i32 + template_y[k] as i32;
                        if i0 >= 0 && i0 < params.height as i32 && bitmap.get_pixel(j0 as usize, i0 as usize) != 0 {
                            context_label |= 1 << shift;
                        }
                    }
                    shift -= 1;
                }
                context_label
            };
            let pixel = decoder.read_bit(contexts.as_mut(), context_label as usize);
            bitmap.set_pixel(j, i, pixel);
        }
    }
    Ok(bitmap)
}