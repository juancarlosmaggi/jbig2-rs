use super::{IntermediateResources, PageComposeTarget, SegmentSlice};
use crate::bitmap::Bitmap;
use crate::common::error::Jbig2Error;
use crate::common::options::{DecodeLimits, check_cancelled, packed_bitmap_len};
use crate::common::profile::DecodeProfile;
use crate::decoders::halftone::ShiftedPattern;
use crate::document::{Jbig2Page, PageInfo};
use crate::huffman::HuffmanTable;
use crate::parser::segment::{
    GenericRegion, HalftoneRegionParams, PatternDictionaryParams, RegionInfo,
    SymbolDictionaryParams, TextRegionParams,
};
use std::collections::HashMap;
use std::sync::{Arc, atomic::AtomicBool};
use std::time::{Duration, Instant};

/// Visitor that accumulates decoded segments into pages.
#[derive(Default)]
pub struct SimpleSegmentVisitor {
    pub pages: Vec<Jbig2Page>,
    pub current_page_info: Option<PageInfo>,
    pub current_bitmap: Option<Bitmap>,
    pub current_y: usize,
    pub symbols: HashMap<u32, Vec<Bitmap>>,
    pub patterns: HashMap<u32, Vec<Bitmap>>,
    pub(crate) pattern_shifts: HashMap<u32, Arc<Vec<ShiftedPattern>>>,
    pub custom_tables: HashMap<u32, HuffmanTable>,
    pub bitmaps: HashMap<u32, Bitmap>,
    profile: Option<DecodeProfile>,
    limits: DecodeLimits,
    cancel_flag: Option<Arc<AtomicBool>>,
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

    pub fn new_with_options(limits: DecodeLimits, cancel_flag: Option<Arc<AtomicBool>>) -> Self {
        Self {
            limits,
            cancel_flag,
            ..Default::default()
        }
    }

