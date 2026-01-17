use crate::bitmap::Bitmap;
use crate::contexts::DecodingContext;
use crate::decode::decode_pattern::decode_pattern_dictionary;
use crate::error::Jbig2Error;
use std::path::PathBuf;

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

    if let Some(dir) = std::env::var_os("JBIG2_RS_DUMP_PATTERNS") {
        let dir = PathBuf::from(dir);
        std::fs::create_dir_all(&dir)
            .map_err(|e| Jbig2Error::new(&format!("pattern dump mkdir failed: {e}")))?;
        for (idx, pattern) in patterns_vec.iter().enumerate() {
            let mut out = String::new();
            out.push_str("P1\n");
            out.push_str(&format!("{} {}\n", pattern.width, pattern.height));
            for y in 0..pattern.height {
                for x in 0..pattern.width {
                    let bit = pattern.get_pixel(x, y);
                    out.push(if bit != 0 { '1' } else { '0' });
                    if x + 1 < pattern.width {
                        out.push(' ');
                    }
                }
                out.push('\n');
            }
            let path = dir.join(format!("pattern_{current_segment}_{idx}.pbm"));
            std::fs::write(&path, out).map_err(|e| {
                Jbig2Error::new(&format!("pattern dump write failed: {e}"))
            })?;
        }
    }

    patterns.insert(current_segment, patterns_vec);

    Ok(())
}
