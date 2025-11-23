//! JBIG2 Segment Module
//!
//! This module handles parsing and processing of JBIG2 segments according to the
//! ITU-T T.88 specification (section 7 - Segment organization).
//!
//! ## Overview
//!
//! A JBIG2 data stream is organized into segments, where each segment contains either:
//! - Page information (dimensions, defaults)
//! - Symbol dictionaries (reusable glyph data)
//! - Text regions (text rendered using symbols)
//! - Generic regions (arbitrary bitmap data)
//! - Halftone regions (halftone patterns)
//! - Pattern dictionaries (patterns for halftone regions)
//! - Huffman tables (custom encoding tables)
//! - End-of-page/file markers
//!
//! ## Module Structure
//!
//! This module is organized into focused submodules:
//!
//! - **`types`** - Core data structures (`SegmentHeader`, `Segment`, `PageInfo`, etc.)
//! - **`parser`** - Segment header and data parsing functions
//! - **`processor`** - Segment dispatching and processing logic
//! - **`utils`** - Binary reading utilities (endianness handling)
//! - **`segment_params`** - Parameter structures for various segment types
//!
//! ## Segment Types
//!
//! Supported segment types (per ITU T.88 Table 2):
//! - Type 0: Symbol Dictionary
//! - Type 4-7: Text Regions (immediate and intermediate)
//! - Type 16: Pattern Dictionary
//! - Type 20-23: Halftone Regions
//! - Type 36-43: Generic Regions and Refinement Regions
//! - Type 48: Page Information
//! - Type 49-51: End-of-Page/Stripe/File
//! - Type 53: Huffman Tables
//! - Type 62: Extension
//!
//! ## Usage
//!
//! ```no_run
//! use jbig2_rs::segment::{read_segments, process_segments};
//! use jbig2_rs::visitor::SimpleSegmentVisitor;
//!
//! # fn example(data: &[u8]) -> Result<(), jbig2_rs::Jbig2Error> {
//! let segments = read_segments(data, 0, data.len(), true, 0, false)?;
//! let mut visitor = SimpleSegmentVisitor::new();
//! process_segments(&segments, &mut visitor)?;
//! # Ok(())
//! # }
//! ```

// Segment module - organized into focused submodules
mod parser;
mod processor;
mod segment_params;
mod types;
mod utils;

// Re-export public types and constants
pub use types::{
    ERR_INSUFFICIENT_DATA, ERR_INVALID_SEGMENT, ERR_MISMATCH, ERR_OVERRUN, ERR_UNKNOWN_LENGTH,
    GenericRegion, PageInfo, REGION_SEGMENT_INFORMATION_FIELD_LENGTH, RegionInfo, SEGMENT_TYPES,
    Segment, SegmentHeader, SymbolDictionaryParams,
};

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
