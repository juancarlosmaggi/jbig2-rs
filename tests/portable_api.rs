use jbig2_rs::{
    BitmapPolarity, DecodeLimits, DecodeOptions, Jbig2Document, Jbig2ErrorCode, decode_page,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

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

fn page_info_segment(number: u32, page_association: u8, width: u32, height: u32) -> Vec<u8> {
    segment(
        number,
        48,
        page_association,
        &page_info_payload(width, height),
    )
}

#[test]
fn portable_decode_returns_packed_bitmap() {
    let data = page_info_segment(1, 1, 8, 1);
    let page = decode_page(&data, None, DecodeOptions::default()).unwrap();

    assert_eq!(page.page_index, 0);
    assert_eq!(page.page_id, Some(1));
    assert_eq!(page.width, 8);
    assert_eq!(page.height, 1);
    assert_eq!(page.stride, 1);
    assert_eq!(page.data, vec![0]);
    assert_eq!(page.polarity, BitmapPolarity::OneIsBlack);
}

#[test]
fn portable_decode_accepts_global_and_page_segments() {
    let globals = segment(1, 62, 0, &[]);
    let page_data = page_info_segment(2, 1, 9, 2);
    let page = decode_page(&page_data, Some(&globals), DecodeOptions::default()).unwrap();

    assert_eq!(page.width, 9);
    assert_eq!(page.height, 2);
    assert_eq!(page.stride, 2);
    assert_eq!(page.data.len(), 4);
}

#[test]
fn portable_decode_selects_page_index() {
    let mut data = page_info_segment(1, 1, 8, 1);
    data.extend_from_slice(&page_info_segment(2, 2, 9, 2));

    let options = DecodeOptions::default().with_page_index(1);
    let page = decode_page(&data, None, options).unwrap();

    assert_eq!(page.page_index, 1);
    assert_eq!(page.page_id, Some(2));
    assert_eq!(page.width, 9);
    assert_eq!(page.height, 2);
}

#[test]
fn over_budget_stream_returns_structured_error() {
    let data = page_info_segment(1, 1, 8, 1);
    let limits = DecodeLimits {
        max_decoded_pixels: Some(7),
        ..DecodeLimits::default()
    };
    let options = DecodeOptions::default().with_limits(limits);
    let error = decode_page(&data, None, options).unwrap_err();

    assert_eq!(error.code(), Jbig2ErrorCode::ResourceLimitExceeded);
    assert_eq!(error.code_name(), "resource_limit_exceeded");
}

#[test]
fn cancelled_stream_returns_structured_error() {
    let data = page_info_segment(1, 1, 8, 1);
    let cancel = Arc::new(AtomicBool::new(true));
    let options = DecodeOptions::default().with_cancel_flag(cancel);
    let error = decode_page(&data, None, options).unwrap_err();

    assert_eq!(error.code(), Jbig2ErrorCode::Cancelled);
}

#[test]
fn malformed_and_truncated_streams_return_errors() {
    let malformed = [0xff, 0x00, 0x10, 0x20];
    assert!(Jbig2Document::parse(&malformed).is_err());

    let mut truncated = segment(1, 48, 1, &page_info_payload(8, 1));
    truncated.truncate(truncated.len() - 3);
    let error = match Jbig2Document::parse(&truncated) {
        Ok(_) => panic!("truncated stream unexpectedly decoded"),
        Err(error) => error,
    };
    assert_eq!(error.code(), Jbig2ErrorCode::InsufficientData);
}

#[test]
fn cancellation_can_be_flipped_before_decode() {
    let data = page_info_segment(1, 1, 8, 1);
    let cancel = Arc::new(AtomicBool::new(false));
    let options = DecodeOptions::default().with_cancel_flag(cancel.clone());
    cancel.store(true, Ordering::Relaxed);
    let error = decode_page(&data, None, options).unwrap_err();

    assert_eq!(error.code(), Jbig2ErrorCode::Cancelled);
}
