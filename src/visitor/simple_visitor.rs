use crate::bitmap::Bitmap;
use crate::bitmap_utils;
use crate::contexts::DecodingContext;
use crate::decode::decode_generic::{DecodeBitmapParams, decode_bitmap};
use crate::decode::decode_halftone::decode_halftone_region;
use crate::decode::decode_pattern::decode_pattern_dictionary;
use crate::decode::decode_symbol::decode_symbol_dictionary;
use crate::decode::decode_text::decode_text_region;
use crate::error::Jbig2Error;
use crate::huffman::{HuffmanTable, TextRegionHuffmanParams, decode_tables_segment};
use crate::reader::Reader;
use crate::segment::{
    GenericRegion, PageInfo, RegionInfo, SymbolDictionaryParams, parse_at_parameters, read_u16,
};
use std::collections::HashMap;

fn bitmap_to_bit_packed(bitmap: &Bitmap) -> Vec<u8> {
    let width = bitmap.width;
    let height = bitmap.height;
    let row_size = width.div_ceil(8); // bytes per row
    let mut packed = vec![0u8; row_size * height];

    for y in 0..height {
        for x in 0..width {
            let pixel = bitmap.get_pixel(x, y);
            if pixel != 0 {
                let byte_index = y * row_size + (x / 8);
                let bit_index = 7 - (x % 8); // MSB first
                packed[byte_index] |= 1 << bit_index;
            }
        }
    }

    packed
}

#[derive(Clone)]
pub struct Jbig2Page {
    pub page_info: PageInfo,
    pub bitmap: Bitmap,
    pub bit_packed_data: Vec<u8>,
}

impl Jbig2Page {
    pub fn to_image_data(&self) -> Vec<u8> {
        let width = self.page_info.width as usize;
        let height = self.page_info.height as usize;
        let mut img_data = vec![0u8; width * height];
        let row_size = width.div_ceil(8);
        for y in 0..height {
            for x in 0..width {
                let byte_index = y * row_size + (x / 8);
                let bit_index = 7 - (x % 8);
                let pixel = if (self.bit_packed_data[byte_index] & (1 << bit_index)) != 0 {
                    255
                } else {
                    0
                };
                img_data[y * width + x] = pixel;
            }
        }
        img_data
    }
}

#[derive(Default)]
pub struct SimpleSegmentVisitor {
    pub pages: Vec<Jbig2Page>,
    pub current_page_info: Option<PageInfo>,
    pub current_bitmap: Option<Bitmap>,
    pub symbols: HashMap<u32, Vec<Bitmap>>,
    pub patterns: HashMap<u32, Vec<Bitmap>>,
    pub custom_tables: HashMap<u32, HuffmanTable>,
    pub referred_to_symbols: HashMap<u32, Vec<Bitmap>>, // For temporary storage
}

impl SimpleSegmentVisitor {
    pub fn new() -> Self {
        Self::default()
    }

    fn collect_input_symbols(&self, referred_segments: &[u32]) -> Vec<Bitmap> {
        let mut input_symbols = Vec::new();
        for &segment_id in referred_segments {
            if let Some(symbols) = self.symbols.get(&segment_id) {
                input_symbols.extend(symbols.clone());
            }
        }
        input_symbols
    }

    pub fn on_page_information(&mut self, info: PageInfo) {
        // If we have a previous page, finalize it
        if let (Some(page_info), Some(bitmap)) =
            (self.current_page_info.take(), self.current_bitmap.take())
        {
            let bit_packed_data = bitmap_to_bit_packed(&bitmap);
            self.pages.push(Jbig2Page {
                page_info,
                bitmap,
                bit_packed_data,
            });
        }

        self.current_page_info = Some(info.clone());
        let width = info.width as usize;
        let height = info.height as usize;
        let bitmap =
            bitmap_utils::create_initialized_bitmap(width, height, info.default_pixel_value);
        self.current_bitmap = Some(bitmap);
    }

