use crate::bitmap::Bitmap;
use crate::contexts::DecodingContext;
use crate::decode::decode_pattern::decode_pattern_dictionary;
use crate::error::Jbig2Error;

/// Handle pattern dictionary segment
#[allow(clippy::too_many_arguments)]
pub(super) fn on_pattern_dictionary(
    patterns: &mut std::collections::HashMap<u32, Vec<Bitmap>>,
    mmr: bool,
    pattern_width: usize,
    pattern_height: usize,
    max_pattern_index: usize,
    template: usize,
    current_segment: u32,
    data: &[u8],
    start: usize,
    end: usize,
) -> Result<(), Jbig2Error> {
    let slice = &data[start..end];
    let mut decoding_context = DecodingContext::new(slice.to_vec(), 0, slice.len());

    let params = crate::decode::decode_pattern::PatternDictionaryParams {
        mmr,
        pattern_width,
        pattern_height,
        max_pattern_index,
        template,
    };

    let patterns_vec = decode_pattern_dictionary(&params, &mut decoding_context)?;

    patterns.insert(current_segment, patterns_vec);

    Ok(())
}
