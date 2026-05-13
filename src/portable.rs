use crate::common::error::Jbig2Error;
use crate::common::options::{DecodeOptions, packed_bitmap_len};
use crate::common::profile::DecodeProfile;
use crate::document::{Jbig2Chunk, Jbig2Document};

/// Polarity used by packed bitmap output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitmapPolarity {
    /// A set bit (`1`) means black/foreground; a clear bit (`0`) means white/background.
    OneIsBlack,
}

/// Packed page output for embedders.
#[derive(Debug, Clone)]
pub struct DecodedPage {
    pub page_index: usize,
    pub page_id: Option<u32>,
    pub width: u32,
    pub height: u32,
    pub stride: usize,
    pub data: Vec<u8>,
    pub polarity: BitmapPolarity,
    pub profile: Option<DecodeProfile>,
}

impl DecodedPage {
    /// Expand packed 1bpp output into 8-bit grayscale (`0` black, `255` white).
    pub fn to_grayscale(&self) -> Vec<u8> {
        let width = self.width as usize;
        let height = self.height as usize;
        let mut pixels = vec![0u8; width * height];
        for y in 0..height {
            let row = &self.data[y * self.stride..y * self.stride + self.stride];
            for x in 0..width {
                let byte = row[x >> 3];
                let bit = (byte >> (7 - (x & 7))) & 1;
                pixels[y * width + x] = if bit == 1 { 0 } else { 255 };
            }
        }
        pixels
    }
}

/// Decode one JBIG2 page for native/mobile embedding.
///
/// `page_bytes` contains the page segment stream or a complete JBIG2 file.
/// `global_bytes`, when present, is decoded before the page bytes and is used
/// for PDF-style global dictionaries. Output is packed 1bpp, 8 pixels per byte,
/// MSB-first, with `stride = (width + 7) / 8`.
pub fn decode_page(
    page_bytes: &[u8],
    global_bytes: Option<&[u8]>,
    options: DecodeOptions,
) -> Result<DecodedPage, Jbig2Error> {
    options.check_cancelled()?;
    let total_input = page_bytes
        .len()
        .checked_add(global_bytes.map_or(0, <[u8]>::len))
        .ok_or_else(|| {
            Jbig2Error::resource_limit_exceeded("input bytes", usize::MAX, usize::MAX)
        })?;
    options.limits.check_input_bytes(total_input)?;

    let (document, profile) = if let Some(global_bytes) = global_bytes {
        let global_chunk = Jbig2Chunk {
            data: global_bytes.to_vec(),
            start: 0,
            end: global_bytes.len(),
        };
        let page_chunk = Jbig2Chunk {
            data: page_bytes.to_vec(),
            start: 0,
            end: page_bytes.len(),
        };
        if options.collect_profile {
            let (document, profile) = Jbig2Document::parse_chunks_with_options_and_profile(
                &[global_chunk, page_chunk],
                &options,
            )?;
            (document, Some(profile))
        } else {
            (
                Jbig2Document::parse_chunks_with_options(&[global_chunk, page_chunk], &options)?,
                None,
            )
        }
    } else if options.collect_profile {
        let (document, profile) =
            Jbig2Document::parse_with_options_and_profile(page_bytes, &options)?;
        (document, Some(profile))
    } else {
        (
            Jbig2Document::parse_with_options(page_bytes, &options)?,
            None,
        )
    };

    let page = document.get_page(options.page_index).ok_or_else(|| {
        Jbig2Error::invalid_field_value("page_index", options.page_index.to_string())
    })?;
    let width = page.page_info.width;
    let height = page.page_info.height;
    let stride = page.stride();
    let expected_len = packed_bitmap_len(width as usize, height as usize).ok_or_else(|| {
        Jbig2Error::resource_limit_exceeded("packed bitmap bytes", usize::MAX, usize::MAX)
    })?;
    if page.packed_bitmap().len() != expected_len {
        return Err(Jbig2Error::invalid_segment("packed bitmap length mismatch"));
    }

    Ok(DecodedPage {
        page_index: options.page_index,
        page_id: Some((options.page_index + 1) as u32),
        width,
        height,
        stride,
        data: page.packed_bitmap().to_vec(),
        polarity: BitmapPolarity::OneIsBlack,
        profile,
    })
}

/// Decode the first page with default limits and no global bytes.
pub fn decode_first_page(page_bytes: &[u8]) -> Result<DecodedPage, Jbig2Error> {
    decode_page(page_bytes, None, DecodeOptions::default())
}