    pub fn draw_bitmap(
        &mut self,
        region_info: &RegionInfo,
        src_bitmap: &Bitmap,
    ) -> Result<(), Jbig2Error> {
        let page_info = self
            .current_page_info
            .as_ref()
            .ok_or(Jbig2Error::new("no current page info"))?;
        let page_width = page_info.width as usize;
        let page_height = page_info.height as usize;
        let combo_op = if page_info.combination_operator_override {
            region_info.combination_operator
        } else {
            page_info.combination_operator
        };
        let dst = self
            .current_bitmap
            .as_mut()
            .ok_or(Jbig2Error::new("no current bitmap"))?;

        // Region coordinates are validated by checking bounds below

        let reg_x = region_info.x as usize;
        let reg_y = region_info.y as usize;

        // Check if region is completely outside page bounds
        if reg_x >= page_width || reg_y >= page_height {
            return Ok(()); // Nothing to draw
        }

        let width = (region_info.width as usize).min(page_width - reg_x);
        let height = (region_info.height as usize).min(page_height - reg_y);

        // Validate source bitmap dimensions
        if src_bitmap.width < width || src_bitmap.height < height {
            return Err(Jbig2Error::new("source bitmap too small for region"));
        }

        for i in 0..height {
            for j in 0..width {
                let src = src_bitmap.get_pixel(j, i);
                let dx = reg_x + j;
                let dy = reg_y + i;
                let old_dst = dst.get_pixel(dx, dy);
                let new_val = bitmap_utils::apply_page_combination_operator(old_dst, src, combo_op);
                dst.set_pixel(dx, dy, new_val);
            }
        }
        Ok(())
    }

    pub fn on_immediate_generic_region(
        &mut self,
        region: &GenericRegion,
        data: &[u8],
        start: usize,
        end: usize,
    ) -> Result<(), Jbig2Error> {
        let region_info = &region.info;
        let at_bytes = region.at.len() * 2;
        let decoding_start = start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + at_bytes;
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
        self.draw_bitmap(region_info, &bitmap)?;
        Ok(())
    }

    pub fn on_immediate_generic_refinement_region(
        &mut self,
        region_info: &RegionInfo,
        data: &[u8],
        start: usize,
        end: usize,
    ) -> Result<(), Jbig2Error> {
        // Parse refinement region parameters
        let mut pos = start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH;
        let generic_region_segment_flags = data[pos];
        pos += 1;
        let template = ((generic_region_segment_flags >> 1) & 3) as usize;
        let at_length = if template == 0 { 2 } else { 0 };
        let at = if at_length > 0 {
            parse_at_parameters(data, pos, at_length)?
        } else {
            Vec::new()
        };
        let pos = if at_length > 0 {
            pos + at_length * 2
        } else {
            pos
        };
        // Get reference bitmap from referred segment
        let reference_bitmap = if let Some(ref_bitmap) = self.current_bitmap.as_ref() {
            ref_bitmap
        } else {
            return Err(Jbig2Error::new("no reference bitmap for refinement region"));
        };
        let slice = &data[pos..end];
        let mut decoding_context = DecodingContext::new(slice.to_vec(), 0, slice.len());
        let bitmap = crate::decode::decode_refinement::decode_refinement(
            &crate::decode::decode_refinement::RefinementParams {
                width: region_info.width as usize,
                height: region_info.height as usize,
                template_index: template,
                reference_bitmap,
                offset_x: 0, // Default offset
                offset_y: 0,
                prediction: false,
                at,
            },
            &mut decoding_context,
        )?;
        self.draw_bitmap(region_info, &bitmap)?;
        Ok(())
    }

