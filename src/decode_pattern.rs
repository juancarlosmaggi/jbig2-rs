use crate::bitmap::Bitmap;
use crate::contexts::DecodingContext;
use crate::decode_generic::{decode_bitmap, DecodeBitmapParams};
use crate::error::Jbig2Error;

#[derive(Clone)]
pub struct PatternDictionaryParams {
    pub mmr: bool,
    pub pattern_width: usize,
    pub pattern_height: usize,
    pub max_pattern_index: usize,
    pub template: usize,
}

pub fn decode_pattern_dictionary(
    params: &PatternDictionaryParams,
    decoding_context: &mut DecodingContext,
) -> Result<Vec<Bitmap>, Jbig2Error> {
    let at = if !params.mmr {
        let mut at_vec = vec![(-(params.pattern_width as i8), 0i8)];
        if params.template == 0 {
            at_vec.extend(vec![
                (-3i8, -1i8),
                (2i8, -2i8),
                (-2i8, -2i8),
            ]);
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
        at,
    };
    let collective_bitmap = decode_bitmap(&decode_params, decoding_context)?;

    // Divide collective bitmap into individual patterns
    let mut patterns = Vec::new();
    for i in 0..=params.max_pattern_index {
        let x_start = i * params.pattern_width;
        let mut pattern = Bitmap::new(params.pattern_width, params.pattern_height);

        for y in 0..params.pattern_height {
            let collective_byte_offset = y * collective_bitmap.stride + (x_start >> 3);

            // Extract bytes for this row of the pattern, handling bit alignment
            for px in 0..params.pattern_width {
                let collective_x = x_start + px;
                let collective_byte_idx = collective_x >> 3;
                let collective_bit_idx = 7 - (collective_x & 7);
                let byte_idx_in_row = collective_byte_idx - (x_start >> 3);

                let pixel = if byte_idx_in_row < collective_bitmap.data[collective_byte_offset..].len() {
                    (collective_bitmap.data[collective_byte_offset + byte_idx_in_row] >> collective_bit_idx) & 1
                } else {
                    0
                };
                pattern.set_pixel(px, y, pixel);
            }
        }
        patterns.push(pattern);
    }
    Ok(patterns)
}