// Segment module - organized into focused submodules

mod types;
mod utils;
mod parser;
mod processor;
mod segment_params;

// Re-export public types and constants
pub use types::{
    ERR_INSUFFICIENT_DATA,
    ERR_INVALID_SEGMENT,
    ERR_OVERRUN,
    ERR_MISMATCH,
    ERR_UNKNOWN_LENGTH,
    SEGMENT_TYPES,
    REGION_SEGMENT_INFORMATION_FIELD_LENGTH,
    SegmentHeader,
    Segment,
    PageInfo,
    RegionInfo,
    GenericRegion,
    SymbolDictionaryParams,
};

// Re-export public utility functions
pub use utils::{
    read_u32,
    read_u32_le,
    read_u16,
    read_u16_le,
    parse_at_parameters,
};

// Re-export public parser functions
pub use parser::{
    read_segment_header,
    read_region_segment_information,
    read_generic_region,
    read_segments,
    parse_halftone_region_params,
    parse_text_region_params,
    parse_pattern_dictionary_params,
};

// Re-export parameter structs
pub use segment_params::{
    HalftoneRegionParams,
    TextRegionParams,
    PatternDictionaryParams,
};

// Re-export public processor functions
pub use processor::{
    process_segments,
    process_segment,
};
