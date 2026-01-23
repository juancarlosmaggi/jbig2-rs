use crate::bitmap::Bitmap;
use crate::arithmetic::contexts::DecodingContext;
use crate::decoders::generic::{DecodeBitmapParams, decode_bitmap};
use crate::common::error::Jbig2Error;

/// Inputs required to decode a pattern dictionary.
#[derive(Clone)]
pub struct PatternDictionaryParams {
    pub mmr: bool,
    pub pattern_width: usize,
    pub pattern_height: usize,
    pub max_pattern_index: usize,
    pub template: usize,
}

/// Decode a pattern dictionary into individual pattern bitmaps.
pub fn decode_pattern_dictionary(
    params: &PatternDictionaryParams,
    decoding_context: &mut DecodingContext<'_>,
) -> Result<Vec<Bitmap>, Jbig2Error> {
    let at = if !params.mmr {
        let mut at_vec = vec![(-(params.pattern_width as i8), 0i8)];
        if params.template <= 1 {
            at_vec.extend(vec![(-3i8, -1i8), (2i8, -2i8), (-2i8, -2i8)]);
        }
        at_vec
    } else {
        vec![]
    };

    let collective_width = (params.max_pattern_index + 1) * params.pattern_width;
    let decode_params = DecodeBitmapParams {
        mmr: params.mmr,
        width: collective_width,
        height: params.pattern_height,
        template_index: params.template,
        prediction: false,
        skip: None,
        at: at.as_slice(),
    };
    let collective_bitmap = decode_bitmap(&decode_params, decoding_context)?;

    // Split the collective bitmap into individual pattern tiles.
    let mut patterns = Vec::with_capacity(params.max_pattern_index.saturating_add(1));
    let collective_stride = collective_bitmap.stride;
    let rem_bits = params.pattern_width & 7;
    let tail_mask = if rem_bits == 0 {
        0xFF
    } else {
        0xFFu8 << (8 - rem_bits)
    };
    for i in 0..=params.max_pattern_index {
        let x_start = i * params.pattern_width;
        let mut pattern = Bitmap::new(params.pattern_width, params.pattern_height);
        let pattern_stride = pattern.stride;
        if pattern_stride == 0 {
            patterns.push(pattern);
            continue;
        }
        let src_byte_offset = x_start >> 3;
        let src_bit_offset = (x_start & 7) as u8;
        for y in 0..params.pattern_height {
            let src_row_start = y * collective_stride + src_byte_offset;
            let src_row_end = y * collective_stride + collective_stride;
            let src_row = &collective_bitmap.data[src_row_start..src_row_end];
            debug_assert!(src_row.len() >= pattern_stride);
            let dst_row_start = y * pattern_stride;
            let dst_row = &mut pattern.data[dst_row_start..dst_row_start + pattern_stride];

            if src_bit_offset == 0 {
                dst_row.copy_from_slice(&src_row[..pattern_stride]);
            } else {
                let inv_shift = 8 - src_bit_offset;
                for b in 0..pattern_stride {
                    let cur = src_row[b];
                    let next = if b + 1 < src_row.len() {
                        src_row[b + 1]
                    } else {
                        0
                    };
                    dst_row[b] = (cur << src_bit_offset) | (next >> inv_shift);
                }
            }
            if rem_bits != 0 {
                dst_row[pattern_stride - 1] &= tail_mask;
            }
        }
        patterns.push(pattern);
    }
    Ok(patterns)
}