    pub fn on_symbol_dictionary(
        &mut self,
        params: &SymbolDictionaryParams,
    ) -> Result<(), Jbig2Error> {
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
            at = parse_at_parameters(params.data, pos, at_length)?;
            pos += at_length * 2;
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
        let input_symbols = self.collect_input_symbols(params.referred_segments);
        let slice = &params.data[pos..params.end];
        let mut decoding_context = DecodingContext::new(slice.to_vec(), 0, slice.len());

        // Get Huffman tables if needed
        let huffman_tables = if huffman {
            Some(crate::huffman::get_symbol_dictionary_huffman_tables(
                ((params.dictionary_flags >> 2) & 3) as u8, // huffmanDHSelector
                ((params.dictionary_flags >> 4) & 3) as u8, // huffmanDWSelector
                ((params.dictionary_flags >> 6) & 1) != 0,  // bitmapSizeSelector
                ((params.dictionary_flags >> 7) & 1) != 0,  // aggregationInstancesSelector
                params.referred_segments,
                &self.custom_tables,
            )?)
        } else {
            None
        };

        let symbol_params = crate::decode::decode_symbol::SymbolDictionaryParams {
            huffman,
            refinement,
            symbols: input_symbols,
            number_of_new_symbols: params.number_of_new_symbols as usize,
            number_of_exported_symbols: params.number_of_exported_symbols as usize,
            template_index: template,
            at,
            refinement_template_index: refinement_template,
            refinement_at,
            huffman_tables,
        };

        let mut huffman_input = if huffman {
            Some(Reader::new(slice.to_vec(), 0, slice.len()))
        } else {
            None
        };

        let exported_symbols = decode_symbol_dictionary(
            &symbol_params,
            &mut decoding_context,
            huffman_input.as_mut(),
        )?;
        self.symbols
            .insert(params.current_segment, exported_symbols);
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
        let input_symbols = self.collect_input_symbols(referred_segments);
        let symbol_code_length = crate::core_utils::log2(input_symbols.len() as u32);
        // Parse Huffman flags and refinement AT
        let mut refinement_at = Vec::new();
        let mut pos = start + REGION_SEGMENT_INFORMATION_FIELD_LENGTH + 2; // flags are 2 bytes
        let mut huffman_fs = 0u8;
        let mut huffman_ds = 0u8;
        let mut huffman_dt = 0u8;
        let mut huffman_refinement_dw = 0u8;
        let mut huffman_refinement_dh = 0u8;
        let mut huffman_refinement_dx = 0u8;
        let mut huffman_refinement_dy = 0u8;
        let mut huffman_refinement_size_selector = false;
        if huffman {
            let huffman_flags = read_u16(data, pos);
            pos += 2;
            huffman_fs = (huffman_flags & 3) as u8;
            huffman_ds = ((huffman_flags >> 2) & 3) as u8;
            huffman_dt = ((huffman_flags >> 4) & 3) as u8;
            huffman_refinement_dw = ((huffman_flags >> 6) & 3) as u8;
            huffman_refinement_dh = ((huffman_flags >> 8) & 3) as u8;
            huffman_refinement_dx = ((huffman_flags >> 10) & 3) as u8;
            huffman_refinement_dy = ((huffman_flags >> 12) & 3) as u8;
            huffman_refinement_size_selector = (huffman_flags & 0x4000) != 0;
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

        // Get Huffman tables if needed
        let mut huffman_reader = if huffman {
            Some(Reader::new(slice.to_vec(), 0, slice.len()))
        } else {
            None
        };

        let huffman_tables = if let Some(ref mut reader) = huffman_reader {
            let params = TextRegionHuffmanParams {
                huffman_fs,
                huffman_ds,
                huffman_dt,
                huffman_refinement_dw,
                huffman_refinement_dh,
                huffman_refinement_dx,
                huffman_refinement_dy,
                huffman_refinement_size_selector,
            };
            Some(crate::huffman::get_text_region_huffman_tables(
                &params,
                referred_segments,
                &self.custom_tables,
                input_symbols.len(),
                reader,
            )?)
        } else {
            None
        };

        let params = crate::decode::decode_text::TextRegionParams {
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
            huffman_tables,
            refinement_template_index: refinement_template,
            refinement_at,
        };

        let bitmap = decode_text_region(&params, &mut decoding_context, huffman_reader.as_mut())?;
        self.draw_bitmap(region_info, &bitmap)?;
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
        let params = crate::decode::decode_pattern::PatternDictionaryParams {
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
        let params = crate::decode::decode_halftone::HalftoneRegionParams {
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
        self.draw_bitmap(region_info, &bitmap)?;
        Ok(())
    }

    pub fn on_tables(
        &mut self,
        segment_number: u32,
        data: &[u8],
        start: usize,
        end: usize,
    ) -> Result<(), Jbig2Error> {
        let table = decode_tables_segment(data, start, end)?;
        self.custom_tables.insert(segment_number, table);
        Ok(())
    }

    pub fn on_intermediate_generic_region(
        &mut self,
        region: &GenericRegion,
        referred_segments: &[u32],
        data: &[u8],
        start: usize,
        end: usize,
    ) -> Result<(), Jbig2Error> {
        // Basic validation: check that referred segments exist
        for &segment_id in referred_segments {
            if !self.symbols.contains_key(&segment_id)
                && !self.patterns.contains_key(&segment_id)
                && !self.custom_tables.contains_key(&segment_id)
            {
                return Err(Jbig2Error::new("referred segment not found"));
            }
        }
        // Intermediate generic regions are handled the same as immediate in the spec
        self.on_immediate_generic_region(region, data, start, end)
    }

    pub fn on_intermediate_generic_refinement_region(
        &mut self,
        region_info: &RegionInfo,
        referred_segments: &[u32],
        data: &[u8],
        start: usize,
        end: usize,
    ) -> Result<(), Jbig2Error> {
        // Basic validation: check that referred segments exist
        for &segment_id in referred_segments {
            if !self.symbols.contains_key(&segment_id)
                && !self.patterns.contains_key(&segment_id)
                && !self.custom_tables.contains_key(&segment_id)
            {
                return Err(Jbig2Error::new("referred segment not found"));
            }
        }
        // Intermediate refinement regions are handled the same as immediate in the spec
        self.on_immediate_generic_refinement_region(region_info, data, start, end)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn on_intermediate_text_region(
        &mut self,
        region_info: &RegionInfo,
        text_region_segment_flags: u16,
        number_of_symbol_instances: u32,
        referred_segments: &[u32],
        data: &[u8],
        start: usize,
        end: usize,
    ) -> Result<(), Jbig2Error> {
        // Basic validation: check that referred segments exist
        for &segment_id in referred_segments {
            if !self.symbols.contains_key(&segment_id)
                && !self.patterns.contains_key(&segment_id)
                && !self.custom_tables.contains_key(&segment_id)
            {
                return Err(Jbig2Error::new("referred segment not found"));
            }
        }
        // Intermediate text regions are handled the same as immediate in the spec
        self.on_immediate_text_region(
            region_info,
            text_region_segment_flags,
            number_of_symbol_instances,
            referred_segments,
            data,
            start,
            end,
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
        referred_segments: &[u32],
        data: &[u8],
        start: usize,
        end: usize,
    ) -> Result<(), Jbig2Error> {
        // Basic validation: check that referred segments exist
        for &segment_id in referred_segments {
            if !self.symbols.contains_key(&segment_id)
                && !self.patterns.contains_key(&segment_id)
                && !self.custom_tables.contains_key(&segment_id)
            {
                return Err(Jbig2Error::new("referred segment not found"));
            }
        }
        // Intermediate halftone regions are handled the same as immediate in the spec
        self.on_immediate_halftone_region(
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
            referred_segments,
            data,
            start,
            end,
        )
    }

    // Finalize the current page and add it to the pages vector
    pub fn finalize_current_page(&mut self) {
        if let (Some(page_info), Some(bitmap)) =
            (self.current_page_info.take(), self.current_bitmap.take())
        {
            let bit_packed_data = bitmap_to_bit_packed(&bitmap);
            self.pages.push(Jbig2Page {
                page_info,
                bitmap,
                bit_packed_data,
            });
        }
    }
}

const REGION_SEGMENT_INFORMATION_FIELD_LENGTH: usize = 17;
