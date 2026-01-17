use crate::bitmap::Bitmap;
use crate::error::Jbig2Error;
use crate::huffman::HuffmanTable;
use crate::profile::DecodeProfile;
use crate::segment::{GenericRegion, PageInfo, RegionInfo, SymbolDictionaryParams};
use std::collections::HashMap;
use std::time::{Duration, Instant};

// Re-export Jbig2Page from page_handler.
pub use super::page_handler::Jbig2Page;

/// Visitor that accumulates decoded segments into pages.
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
    profile: Option<DecodeProfile>,
}

impl SimpleSegmentVisitor {
    /// Create a visitor with empty state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a visitor with decode profiling enabled.
    pub fn new_with_profile() -> Self {
        Self {
            profile: Some(DecodeProfile::default()),
            ..Default::default()
        }
    }

    pub fn record_profile(&mut self, label: &'static str, duration: Duration) {
        if let Some(profile) = self.profile.as_mut() {
            profile.record(label, duration);
        }
    }

    pub fn take_profile(&mut self) -> Option<DecodeProfile> {
        self.profile.take()
    }

    fn time_call<T, F>(&mut self, label: &'static str, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        let start = self.profile.is_some().then(Instant::now);
        let result = f(self);
        if let (Some(start), Some(profile)) = (start, self.profile.as_mut()) {
            profile.record(label, start.elapsed());
        }
        result
    }

    /// Apply page information to the current decode state.
    pub fn on_page_information(&mut self, info: PageInfo) {
        self.time_call("page_information", |this| {
            super::page_handler::on_page_information(
                &mut this.current_page_info,
                &mut this.current_bitmap,
                &mut this.current_y,
                &mut this.pages,
                info,
            );
        });
    }

    /// Advance the current stripe offset.
    pub fn on_end_of_stripe(&mut self, end_row: usize) {
        self.time_call("end_of_stripe", |this| {
            super::page_handler::on_end_of_stripe(
                &mut this.current_page_info,
                &mut this.current_bitmap,
                &mut this.current_y,
                end_row,
            );
        });
    }

    /// Composite a bitmap onto the current page.
    pub fn draw_bitmap(
        &mut self,
        region_info: &RegionInfo,
        src_bitmap: &Bitmap,
    ) -> Result<(), Jbig2Error> {
        self.time_call("draw_bitmap", |this| {
            super::region_handlers::draw_bitmap(
                &this.current_page_info,
                &mut this.current_bitmap,
                this.current_y,
                region_info,
                src_bitmap,
            )
        })
    }

    /// Decode and draw an immediate generic region.
    pub fn on_immediate_generic_region(
        &mut self,
        region: &GenericRegion,
        data: &[u8],
        start: usize,
        end: usize,
    ) -> Result<(), Jbig2Error> {
        self.time_call("immediate_generic_region", |this| {
            super::region_handlers::on_immediate_generic_region(
                &mut this.current_page_info,
                &mut this.current_bitmap,
                this.current_y,
                region,
                data,
                start,
                end,
            )
        })
    }

    /// Decode and draw an immediate generic refinement region.
    pub fn on_immediate_generic_refinement_region(
        &mut self,
        region_info: &RegionInfo,
        referred_to: &[u32],
        data: &[u8],
        start: usize,
        end: usize,
    ) -> Result<(), Jbig2Error> {
        self.time_call("immediate_generic_refinement_region", |this| {
            super::region_handlers::on_immediate_generic_refinement_region(
                &mut this.current_page_info,
                &mut this.current_bitmap,
                this.current_y,
                &this.bitmaps,
                region_info,
                referred_to,
                data,
                start,
                end,
            )
        })
    }

    /// Decode and store a symbol dictionary segment.
    pub fn on_symbol_dictionary(
        &mut self,
        params: &SymbolDictionaryParams,
    ) -> Result<(), Jbig2Error> {
        self.time_call("symbol_dictionary", |this| {
            super::symbol_handler::on_symbol_dictionary(
                &mut this.symbols,
                &this.custom_tables,
                params,
            )
        })
    }

