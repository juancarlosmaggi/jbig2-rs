#![cfg(feature = "ffi")]

use jbig2_rs::ffi::{
    jbig2_ffi_decode_options_default, jbig2_ffi_decode_page, jbig2_ffi_error_code_name,
    jbig2_ffi_result_free,
};
use std::ffi::CStr;
use std::ptr;

fn segment(number: u32, segment_type: u8, page_association: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&number.to_be_bytes());
    out.push(segment_type);
    out.push(0);
    out.push(page_association);
    out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    out.extend_from_slice(payload);
    out
}

fn page_info_payload(width: u32, height: u32) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&width.to_be_bytes());
    payload.extend_from_slice(&height.to_be_bytes());
    payload.extend_from_slice(&300u32.to_be_bytes());
    payload.extend_from_slice(&300u32.to_be_bytes());
    payload.push(0);
    payload.extend_from_slice(&0u16.to_be_bytes());
    payload
}

#[test]
fn c_abi_smoke_decodes_and_frees_result() {
    let data = segment(1, 48, 1, &page_info_payload(8, 1));
    let mut options = jbig2_ffi_decode_options_default();
    options.collect_profile = 1;

    let result =
        unsafe { jbig2_ffi_decode_page(data.as_ptr(), data.len(), ptr::null(), 0, &options) };

    assert_eq!(result.code, 0);
    assert_eq!(result.width, 8);
    assert_eq!(result.height, 1);
    assert_eq!(result.stride, 1);
    assert_eq!(result.data_len, 1);
    assert!(!result.data.is_null());
    assert!(!result.profile_report.is_null());

    unsafe {
        jbig2_ffi_result_free(result);
    }
}

#[test]
fn c_abi_reports_stable_error_code_names() {
    let name = unsafe { CStr::from_ptr(jbig2_ffi_error_code_name(0)) };
    assert_eq!(name.to_str().unwrap(), "ok");
}
