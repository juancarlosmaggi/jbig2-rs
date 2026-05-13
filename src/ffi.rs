use crate::common::error::{Jbig2Error, Jbig2ErrorCode};
use crate::{DecodeLimits, DecodeOptions, decode_page};
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

/// C ABI decoder options.
///
/// A zero limit uses the Rust default. `u64::MAX` disables that limit.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Jbig2FfiDecodeOptions {
    pub max_input_bytes: u64,
    pub max_decoded_pixels: u64,
    pub max_page_count: u64,
    pub max_segment_count: u64,
    pub max_symbol_dictionary_bytes: u64,
    pub max_intermediate_bitmap_bytes: u64,
    pub page_index: usize,
    pub collect_profile: u8,
}

/// C ABI decode result.
///
/// On success, `code == 0` and `data` points to packed 1bpp bytes owned by
/// Rust. On failure, `code != 0` and `error_message` describes the error.
/// Release every result with `jbig2_ffi_result_free`.
#[repr(C)]
pub struct Jbig2FfiResult {
    pub code: u32,
    pub width: u32,
    pub height: u32,
    pub stride: usize,
    pub page_index: usize,
    pub page_id: u32,
    pub data: *mut u8,
    pub data_len: usize,
    pub error_message: *mut c_char,
    pub profile_report: *mut c_char,
}

impl Jbig2FfiResult {
    fn success(page: crate::DecodedPage) -> Self {
        let profile_report = page
            .profile
            .map(|profile| into_c_string(profile.report()))
            .unwrap_or(ptr::null_mut());
        let mut data = page.data.into_boxed_slice();
        let data_len = data.len();
        let data_ptr = data.as_mut_ptr();
        std::mem::forget(data);

        Self {
            code: 0,
            width: page.width,
            height: page.height,
            stride: page.stride,
            page_index: page.page_index,
            page_id: page.page_id.unwrap_or(0),
            data: data_ptr,
            data_len,
            error_message: ptr::null_mut(),
            profile_report,
        }
    }

    fn failure(error: Jbig2Error) -> Self {
        Self {
            code: error.code() as u32,
            width: 0,
            height: 0,
            stride: 0,
            page_index: 0,
            page_id: 0,
            data: ptr::null_mut(),
            data_len: 0,
            error_message: into_c_string(error.to_string()),
            profile_report: ptr::null_mut(),
        }
    }
}

/// Return default FFI options. Callers may override any field before decoding.
#[unsafe(no_mangle)]
pub extern "C" fn jbig2_ffi_decode_options_default() -> Jbig2FfiDecodeOptions {
    Jbig2FfiDecodeOptions {
        max_input_bytes: 0,
        max_decoded_pixels: 0,
        max_page_count: 0,
        max_segment_count: 0,
        max_symbol_dictionary_bytes: 0,
        max_intermediate_bitmap_bytes: 0,
        page_index: 0,
        collect_profile: 0,
    }
}

/// Decode one JBIG2 page through the stable C ABI.
///
/// Inputs are borrowed for the duration of the call and are never freed by
/// Rust. If `global_len` is zero, `global_ptr` is ignored.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jbig2_ffi_decode_page(
    page_ptr: *const u8,
    page_len: usize,
    global_ptr: *const u8,
    global_len: usize,
    options: *const Jbig2FfiDecodeOptions,
) -> Jbig2FfiResult {
    let page_bytes = match unsafe { ffi_slice(page_ptr, page_len, "page_ptr") } {
        Ok(bytes) => bytes,
        Err(error) => return Jbig2FfiResult::failure(error),
    };
    let global_bytes = if global_len == 0 {
        None
    } else {
        match unsafe { ffi_slice(global_ptr, global_len, "global_ptr") } {
            Ok(bytes) => Some(bytes),
            Err(error) => return Jbig2FfiResult::failure(error),
        }
    };
    let options = unsafe { options_from_ffi(options) };

    match decode_page(page_bytes, global_bytes, options) {
        Ok(page) => Jbig2FfiResult::success(page),
        Err(error) => Jbig2FfiResult::failure(error),
    }
}

/// Free buffers owned by a C ABI decode result.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jbig2_ffi_result_free(result: Jbig2FfiResult) {
    if !result.data.is_null() {
        let slice = ptr::slice_from_raw_parts_mut(result.data, result.data_len);
        unsafe {
            drop(Box::from_raw(slice));
        }
    }
    if !result.error_message.is_null() {
        unsafe {
            drop(CString::from_raw(result.error_message));
        }
    }
    if !result.profile_report.is_null() {
        unsafe {
            drop(CString::from_raw(result.profile_report));
        }
    }
}

