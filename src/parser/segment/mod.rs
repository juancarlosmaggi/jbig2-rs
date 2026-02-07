//! Segment parsing and dispatch support.
//!
//! This module owns the structures and helpers for reading segment headers,
//! parsing segment payloads, and forwarding decoded content to visitors.

// Segment module - organized into focused submodules
mod parser;
mod processor;
mod segment_params;
mod types;
mod utils;

// Re-export public types and constants
pub use types::{
    ERR_INSUFFICIENT_DATA, ERR_INVALID_SEGMENT, ERR_MISMATCH, ERR_OVERRUN, ERR_UNKNOWN_LENGTH,
    GenericRegion, REGION_SEGMENT_INFORMATION_FIELD_LENGTH, RegionInfo, SEGMENT_TYPES, Segment,
    SegmentHeader, SymbolDictionaryParams,
};

pub use crate::document::PageInfo;

// Re-export public utility functions
pub use utils::{parse_at_parameters, read_u16, read_u16_le, read_u32, read_u32_le};

// Re-export public parser functions
pub use parser::{
    parse_halftone_region_params, parse_pattern_dictionary_params, parse_text_region_params,
    read_generic_region, read_region_segment_information, read_segment_header, read_segments,
};

// Re-export parameter structs
pub use segment_params::{HalftoneRegionParams, PatternDictionaryParams, TextRegionParams};

// Re-export public processor functions
pub use processor::{process_segment, process_segments};
