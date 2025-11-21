//! Property-based tests for jbig2-rs
//!
//! These tests use proptest to verify invariants and find edge cases
//! that traditional unit tests might miss.

pub mod bitmap_properties;
pub mod reader_properties;
