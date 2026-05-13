use crate::common::error::Jbig2Error;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// Decoder resource limits for untrusted JBIG2 input.
///
/// `None` disables an individual limit. The default values are intended to be
/// conservative enough for mobile embedding while still covering the repository
/// fixtures, including the larger UBC test files.
#[derive(Debug, Clone)]
pub struct DecodeLimits {
    pub max_input_bytes: Option<usize>,
    pub max_decoded_pixels: Option<usize>,
    pub max_page_count: Option<usize>,
    pub max_segment_count: Option<usize>,
    pub max_symbol_dictionary_bytes: Option<usize>,
    pub max_intermediate_bitmap_bytes: Option<usize>,
}

impl Default for DecodeLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: Some(128 * 1024 * 1024),
            max_decoded_pixels: Some(250_000_000),
            max_page_count: Some(256),
            max_segment_count: Some(250_000),
            max_symbol_dictionary_bytes: Some(128 * 1024 * 1024),
            max_intermediate_bitmap_bytes: Some(128 * 1024 * 1024),
        }
    }
}

impl DecodeLimits {
    /// Return limits with every budget disabled.
    pub fn unbounded() -> Self {
        Self {
            max_input_bytes: None,
            max_decoded_pixels: None,
            max_page_count: None,
            max_segment_count: None,
            max_symbol_dictionary_bytes: None,
            max_intermediate_bitmap_bytes: None,
        }
    }

    pub(crate) fn check_input_bytes(&self, actual: usize) -> Result<(), Jbig2Error> {
        check_limit("input bytes", self.max_input_bytes, actual)
    }

    pub(crate) fn check_page_count(&self, actual: usize) -> Result<(), Jbig2Error> {
        check_limit("page count", self.max_page_count, actual)
    }

    pub(crate) fn check_segment_count(&self, actual: usize) -> Result<(), Jbig2Error> {
        check_limit("segment count", self.max_segment_count, actual)
    }

    pub(crate) fn check_pixels(
        &self,
        width: usize,
        height: usize,
        resource: &'static str,
    ) -> Result<(), Jbig2Error> {
        let actual = width
            .checked_mul(height)
            .ok_or_else(|| Jbig2Error::resource_limit_exceeded(resource, usize::MAX, usize::MAX))?;
        check_limit(resource, self.max_decoded_pixels, actual)
    }

    pub(crate) fn check_bitmap_bytes(
        &self,
        width: usize,
        height: usize,
        resource: &'static str,
    ) -> Result<(), Jbig2Error> {
        let actual = packed_bitmap_len(width, height)
            .ok_or_else(|| Jbig2Error::resource_limit_exceeded(resource, usize::MAX, usize::MAX))?;
        check_limit(resource, self.max_intermediate_bitmap_bytes, actual)
    }

    pub(crate) fn check_symbol_dictionary_bytes(&self, actual: usize) -> Result<(), Jbig2Error> {
        check_limit(
            "symbol dictionary bytes",
            self.max_symbol_dictionary_bytes,
            actual,
        )
    }
}

fn check_limit(
    resource: &'static str,
    limit: Option<usize>,
    actual: usize,
) -> Result<(), Jbig2Error> {
    if let Some(limit) = limit
        && actual > limit
    {
        return Err(Jbig2Error::resource_limit_exceeded(resource, limit, actual));
    }
    Ok(())
}

pub(crate) fn packed_bitmap_len(width: usize, height: usize) -> Option<usize> {
    let stride = width.checked_add(7)? >> 3;
    stride.checked_mul(height)
}

/// Options for high-level embedding and bounded document decoding.
#[derive(Debug, Clone)]
pub struct DecodeOptions {
    pub limits: DecodeLimits,
    pub page_index: usize,
    pub collect_profile: bool,
    cancel_flag: Option<Arc<AtomicBool>>,
}

impl Default for DecodeOptions {
    fn default() -> Self {
        Self {
            limits: DecodeLimits::default(),
            page_index: 0,
            collect_profile: false,
            cancel_flag: None,
        }
    }
}

impl DecodeOptions {
    pub fn with_limits(mut self, limits: DecodeLimits) -> Self {
        self.limits = limits;
        self
    }

    pub fn with_page_index(mut self, page_index: usize) -> Self {
        self.page_index = page_index;
        self
    }

    pub fn with_profile(mut self, collect_profile: bool) -> Self {
        self.collect_profile = collect_profile;
        self
    }

    pub fn with_cancel_flag(mut self, cancel_flag: Arc<AtomicBool>) -> Self {
        self.cancel_flag = Some(cancel_flag);
        self
    }

    pub(crate) fn cancel_flag(&self) -> Option<Arc<AtomicBool>> {
        self.cancel_flag.clone()
    }

    pub(crate) fn check_cancelled(&self) -> Result<(), Jbig2Error> {
        if let Some(flag) = &self.cancel_flag
            && flag.load(Ordering::Relaxed)
        {
            return Err(Jbig2Error::cancelled());
        }
        Ok(())
    }
}

pub(crate) fn check_cancelled(flag: &Option<Arc<AtomicBool>>) -> Result<(), Jbig2Error> {
    if let Some(flag) = flag
        && flag.load(Ordering::Relaxed)
    {
        return Err(Jbig2Error::cancelled());
    }
    Ok(())
}
