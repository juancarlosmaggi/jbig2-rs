use crate::arithmetic::contexts::DecodingContext;
use crate::bitmap::Bitmap;
use crate::common::error::Jbig2Error;
use crate::decoders::halftone::{ShiftedPattern, build_shifted_patterns};
use crate::decoders::pattern::decode_pattern_dictionary;
use crate::parser::segment::PatternDictionaryParams;
use std::collections::HashMap;
use std::sync::Arc;

use super::SegmentSlice;

/// Decode a pattern dictionary segment and store the patterns.
pub(super) fn on_pattern_dictionary(
    patterns: &mut HashMap<u32, Vec<Bitmap>>,
    pattern_shifts: &mut HashMap<u32, Arc<Vec<ShiftedPattern>>>,
    params: &PatternDictionaryParams,
    current_segment: u32,
    bytes: SegmentSlice<'_>,
) -> Result<(), Jbig2Error> {
    let slice = bytes.as_slice();
    let mut decoding_context = DecodingContext::new(slice, 0, slice.len());

    let decode_params = crate::decoders::pattern::PatternDictionaryParams {
        mmr: params.mmr,
        pattern_width: params.pattern_width,
        pattern_height: params.pattern_height,
        max_pattern_index: params.max_pattern_index,
        template: params.template,
    };

    let patterns_vec = decode_pattern_dictionary(&decode_params, &mut decoding_context)?;

    let shifted_patterns = build_shifted_patterns(&patterns_vec);
    pattern_shifts.insert(current_segment, shifted_patterns);
    patterns.insert(current_segment, patterns_vec);

    Ok(())
}