    #[allow(clippy::too_many_arguments)]
    /// Decode and draw an immediate text region.
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
        self.time_call("immediate_text_region", |this| {
            super::text_handler::on_immediate_text_region(
                &mut this.current_page_info,
                &mut this.current_bitmap,
                this.current_y,
                &this.symbols,
                &this.custom_tables,
                region_info,
                text_region_segment_flags,
                number_of_symbol_instances,
                referred_to,
                data,
                start,
                end,
            )
        })
    }

    #[allow(clippy::too_many_arguments)]
    /// Decode and store a pattern dictionary segment.
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
        self.time_call("pattern_dictionary", |this| {
            super::pattern_handler::on_pattern_dictionary(
                &mut this.patterns,
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
        })
    }

    #[allow(clippy::too_many_arguments)]
    /// Decode and draw an immediate halftone region.
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
        self.time_call("immediate_halftone_region", |this| {
            super::halftone_handler::on_immediate_halftone_region(
                &mut this.current_page_info,
                &mut this.current_bitmap,
                this.current_y,
                &this.patterns,
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
        })
    }

    pub fn on_tables(
        &mut self,
        segment_number: u32,
        data: &[u8],
        start: usize,
        end: usize,
    ) -> Result<(), Jbig2Error> {
        self.time_call("tables", |this| {
            super::tables_handler::on_tables(
                &mut this.custom_tables,
                segment_number,
                data,
                start,
                end,
            )
        })
    }

    /// Decode and store an intermediate generic region.
    pub fn on_intermediate_generic_region(
        &mut self,
        region: &GenericRegion,
        referred_to: &[u32],
        data: &[u8],
        start: usize,
        end: usize,
        segment_number: u32,
    ) -> Result<(), Jbig2Error> {
        self.time_call("intermediate_generic_region", |this| {
            super::region_handlers::on_intermediate_generic_region(
                &this.symbols,
                &this.patterns,
                &this.custom_tables,
                &mut this.bitmaps,
                region,
                referred_to,
                data,
                start,
                end,
                segment_number,
            )
        })
    }

    /// Decode and store an intermediate generic refinement region.
    pub fn on_intermediate_generic_refinement_region(
        &mut self,
        region_info: &RegionInfo,
        referred_to: &[u32],
        data: &[u8],
        start: usize,
        end: usize,
        segment_number: u32,
    ) -> Result<(), Jbig2Error> {
        self.time_call("intermediate_generic_refinement_region", |this| {
            super::region_handlers::on_intermediate_generic_refinement_region(
                &mut this.bitmaps,
                region_info,
                referred_to,
                data,
                start,
                end,
                segment_number,
            )
        })
    }

    #[allow(clippy::too_many_arguments)]
    /// Decode an intermediate text region (no page compositing).
    pub fn on_intermediate_text_region(
        &mut self,
        region_info: &RegionInfo,
        text_region_segment_flags: u16,
        number_of_symbol_instances: u32,
        referred_to: &[u32],
        data: &[u8],
        start: usize,
        end: usize,
        segment_number: u32,
    ) -> Result<(), Jbig2Error> {
        self.time_call("intermediate_text_region", |this| {
            super::text_handler::on_intermediate_text_region(
                &this.symbols,
                &this.patterns,
                &this.custom_tables,
                &mut this.bitmaps,
                region_info,
                text_region_segment_flags,
                number_of_symbol_instances,
                referred_to,
                data,
                start,
                end,
                segment_number,
            )
        })
    }

    #[allow(clippy::too_many_arguments)]
    /// Decode and store an intermediate halftone region.
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
        self.time_call("intermediate_halftone_region", |this| {
            super::halftone_handler::on_intermediate_halftone_region(
                &this.patterns,
                &mut this.bitmaps,
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
        })
    }

    /// Finalize the current page and store it in the page list.
    pub fn finalize_current_page(&mut self) {
        self.time_call("finalize_current_page", |this| {
            super::page_handler::finalize_current_page(
                &mut this.current_page_info,
                &mut this.current_bitmap,
                this.current_y,
                &mut this.pages,
            );
        });
    }
}
