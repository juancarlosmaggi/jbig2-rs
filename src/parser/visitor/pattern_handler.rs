use crate::arithmetic::contexts::DecodingContext;
use crate::bitmap::Bitmap;
use crate::common::error::Jbig2Error;
use crate::decoders::halftone::{ShiftedPattern, build_shifted_patterns};
use crate::decoders::pattern::decode_pattern_dictionary;
use std::collections::HashMap;
use std::sync::Arc;

/// Decode a pattern dictionary segment and store the patterns.
#[allow(clippy::too_many_arguments)]
pub(super) fn on_pattern_dictionary(
    patterns: &mut HashMap<u32, Vec<Bitmap>>,
    pattern_shifts: &mut HashMap<u32, Arc<Vec<ShiftedPattern>>>,
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
    let mut decoding_context = DecodingContext::new(slice, 0, slice.len());

    let params = crate::decoders::pattern::PatternDictionaryParams {
        mmr,
        pattern_width,
        pattern_height,
        max_pattern_index,
        template,
    };

    let patterns_vec = decode_pattern_dictionary(&params, &mut decoding_context)?;

    let shifted_patterns = build_shifted_patterns(&patterns_vec);
    pattern_shifts.insert(current_segment, shifted_patterns);
    patterns.insert(current_segment, patterns_vec);

    Ok(())
}
