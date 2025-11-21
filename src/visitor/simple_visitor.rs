use crate::bitmap::Bitmap;
use crate::error::Jbig2Error;
use crate::huffman::HuffmanTable;
use crate::segment::{GenericRegion, PageInfo, RegionInfo, SymbolDictionaryParams};
use std::collections::HashMap;

// Re-export Jbig2Page from page_handler
pub use super::page_handler::Jbig2Page;

#[derive(Default)]
pub struct SimpleSegmentVisitor {
    pub pages: Vec<Jbig2Page>,
    pub current_page_info: Option<PageInfo>,
    pub current_bitmap: Option<Bitmap>,
    pub current_y: usize,
    pub symbols: HashMap<u32, Vec<Bitmap>>,
    pub patterns: HashMap<u32, Vec<Bitmap>>,
    pub custom_tables: HashMap<u32, HuffmanTable>,
    pub bitmaps: HashMap<u32, Bitmap>,
}

impl SimpleSegmentVisitor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_page_information(&mut self, info: PageInfo) {
        super::page_handler::on_page_information(
            &mut self.current_page_info,
            &mut self.current_bitmap,
            &mut self.current_y,
            &mut self.pages,
            info,
        );
    }

    pub fn on_end_of_stripe(&mut self, height: usize) {
        super::page_handler::on_end_of_stripe(&mut self.current_y, height);
    }

    pub fn draw_bitmap(
        &mut self,
        region_info: &RegionInfo,
        src_bitmap: &Bitmap,
    ) -> Result<(), Jbig2Error> {
        super::region_handlers::draw_bitmap(
            &self.current_page_info,
            &mut self.current_bitmap,
            self.current_y,
            region_info,
            src_bitmap,
        )
    }

    pub fn on_immediate_generic_region(
        &mut self,
        region: &GenericRegion,
        data: &[u8],
        start: usize,
        end: usize,
    ) -> Result<(), Jbig2Error> {
        super::region_handlers::on_immediate_generic_region(
            &mut self.current_page_info,
            &mut self.current_bitmap,
            self.current_y,
            region,
            data,
            start,
            end,
        )
    }

    pub fn on_immediate_generic_refinement_region(
        &mut self,
        region_info: &RegionInfo,
        referred_to: &[u32],
        data: &[u8],
        start: usize,
        end: usize,
    ) -> Result<(), Jbig2Error> {
        super::region_handlers::on_immediate_generic_refinement_region(
            &mut self.current_page_info,
            &mut self.current_bitmap,
            self.current_y,
            &self.bitmaps,
            region_info,
            referred_to,
            data,
            start,
            end,
        )
    }

    pub fn on_symbol_dictionary(
        &mut self,
        params: &SymbolDictionaryParams,
    ) -> Result<(), Jbig2Error> {
        super::symbol_handler::on_symbol_dictionary(&mut self.symbols, &self.custom_tables, params)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn on_immediate_text_region(
        &mut self,
        region_info: &RegionInfo,
        text_region_segment_flags: u16,
        number_of_symbol_instances: u32,
        referred_to: &[u32],
        data: &[u8],
        start: usize,
        end: usize,
    ) -> Result<(), Jbig2Error> {
        super::text_handler::on_immediate_text_region(
            &mut self.current_page_info,
            &mut self.current_bitmap,
            self.current_y,
            &self.symbols,
            &self.custom_tables,
            region_info,
            text_region_segment_flags,
            number_of_symbol_instances,
            referred_to,
            data,
            start,
            end,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn on_pattern_dictionary(
        &mut self,
        mmr: bool,
        pattern_width: usize,
        pattern_height: usize,
        max_pattern_index: usize,
        template: usize,
        current_segment: u32,
        data: &[u8],
        start: usize,
        end: usize,
    ) -> Result<(), Jbig2Error> {
        super::pattern_handler::on_pattern_dictionary(
            &mut self.patterns,
            mmr,
            pattern_width,
            pattern_height,
            max_pattern_index,
            template,
            current_segment,
            data,
            start,
            end,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn on_immediate_halftone_region(
        &mut self,
        region_info: &RegionInfo,
        mmr: bool,
        template: usize,
        enable_skip: bool,
        combination_operator: usize,
        default_pixel_value: u8,
        grid_width: usize,
        grid_height: usize,
        grid_offset_x: i32,
        grid_offset_y: i32,
        grid_vector_x: i16,
        grid_vector_y: i16,
        referred_to: &[u32],
        data: &[u8],
        start: usize,
        end: usize,
    ) -> Result<(), Jbig2Error> {
        super::halftone_handler::on_immediate_halftone_region(
            &mut self.current_page_info,
            &mut self.current_bitmap,
            self.current_y,
            &self.patterns,
            region_info,
            mmr,
            template,
            enable_skip,
            combination_operator,
            default_pixel_value,
            grid_width,
            grid_height,
            grid_offset_x,
            grid_offset_y,
            grid_vector_x,
            grid_vector_y,
            referred_to,
            data,
            start,
            end,
        )
    }

    pub fn on_tables(
        &mut self,
        segment_number: u32,
        data: &[u8],
        start: usize,
        end: usize,
    ) -> Result<(), Jbig2Error> {
        super::tables_handler::on_tables(&mut self.custom_tables, segment_number, data, start, end)
    }

    pub fn on_intermediate_generic_region(
        &mut self,
        region: &GenericRegion,
        referred_to: &[u32],
        data: &[u8],
        start: usize,
        end: usize,
        segment_number: u32,
    ) -> Result<(), Jbig2Error> {
        super::region_handlers::on_intermediate_generic_region(
            &self.symbols,
            &self.patterns,
            &self.custom_tables,
            &mut self.bitmaps,
            region,
            referred_to,
            data,
            start,
            end,
            segment_number,
        )
    }

    pub fn on_intermediate_generic_refinement_region(
        &mut self,
        region_info: &RegionInfo,
        referred_to: &[u32],
        data: &[u8],
        start: usize,
        end: usize,
        segment_number: u32,
    ) -> Result<(), Jbig2Error> {
        super::region_handlers::on_intermediate_generic_refinement_region(
            &mut self.bitmaps,
            region_info,
            referred_to,
            data,
            start,
            end,
            segment_number,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn on_intermediate_text_region(
        &mut self,
        region_info: &RegionInfo,
        text_region_segment_flags: u16,
        number_of_symbol_instances: u32,
        referred_to: &[u32],
        data: &[u8],
        start: usize,
        end: usize,
        _segment_number: u32,
    ) -> Result<(), Jbig2Error> {
        super::text_handler::on_intermediate_text_region(
            &self.symbols,
            &self.patterns,
            &self.custom_tables,
            &self.bitmaps,
            region_info,
            text_region_segment_flags,
            number_of_symbol_instances,
            referred_to,
            data,
            start,
            end,
            _segment_number,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn on_intermediate_halftone_region(
        &mut self,
        region_info: &RegionInfo,
        mmr: bool,
        template: usize,
        enable_skip: bool,
        combination_operator: usize,
        default_pixel_value: u8,
        grid_width: usize,
        grid_height: usize,
        grid_offset_x: i32,
        grid_offset_y: i32,
        grid_vector_x: i16,
        grid_vector_y: i16,
        referred_to: &[u32],
        data: &[u8],
        start: usize,
        end: usize,
        segment_number: u32,
    ) -> Result<(), Jbig2Error> {
        super::halftone_handler::on_intermediate_halftone_region(
            &self.patterns,
            &mut self.bitmaps,
            region_info,
            mmr,
            template,
            enable_skip,
            combination_operator,
            default_pixel_value,
            grid_width,
            grid_height,
            grid_offset_x,
            grid_offset_y,
            grid_vector_x,
            grid_vector_y,
            referred_to,
            data,
            start,
            end,
            segment_number,
        )
    }

    // Finalize the current page and add it to the pages vector
    pub fn finalize_current_page(&mut self) {
        super::page_handler::finalize_current_page(
            &mut self.current_page_info,
            &mut self.current_bitmap,
            &mut self.pages,
        );
    }
}
