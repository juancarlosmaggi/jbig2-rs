// Parameter structures for segment parsing

use super::types::RegionInfo;

/// Parameters parsed from halftone region segment
#[derive(Debug)]
pub struct HalftoneRegionParams {
    pub region_info: RegionInfo,
    pub mmr: bool,
    pub template: usize,
    pub enable_skip: bool,
    pub combination_operator: usize,
    pub default_pixel_value: u8,
    pub grid_width: usize,
    pub grid_height: usize,
    pub grid_offset_x: i32,
    pub grid_offset_y: i32,
    pub grid_vector_x: i16,
    pub grid_vector_y: i16,
}

/// Parameters parsed from text region segment
#[derive(Debug)]
pub struct TextRegionParams {
    pub region_info: RegionInfo,
    pub text_region_segment_flags: u16,
    pub number_of_symbol_instances: u32,
}

/// Parameters parsed from pattern dictionary segment
#[derive(Debug)]
pub struct PatternDictionaryParams {
    pub mmr: bool,
    pub template: usize,
    pub pattern_width: usize,
    pub pattern_height: usize,
    pub max_pattern_index: usize,
}
