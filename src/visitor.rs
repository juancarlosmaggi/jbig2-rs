use crate::bitmap::Bitmap;
use crate::contexts::DecodingContext;
use crate::decode_generic::{decode_bitmap, DecodeBitmapParams};
use crate::decode_halftone::decode_halftone_region;
use crate::decode_pattern::decode_pattern_dictionary;
use crate::decode_symbol::decode_symbol_dictionary;
use crate::decode_text::decode_text_region;
use crate::error::Jbig2Error;
use crate::huffman::{decode_tables_segment, HuffmanTable};
use crate::segment::{GenericRegion, PageInfo, RegionInfo, SymbolDictionaryParams};
use std::collections::HashMap;

#[derive(Default)]
pub struct SimpleSegmentVisitor {
    pub current_page_info: Option<PageInfo>,
    pub bitmap: Option<Bitmap>,
    pub symbols: HashMap<u32, Vec<Bitmap>>,
    pub patterns: HashMap<u32, Vec<Bitmap>>,
    pub custom_tables: HashMap<u32, HuffmanTable>,
    pub referred_to_symbols: HashMap<u32, Vec<Bitmap>>, // For temporary storage
}

impl SimpleSegmentVisitor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_page_information(&mut self, info: PageInfo) {
        self.current_page_info = Some(info.clone());
        let width = info.width as usize;
        let height = info.height as usize;
        let mut bitmap = Bitmap::new(width, height);
        if info.default_pixel_value != 0 {
            for y in 0..height {
                for x in 0..width {
                    bitmap.set_pixel(x, y, 1);
                }
            }
        }
        self.bitmap = Some(bitmap);
    }

    pub fn draw_bitmap(&mut self, region_info: &RegionInfo, src_bitmap: &Bitmap) {
        let page_info = self.current_page_info.as_ref().unwrap();
        let page_width = page_info.width as usize;
        let page_height = page_info.height as usize;
        let combo_op = if page_info.combination_operator_override {
            region_info.combination_operator
        } else {
            page_info.combination_operator
        };
        let dst = self.bitmap.as_mut().unwrap();
        let reg_x = region_info.x as usize;
        let reg_y = region_info.y as usize;
        let width = region_info.width.min(page_width as u32 - reg_x as u32) as usize;
        let height = region_info.height.min(page_height as u32 - reg_y as u32) as usize;
        for i in 0..height {
            for j in 0..width {
                let src = src_bitmap.get_pixel(j, i);
                let dx = reg_x + j;
                let dy = reg_y + i;
                let old_dst = dst.get_pixel(dx, dy);
                let new_val = match combo_op {
                    0 => src, // replace
                    1 => old_dst | src, // OR
                    2 => old_dst & src, // AND
                    3 => old_dst ^ src, // XOR
                    4 => !(old_dst ^ src) & 1, // XNOR (bi-level)
                    _ => old_dst, // undefined: no-op
                };
                dst.set_pixel(dx, dy, new_val);
            }
        }
    }

    pub fn on_immediate_generic_region(&mut self, region: &GenericRegion, data: &[u8], start: usize, end: usize) -> Result<(), Jbig2Error> {
        let region_info = &region.info;
        let at_bytes = region.at.len() * 2;
        let decoding_start = start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 1 + at_bytes;
        if decoding_start >= end {
            return Err(Jbig2Error::new("insufficient data for generic region"));
        }
        let slice = &data[decoding_start..end];
        let mut decoding_context = DecodingContext::new(slice.to_vec(), 0, slice.len());
        let params = DecodeBitmapParams {
            mmr: region.mmr,
            width: region_info.width as usize,
            height: region_info.height as usize,
            template_index: region.template,
            prediction: region.prediction,
            skip: None,
            at: region.at.clone(),
        };
        let bitmap = decode_bitmap(&params, &mut decoding_context)?;
        self.draw_bitmap(region_info, &bitmap);
        Ok(())
    }

    pub fn on_symbol_dictionary(&mut self, params: &SymbolDictionaryParams) -> Result<(), Jbig2Error> {
        let huffman = (params.dictionary_flags & 1) != 0;
        let refinement = (params.dictionary_flags & 2) != 0;
        let template = ((params.dictionary_flags >> 10) & 3) as usize;
        let refinement_template = ((params.dictionary_flags >> 12) & 1) as usize;
        // Parse AT parameters if not Huffman
        let mut at = Vec::new();
        let mut refinement_at = Vec::new();
        let mut pos = params.start;
        if huffman {
            pos = params.start + 4;
        }
        if !huffman {
            let at_length = if template == 0 { 4 } else { 1 };
            for _ in 0..at_length {
                let x = params.data[pos] as i8;
                let y = params.data[pos + 1] as i8;
                at.push((x, y));
                pos += 2;
            }
        }
        if refinement && refinement_template == 0 {
            for _ in 0..2 {
                let x = params.data[pos] as i8;
                let y = params.data[pos + 1] as i8;
                refinement_at.push((x, y));
                pos += 2;
            }
        }
        // Collect input symbols from referred segments
        let mut input_symbols = Vec::new();
        for &segment_id in params.referred_segments {
            if let Some(symbols) = self.symbols.get(&segment_id) {
                input_symbols.extend(symbols.clone());
            }
        }
        let slice = &params.data[pos..params.end];
        let mut decoding_context = DecodingContext::new(slice.to_vec(), 0, slice.len());
        let symbol_params = crate::decode_symbol::SymbolDictionaryParams {
            huffman,
            refinement,
            symbols: input_symbols,
            number_of_new_symbols: params.number_of_new_symbols as usize,
            number_of_exported_symbols: params.number_of_exported_symbols as usize,
            template_index: template,
            at,
            refinement_template_index: refinement_template,
            refinement_at,
        };
        let exported_symbols = decode_symbol_dictionary(&symbol_params, &mut decoding_context)?;
        self.symbols.insert(params.current_segment, exported_symbols);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn on_immediate_text_region(
        &mut self,
        region_info: &RegionInfo,
        text_region_segment_flags: u16,
        number_of_symbol_instances: u32,
        referred_segments: &[u32],
        data: &[u8],
        start: usize,
        end: usize,
    ) -> Result<(), Jbig2Error> {
        let huffman = (text_region_segment_flags & 1) != 0;
        let refinement = (text_region_segment_flags & 2) != 0;
        let log_strip_size = ((text_region_segment_flags >> 2) & 3) as usize;
        let strip_size = 1 << log_strip_size;
        let reference_corner = ((text_region_segment_flags >> 4) & 3) as usize;
        let transposed = (text_region_segment_flags & 64) != 0;
        let combination_operator = ((text_region_segment_flags >> 7) & 3) as usize;
        let default_pixel_value = ((text_region_segment_flags >> 9) & 1) as u8;
        let ds_offset = ((text_region_segment_flags as i32) << 17) >> 27;
        let refinement_template = ((text_region_segment_flags >> 15) & 1) as usize;
        // Collect input symbols from referred segments
        let mut input_symbols = Vec::new();
        for &segment_id in referred_segments {
            if let Some(symbols) = self.symbols.get(&segment_id) {
                input_symbols.extend(symbols.clone());
            }
        }
        let symbol_code_length = crate::core_utils::log2(input_symbols.len() as u32);
        // Parse refinement AT if needed
        let mut refinement_at = Vec::new();
        let mut pos = start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 2; // flags are 2 bytes
        if huffman {
            // Skip Huffman flags for now
            pos += 2;
        }
        if refinement && refinement_template == 0 {
            for _ in 0..2 {
                let x = data[pos] as i8;
                let y = data[pos + 1] as i8;
                refinement_at.push((x, y));
                pos += 2;
            }
        }
        let slice = &data[pos..end];
        let mut decoding_context = DecodingContext::new(slice.to_vec(), 0, slice.len());
        let params = crate::decode_text::TextRegionParams {
            huffman,
            refinement,
            width: region_info.width as usize,
            height: region_info.height as usize,
            default_pixel_value,
            number_of_symbol_instances: number_of_symbol_instances as usize,
            strip_size,
            input_symbols,
            symbol_code_length: symbol_code_length as usize,
            transposed,
            ds_offset,
            reference_corner,
            combination_operator,
            log_strip_size,
        };
        let bitmap = decode_text_region(&params, &mut decoding_context)?;
        self.draw_bitmap(region_info, &bitmap);
        Ok(())
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
        let slice = &data[start..end];
        let mut decoding_context = DecodingContext::new(slice.to_vec(), 0, slice.len());
        let params = crate::decode_pattern::PatternDictionaryParams {
            mmr,
            pattern_width,
            pattern_height,
            max_pattern_index,
            template,
        };
        let patterns = decode_pattern_dictionary(&params, &mut decoding_context)?;
        self.patterns.insert(current_segment, patterns);
        Ok(())
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
        referred_segments: &[u32],
        data: &[u8],
        start: usize,
        end: usize,
    ) -> Result<(), Jbig2Error> {
        // Get patterns from referred segment
        let patterns = if let Some(patterns) = self.patterns.get(&referred_segments[0]) {
            patterns
        } else {
            return Err(Jbig2Error::new("pattern dictionary not found"));
        };
        let slice = &data[start..end];
        let mut decoding_context = DecodingContext::new(slice.to_vec(), 0, slice.len());
        let params = crate::decode_halftone::HalftoneRegionParams {
            mmr,
            patterns: patterns.clone(),
            template,
            region_width: region_info.width as usize,
            region_height: region_info.height as usize,
            default_pixel_value,
            enable_skip,
            combination_operator,
            grid_width,
            grid_height,
            grid_offset_x,
            grid_offset_y,
            grid_vector_x,
            grid_vector_y,
        };
        let bitmap = decode_halftone_region(&params, &mut decoding_context)?;
        self.draw_bitmap(region_info, &bitmap);
        Ok(())
    }

    pub fn on_tables(&mut self, segment_number: u32, data: &[u8], start: usize, end: usize) -> Result<(), Jbig2Error> {
        let table = decode_tables_segment(data, start, end)?;
        self.custom_tables.insert(segment_number, table);
        Ok(())
    }
}

const REGION_SEGMENT_INFORMATION_FIELD_LENGTH: usize = 17;