#[cfg(test)]
mod tests {
    use jbig2_rs::bitmap::Bitmap;
    use jbig2_rs::contexts::DecodingContext;
    use jbig2_rs::decode::decode_text::{decode_text_region, TextRegionParams};

    #[test]
    fn test_decode_text_region_no_symbols() {
        let data = vec![0u8; 100];
        let mut context = DecodingContext::new(data.clone(), 0, data.len());

        let params = TextRegionParams {
            huffman: false,
            refinement: false,
            width: 100,
            height: 100,
            default_pixel_value: 0,
            number_of_symbol_instances: 1,
            strip_size: 1,
            input_symbols: vec![], // Empty symbols
            symbol_code_length: 1,
            transposed: false,
            ds_offset: 0,
            reference_corner: 0,
            combination_operator: 0,
            log_strip_size: 0,
            huffman_tables: None,
            refinement_template_index: 0,
            refinement_at: vec![],
        };

        let result = decode_text_region(&params, &mut context, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_text_region_invalid_dimensions() {
        let data = vec![0u8; 100];
        let mut context = DecodingContext::new(data.clone(), 0, data.len());
        let symbol = Bitmap::new(8, 8);

        let params = TextRegionParams {
            huffman: false,
            refinement: false,
            width: 0, // Invalid
            height: 100,
            default_pixel_value: 0,
            number_of_symbol_instances: 1,
            strip_size: 1,
            input_symbols: vec![symbol],
            symbol_code_length: 1,
            transposed: false,
            ds_offset: 0,
            reference_corner: 0,
            combination_operator: 0,
            log_strip_size: 0,
            huffman_tables: None,
            refinement_template_index: 0,
            refinement_at: vec![],
        };

        let result = decode_text_region(&params, &mut context, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_text_region_invalid_reference_corner() {
        let data = vec![0u8; 100];
        let mut context = DecodingContext::new(data.clone(), 0, data.len());
        let symbol = Bitmap::new(8, 8);

        let params = TextRegionParams {
            huffman: false,
            refinement: false,
            width: 100,
            height: 100,
            default_pixel_value: 0,
            number_of_symbol_instances: 1,
            strip_size: 1,
            input_symbols: vec![symbol],
            symbol_code_length: 1,
            transposed: false,
            ds_offset: 0,
            reference_corner: 4, // Invalid (should be 0-3)
            combination_operator: 0,
            log_strip_size: 0,
            huffman_tables: None,
            refinement_template_index: 0,
            refinement_at: vec![],
        };

        let result = decode_text_region(&params, &mut context, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_text_region_invalid_combination_operator() {
        let data = vec![0u8; 100];
        let mut context = DecodingContext::new(data.clone(), 0, data.len());
        let symbol = Bitmap::new(8, 8);

        let params = TextRegionParams {
            huffman: false,
            refinement: false,
            width: 100,
            height: 100,
            default_pixel_value: 0,
            number_of_symbol_instances: 1,
            strip_size: 1,
            input_symbols: vec![symbol],
            symbol_code_length: 1,
            transposed: false,
            ds_offset: 0,
            reference_corner: 0,
            combination_operator: 13, // Invalid (should be 0-12)
            log_strip_size: 0,
            huffman_tables: None,
            refinement_template_index: 0,
            refinement_at: vec![],
        };

        let result = decode_text_region(&params, &mut context, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_text_region_refinement_with_huffman() {
        let data = vec![0u8; 100];
        let mut context = DecodingContext::new(data.clone(), 0, data.len());
        let symbol = Bitmap::new(8, 8);

        let params = TextRegionParams {
            huffman: true,
            refinement: true, // Invalid combination
            width: 100,
            height: 100,
            default_pixel_value: 0,
            number_of_symbol_instances: 1,
            strip_size: 1,
            input_symbols: vec![symbol],
            symbol_code_length: 1,
            transposed: false,
            ds_offset: 0,
            reference_corner: 0,
            combination_operator: 0,
            log_strip_size: 0,
            huffman_tables: None,
            refinement_template_index: 0,
            refinement_at: vec![],
        };

        let result = decode_text_region(&params, &mut context, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_text_region_with_huffman_no_tables() {
        let data = vec![0u8; 100];
        let mut context = DecodingContext::new(data.clone(), 0, data.len());
        let symbol = Bitmap::new(8, 8);

        let params = TextRegionParams {
            huffman: true,
            refinement: false,
            width: 100,
            height: 100,
            default_pixel_value: 0,
            number_of_symbol_instances: 1,
            strip_size: 1,
            input_symbols: vec![symbol],
            symbol_code_length: 1,
            transposed: false,
            ds_offset: 0,
            reference_corner: 0,
            combination_operator: 0,
            log_strip_size: 0,
            huffman_tables: None, // Missing tables for Huffman
            refinement_template_index: 0,
            refinement_at: vec![],
        };

        let result = decode_text_region(&params, &mut context, None);
        // Should fail because Huffman requires tables or huffman_input
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_text_region_valid_minimal() {
        let data = vec![0u8; 1000];
        let mut context = DecodingContext::new(data.clone(), 0, data.len());
        let symbol = Bitmap::new(8, 8);

        let params = TextRegionParams {
            huffman: false,
            refinement: false,
            width: 16,
            height: 16,
            default_pixel_value: 0,
            number_of_symbol_instances: 1,
            strip_size: 1,
            input_symbols: vec![symbol],
            symbol_code_length: 1,
            transposed: false,
            ds_offset: 0,
            reference_corner: 0,
            combination_operator: 0,
            log_strip_size: 0,
            huffman_tables: None,
            refinement_template_index: 0,
            refinement_at: vec![],
        };

        let result = decode_text_region(&params, &mut context, None);
        // With dummy data, this may fail, but should not panic
        let _ = result;
    }

    #[test]
    fn test_decode_text_region_with_multiple_symbols() {
        let data = vec![0u8; 1000];
        let mut context = DecodingContext::new(data.clone(), 0, data.len());
        let symbols = vec![
            Bitmap::new(4, 4),
            Bitmap::new(8, 8),
            Bitmap::new(16, 16),
        ];

        let params = TextRegionParams {
            huffman: false,
            refinement: false,
            width: 32,
            height: 32,
            default_pixel_value: 0,
            number_of_symbol_instances: 3,
            strip_size: 1,
            input_symbols: symbols,
            symbol_code_length: 2, // log2(3) ≈ 2
            transposed: false,
            ds_offset: 0,
            reference_corner: 0,
            combination_operator: 0,
            log_strip_size: 0,
            huffman_tables: None,
            refinement_template_index: 0,
            refinement_at: vec![],
        };

        let result = decode_text_region(&params, &mut context, None);
        // Should handle multiple symbols without crashing
        let _ = result;
    }

    #[test]
    fn test_decode_text_region_transposed() {
        let data = vec![0u8; 1000];
        let mut context = DecodingContext::new(data.clone(), 0, data.len());
        let symbol = Bitmap::new(8, 8);

        let params = TextRegionParams {
            huffman: false,
            refinement: false,
            width: 16,
            height: 16,
            default_pixel_value: 0,
            number_of_symbol_instances: 1,
            strip_size: 1,
            input_symbols: vec![symbol],
            symbol_code_length: 1,
            transposed: true, // Test transposed
            ds_offset: 0,
            reference_corner: 0,
            combination_operator: 0,
            log_strip_size: 0,
            huffman_tables: None,
            refinement_template_index: 0,
            refinement_at: vec![],
        };

        let result = decode_text_region(&params, &mut context, None);
        // Should handle transposed symbols without crashing
        let _ = result;
    }
}