/// Return a static snake_case name for an error code.
#[unsafe(no_mangle)]
pub extern "C" fn jbig2_ffi_error_code_name(code: u32) -> *const c_char {
    match code {
        0 => c"ok".as_ptr(),
        x if x == Jbig2ErrorCode::InsufficientData as u32 => c"insufficient_data".as_ptr(),
        x if x == Jbig2ErrorCode::InvalidSegment as u32 => c"invalid_segment".as_ptr(),
        x if x == Jbig2ErrorCode::UnknownSegmentLength as u32 => c"unknown_segment_length".as_ptr(),
        x if x == Jbig2ErrorCode::InvalidFieldValue as u32 => c"invalid_field_value".as_ptr(),
        x if x == Jbig2ErrorCode::InvalidDimensions as u32 => c"invalid_dimensions".as_ptr(),
        x if x == Jbig2ErrorCode::DimensionsTooLarge as u32 => c"dimensions_too_large".as_ptr(),
        x if x == Jbig2ErrorCode::InvalidTemplateIndex as u32 => c"invalid_template_index".as_ptr(),
        x if x == Jbig2ErrorCode::InvalidCombinationOperator as u32 => {
            c"invalid_combination_operator".as_ptr()
        }
        x if x == Jbig2ErrorCode::InvalidReferenceCorner as u32 => {
            c"invalid_reference_corner".as_ptr()
        }
        x if x == Jbig2ErrorCode::MmrDecodingFailed as u32 => c"mmr_decoding_failed".as_ptr(),
        x if x == Jbig2ErrorCode::ArithmeticDecodingFailed as u32 => {
            c"arithmetic_decoding_failed".as_ptr()
        }
        x if x == Jbig2ErrorCode::HuffmanDecodingFailed as u32 => {
            c"huffman_decoding_failed".as_ptr()
        }
        x if x == Jbig2ErrorCode::InvalidRunLength as u32 => c"invalid_run_length".as_ptr(),
        x if x == Jbig2ErrorCode::TooManySymbols as u32 => c"too_many_symbols".as_ptr(),
        x if x == Jbig2ErrorCode::InfiniteLoopDetected as u32 => c"infinite_loop_detected".as_ptr(),
        x if x == Jbig2ErrorCode::BufferOverrun as u32 => c"buffer_overrun".as_ptr(),
        x if x == Jbig2ErrorCode::ResourceLimitExceeded as u32 => {
            c"resource_limit_exceeded".as_ptr()
        }
        x if x == Jbig2ErrorCode::Cancelled as u32 => c"cancelled".as_ptr(),
        x if x == Jbig2ErrorCode::MissingResource as u32 => c"missing_resource".as_ptr(),
        x if x == Jbig2ErrorCode::UnsupportedFeature as u32 => c"unsupported_feature".as_ptr(),
        _ => c"other".as_ptr(),
    }
}

unsafe fn ffi_slice<'a>(
    ptr: *const u8,
    len: usize,
    name: &'static str,
) -> Result<&'a [u8], Jbig2Error> {
    if len == 0 {
        return Ok(&[]);
    }
    if ptr.is_null() {
        return Err(Jbig2Error::invalid_field_value(name, "null"));
    }
    Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
}

unsafe fn options_from_ffi(options: *const Jbig2FfiDecodeOptions) -> DecodeOptions {
    let defaults = DecodeLimits::default();
    let ffi = if options.is_null() {
        jbig2_ffi_decode_options_default()
    } else {
        unsafe { *options }
    };

    let limits = DecodeLimits {
        max_input_bytes: limit_from_ffi(ffi.max_input_bytes, defaults.max_input_bytes),
        max_decoded_pixels: limit_from_ffi(ffi.max_decoded_pixels, defaults.max_decoded_pixels),
        max_page_count: limit_from_ffi(ffi.max_page_count, defaults.max_page_count),
        max_segment_count: limit_from_ffi(ffi.max_segment_count, defaults.max_segment_count),
        max_symbol_dictionary_bytes: limit_from_ffi(
            ffi.max_symbol_dictionary_bytes,
            defaults.max_symbol_dictionary_bytes,
        ),
        max_intermediate_bitmap_bytes: limit_from_ffi(
            ffi.max_intermediate_bitmap_bytes,
            defaults.max_intermediate_bitmap_bytes,
        ),
    };

    DecodeOptions::default()
        .with_limits(limits)
        .with_page_index(ffi.page_index)
        .with_profile(ffi.collect_profile != 0)
}

fn limit_from_ffi(value: u64, default: Option<usize>) -> Option<usize> {
    if value == 0 {
        default
    } else if value == u64::MAX {
        None
    } else {
        Some(value.min(usize::MAX as u64) as usize)
    }
}

fn into_c_string(value: String) -> *mut c_char {
    let sanitized = value.replace('\0', "\\0");
    CString::new(sanitized)
        .expect("sanitized string contains no nul bytes")
        .into_raw()
}
