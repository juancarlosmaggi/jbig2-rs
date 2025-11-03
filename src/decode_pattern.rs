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
    // Divide collective bitmap into patterns
    let mut patterns = Vec::new();
    for i in 0..=params.max_pattern_index {
        let mut pattern_bitmap = Vec::new();
        let x_min = params.pattern_width * i;
        let x_max = x_min + params.pattern_width;
        for y in 0..params.pattern_height {
            let row = collective_bitmap.data[y * collective_bitmap.stride..(y + 1) * collective_bitmap.stride]
                .iter()
                .skip(x_min / 8)
                .take(x_max.div_ceil(8) - x_min / 8)
                .cloned()
                .collect::<Vec<_>>();
            // For simplicity, create a new bitmap with the pattern
            // This is a simplified implementation
            let mut pattern = Bitmap::new(params.pattern_width, params.pattern_height);
            for py in 0..params.pattern_height {
                for px in 0..params.pattern_width {
                    let src_x = x_min + px;
                    let bit_index = 7 - (src_x & 7);
                    let byte_index = src_x >> 3;
                    if byte_index < row.len() {
                        let pixel = (row[byte_index] >> bit_index) & 1;
                        pattern.set_pixel(px, py, pixel);
                    }
                }
            }
            pattern_bitmap.push(pattern);
        }
        if !pattern_bitmap.is_empty() {
            patterns.push(pattern_bitmap[0].clone());
        }
    }
    Ok(patterns)
}