    pub fn new_with_profile_and_options(
        limits: DecodeLimits,
        cancel_flag: Option<Arc<AtomicBool>>,
    ) -> Self {
        Self {
            profile: Some(DecodeProfile::default()),
            limits,
            cancel_flag,
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

    fn check_cancelled(&self) -> Result<(), Jbig2Error> {
        check_cancelled(&self.cancel_flag)
    }

    fn check_pages_after_new_page(&self) -> Result<(), Jbig2Error> {
        let active = usize::from(self.current_page_info.is_some());
        self.limits
            .check_page_count(self.pages.len().saturating_add(active).saturating_add(1))
    }

    fn check_pages_after_finalize(&self) -> Result<(), Jbig2Error> {
        if self.current_page_info.is_some() {
            self.limits
                .check_page_count(self.pages.len().saturating_add(1))
        } else {
            self.limits.check_page_count(self.pages.len())
        }
    }

    fn check_bitmap_budget(
        &self,
        width: usize,
        height: usize,
        resource: &'static str,
    ) -> Result<(), Jbig2Error> {
        self.limits.check_pixels(width, height, resource)?;
        self.limits.check_bitmap_bytes(width, height, resource)
    }

    fn check_region_budget(
        &self,
        region_info: &RegionInfo,
        resource: &'static str,
    ) -> Result<(), Jbig2Error> {
        self.check_bitmap_budget(
            region_info.width as usize,
            region_info.height as usize,
            resource,
        )
    }

    fn retained_symbol_bytes(&self) -> usize {
        self.symbols
            .values()
            .flat_map(|symbols| symbols.iter())
            .map(|bitmap| bitmap.data.len())
            .sum()
    }

    fn retained_intermediate_bitmap_bytes(&self) -> usize {
        self.bitmaps.values().map(|bitmap| bitmap.data.len()).sum()
    }

    fn check_retained_budgets(&self) -> Result<(), Jbig2Error> {
        self.limits
            .check_symbol_dictionary_bytes(self.retained_symbol_bytes())?;
        if let Some(limit) = self.limits.max_intermediate_bitmap_bytes {
            let actual = self.retained_intermediate_bitmap_bytes();
            if actual > limit {
                return Err(Jbig2Error::resource_limit_exceeded(
                    "retained intermediate bitmap bytes",
                    limit,
                    actual,
                ));
            }
        }
        Ok(())
    }

    /// Apply page information to the current decode state.
    pub fn on_page_information(&mut self, info: PageInfo) -> Result<(), Jbig2Error> {
        self.check_cancelled()?;
        self.check_pages_after_new_page()?;
        self.check_bitmap_budget(
            info.width as usize,
            info.height as usize,
            "page decoded pixels",
        )?;
        self.time_call("page_information", |this| {
            super::page_handler::on_page_information(
                &mut this.current_page_info,
                &mut this.current_bitmap,
                &mut this.current_y,
                &mut this.pages,
                info,
            );
        });
        Ok(())
    }

    /// Advance the current stripe offset.
    pub fn on_end_of_stripe(&mut self, end_row: usize) -> Result<(), Jbig2Error> {
        self.check_cancelled()?;
        if let Some(page_info) = &self.current_page_info
            && page_info.height_unknown
        {
            let next_row = end_row.saturating_add(1);
            let stripe = page_info.stripe_size as usize;
            let mut next_height = next_row.max(1);
            if stripe > 0 {
                next_height = next_height.saturating_add(stripe);
            }
            self.check_bitmap_budget(page_info.width as usize, next_height, "striped page bitmap")?;
        }
        self.time_call("end_of_stripe", |this| {
            super::page_handler::on_end_of_stripe(
                &mut this.current_page_info,
                &mut this.current_bitmap,
                &mut this.current_y,
                end_row,
            );
        });
        Ok(())
    }

    /// Composite a bitmap onto the current page.
    pub fn draw_bitmap(
        &mut self,
        region_info: &RegionInfo,
        src_bitmap: &Bitmap,
    ) -> Result<(), Jbig2Error> {
        self.check_cancelled()?;
        self.time_call("draw_bitmap", |this| {
            super::region_handlers::draw_bitmap(
                PageComposeTarget {
                    page_info: &mut this.current_page_info,
                    bitmap: &mut this.current_bitmap,
                },
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
        self.check_cancelled()?;
        self.check_region_budget(&region.info, "immediate generic region")?;
        self.time_call("immediate_generic_region", |this| {
            super::region_handlers::on_immediate_generic_region(
                PageComposeTarget {
                    page_info: &mut this.current_page_info,
                    bitmap: &mut this.current_bitmap,
                },
                region,
                SegmentSlice { data, start, end },
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
        self.check_cancelled()?;
        self.check_region_budget(region_info, "immediate refinement region")?;
        self.time_call("immediate_generic_refinement_region", |this| {
            super::region_handlers::on_immediate_generic_refinement_region(
                PageComposeTarget {
                    page_info: &mut this.current_page_info,
                    bitmap: &mut this.current_bitmap,
                },
                &this.bitmaps,
                region_info,
                referred_to,
                SegmentSlice { data, start, end },
            )
        })
    }

    /// Decode and store a symbol dictionary segment.
    pub fn on_symbol_dictionary(
        &mut self,
        params: &SymbolDictionaryParams,
    ) -> Result<(), Jbig2Error> {
        self.check_cancelled()?;
        let result = self.time_call("symbol_dictionary", |this| {
            super::symbol_handler::on_symbol_dictionary(
                &mut this.symbols,
                &this.custom_tables,
                params,
            )
        });
        result?;
        self.check_retained_budgets()
    }

    /// Decode and draw an immediate text region.
    pub fn on_immediate_text_region(
        &mut self,
        params: &TextRegionParams,
        referred_to: &[u32],
        data: &[u8],
        start: usize,
        end: usize,
    ) -> Result<(), Jbig2Error> {
        self.check_cancelled()?;
        self.check_region_budget(&params.region_info, "immediate text region")?;
        self.time_call("immediate_text_region", |this| {
            super::text_handler::on_immediate_text_region(
                PageComposeTarget {
                    page_info: &mut this.current_page_info,
                    bitmap: &mut this.current_bitmap,
                },
                &this.symbols,
                &this.custom_tables,
                params,
                referred_to,
                SegmentSlice { data, start, end },
            )
        })
    }

    /// Decode and store a pattern dictionary segment.
    pub fn on_pattern_dictionary(
        &mut self,
        params: &PatternDictionaryParams,
        current_segment: u32,
        data: &[u8],
        start: usize,
        end: usize,
    ) -> Result<(), Jbig2Error> {
        self.check_cancelled()?;
        let pattern_count = params.max_pattern_index.saturating_add(1);
        let pattern_bytes = packed_bitmap_len(params.pattern_width, params.pattern_height)
            .and_then(|bytes| bytes.checked_mul(pattern_count))
            .ok_or_else(|| {
                Jbig2Error::resource_limit_exceeded(
                    "pattern dictionary bytes",
                    usize::MAX,
                    usize::MAX,
                )
            })?;
        if let Some(limit) = self.limits.max_intermediate_bitmap_bytes
            && pattern_bytes > limit
        {
            return Err(Jbig2Error::resource_limit_exceeded(
                "pattern dictionary bytes",
                limit,
                pattern_bytes,
            ));
        }
        self.time_call("pattern_dictionary", |this| {
            super::pattern_handler::on_pattern_dictionary(
                &mut this.patterns,
                &mut this.pattern_shifts,
                params,
                current_segment,
                SegmentSlice { data, start, end },
            )
        })?;
        self.check_retained_budgets()
    }

    /// Decode and draw an immediate halftone region.
    pub fn on_immediate_halftone_region(
        &mut self,
        params: &HalftoneRegionParams,
        referred_to: &[u32],
        data: &[u8],
        start: usize,
        end: usize,
    ) -> Result<(), Jbig2Error> {
        self.check_cancelled()?;
        self.check_region_budget(&params.region_info, "immediate halftone region")?;
        self.check_bitmap_budget(params.grid_width, params.grid_height, "halftone grid")?;
        self.time_call("immediate_halftone_region", |this| {
            super::halftone_handler::on_immediate_halftone_region(
                PageComposeTarget {
                    page_info: &mut this.current_page_info,
                    bitmap: &mut this.current_bitmap,
                },
                &this.patterns,
                &this.pattern_shifts,
                params,
                referred_to,
                SegmentSlice { data, start, end },
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
        self.check_cancelled()?;
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
        self.check_cancelled()?;
        self.check_region_budget(&region.info, "intermediate generic region")?;
        self.time_call("intermediate_generic_region", |this| {
            super::region_handlers::on_intermediate_generic_region(
                IntermediateResources {
                    symbols: &this.symbols,
                    patterns: &this.patterns,
                    custom_tables: &this.custom_tables,
                    bitmaps: &mut this.bitmaps,
                },
                region,
                referred_to,
                SegmentSlice { data, start, end },
                segment_number,
            )
        })?;
        self.check_retained_budgets()
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
        self.check_cancelled()?;
        self.check_region_budget(region_info, "intermediate refinement region")?;
        self.time_call("intermediate_generic_refinement_region", |this| {
            super::region_handlers::on_intermediate_generic_refinement_region(
                &mut this.bitmaps,
                region_info,
                referred_to,
                SegmentSlice { data, start, end },
                segment_number,
            )
        })?;
        self.check_retained_budgets()
    }

    /// Decode an intermediate text region (no page compositing).
    pub fn on_intermediate_text_region(
        &mut self,
        params: &TextRegionParams,
        referred_to: &[u32],
        data: &[u8],
        start: usize,
        end: usize,
        segment_number: u32,
    ) -> Result<(), Jbig2Error> {
        self.check_cancelled()?;
        self.check_region_budget(&params.region_info, "intermediate text region")?;
        self.time_call("intermediate_text_region", |this| {
            super::text_handler::on_intermediate_text_region(
                IntermediateResources {
                    symbols: &this.symbols,
                    patterns: &this.patterns,
                    custom_tables: &this.custom_tables,
                    bitmaps: &mut this.bitmaps,
                },
                params,
                referred_to,
                SegmentSlice { data, start, end },
                segment_number,
            )
        })?;
        self.check_retained_budgets()
    }

    /// Decode and store an intermediate halftone region.
    pub fn on_intermediate_halftone_region(
        &mut self,
        params: &HalftoneRegionParams,
        referred_to: &[u32],
        data: &[u8],
        start: usize,
        end: usize,
        segment_number: u32,
    ) -> Result<(), Jbig2Error> {
        self.check_cancelled()?;
        self.check_region_budget(&params.region_info, "intermediate halftone region")?;
        self.check_bitmap_budget(params.grid_width, params.grid_height, "halftone grid")?;
        self.time_call("intermediate_halftone_region", |this| {
            super::halftone_handler::on_intermediate_halftone_region(
                &this.patterns,
                &this.pattern_shifts,
                &mut this.bitmaps,
                params,
                referred_to,
                SegmentSlice { data, start, end },
                segment_number,
            )
        })?;
        self.check_retained_budgets()
    }

    /// Finalize the current page and store it in the page list.
    pub fn finalize_current_page(&mut self) -> Result<(), Jbig2Error> {
        self.check_cancelled()?;
        self.check_pages_after_finalize()?;
        if let Some(page_info) = &self.current_page_info {
            self.check_bitmap_budget(
                page_info.width as usize,
                page_info.height as usize,
                "final page bitmap",
            )?;
        }
        self.time_call("finalize_current_page", |this| {
            super::page_handler::finalize_current_page(
                &mut this.current_page_info,
                &mut this.current_bitmap,
                this.current_y,
                &mut this.pages,
            );
        });
        Ok(())
    }
}
