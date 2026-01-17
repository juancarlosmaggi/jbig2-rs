// Data structures and constants for segment handling.

pub const ERR_INSUFFICIENT_DATA: &str = "insufficient data";
pub const ERR_INVALID_SEGMENT: &str = "invalid segment";
pub const ERR_OVERRUN: &str = "segment overruns data";
pub const ERR_MISMATCH: &str = "data mismatch";
pub const ERR_UNKNOWN_LENGTH: &str = "invalid unknown segment length";

/// Segment type names indexed by type id.
pub const SEGMENT_TYPES: [&str; 63] = [
    "SymbolDictionary",
    "",
    "",
    "",
    "IntermediateTextRegion",
    "",
    "ImmediateTextRegion",
    "ImmediateLosslessTextRegion",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "PatternDictionary",
    "",
    "",
    "",
    "IntermediateHalftoneRegion",
    "",
    "ImmediateHalftoneRegion",
    "ImmediateLosslessHalftoneRegion",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "IntermediateGenericRegion",
    "",
    "ImmediateGenericRegion",
    "ImmediateLosslessGenericRegion",
    "IntermediateGenericRefinementRegion",
    "",
    "ImmediateGenericRefinementRegion",
    "ImmediateLosslessGenericRefinementRegion",
    "",
    "",
    "",
    "",
    "PageInformation",
    "EndOfPage",
    "EndOfStripe",
    "EndOfFile",
    "Profiles",
    "Tables",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "Extension",
];

/// Byte length of the fixed region information block.
pub const REGION_SEGMENT_INFORMATION_FIELD_LENGTH: usize = 17;

/// Parsed segment header metadata.
#[derive(Debug, Clone)]
pub struct SegmentHeader {
    pub number: u32,
    pub segment_type: usize,
    pub type_name: String,
    pub deferred_non_retain: bool,
    pub retain_bits: Vec<u8>,
    pub referred_to: Vec<u32>,
    pub page_association: u32,
    pub length: usize,
    pub header_end: usize,
}

/// A segment with its header and slice boundaries into the source data.
#[derive(Clone)]
pub struct Segment<'a> {
    pub header: SegmentHeader,
    pub data: &'a [u8],
    pub start: usize,
    pub end: usize,
}

/// Page-level metadata extracted from the page information segment.
#[derive(Debug, Clone)]
pub struct PageInfo {
    pub width: u32,
    pub height: u32,
    pub resolution_x: u32,
    pub resolution_y: u32,
    pub lossless: bool,
    pub refinement: bool,
    pub default_pixel_value: u8,
    pub combination_operator: u8,
    pub requires_buffer: bool,
    pub combination_operator_override: bool,
    pub striped: bool,
    pub stripe_size: u16,
    pub height_unknown: bool,
}

/// Region geometry and composition parameters.
#[derive(Debug, Clone)]
pub struct RegionInfo {
    pub width: u32,
    pub height: u32,
    pub x: u32,
    pub y: u32,
    pub combination_operator: u8,
}

/// Parsed generic region parameters.
#[derive(Debug)]
pub struct GenericRegion {
    pub info: RegionInfo,
    pub mmr: bool,
    pub template: usize,
    pub prediction: bool,
    pub at: Vec<(i8, i8)>,
}

/// Parameters and payload bounds for symbol dictionary segments.
#[derive(Clone)]
pub struct SymbolDictionaryParams<'a> {
    pub dictionary_flags: u16,
    pub number_of_exported_symbols: u32,
    pub number_of_new_symbols: u32,
    pub current_segment: u32,
    pub referred_segments: &'a [u32],
    pub data: &'a [u8],
    pub start: usize,
    pub end: usize,
    pub at_pixels: Vec<(i8, i8)>,
    pub refinement_at_pixels: Vec<(i8, i8)>,
}
