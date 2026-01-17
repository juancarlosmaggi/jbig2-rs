#[cfg(test)]
mod tests {
    use jbig2_rs::arithmetic::ArithmeticDecoder;
    use jbig2_rs::bitmap::Bitmap;
    use jbig2_rs::contexts::DecodingContext;
    use jbig2_rs::decode::decode_text::{TextRegionParams, decode_text_region};

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
            input_symbols: vec![],
            symbol_code_length: 1,
            symbol_id_limit: 0,
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
    fn test_decode_text_region_transposed() {
        let data = vec![0u8; 1000];
        let mut context = DecodingContext::new(data.clone(), 0, data.len());
        *context.decoder.borrow_mut() = Some(ArithmeticDecoder::new(&data));
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
            symbol_id_limit: 1,
            transposed: true,
            ds_offset: 0,
            reference_corner: 0,
            combination_operator: 0,
            log_strip_size: 0,
            huffman_tables: None,
            refinement_template_index: 0,
            refinement_at: vec![],
        };

        let result = decode_text_region(&params, &mut context, None);
        let _ = result;
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
            symbol_id_limit: 1,
            transposed: false,
            ds_offset: 0,
            reference_corner: 4,
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
            symbol_id_limit: 1,
            transposed: false,
            ds_offset: 0,
            reference_corner: 0,
            combination_operator: 13,
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
            refinement: true,
            width: 100,
            height: 100,
            default_pixel_value: 0,
            number_of_symbol_instances: 1,
            strip_size: 1,
            input_symbols: vec![symbol],
            symbol_code_length: 1,
            symbol_id_limit: 1,
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
        *context.decoder.borrow_mut() = Some(ArithmeticDecoder::new(&data));
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
            symbol_id_limit: 1,
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
    fn test_decode_text_region_valid_minimal() {
        let data = vec![0u8; 1000];
        let mut context = DecodingContext::new(data.clone(), 0, data.len());
        *context.decoder.borrow_mut() = Some(ArithmeticDecoder::new(&data));
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
            symbol_id_limit: 1,
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
        let _ = result;
    }

    #[test]
    fn test_decode_text_region_with_multiple_symbols() {
        let data = vec![0u8; 1000];
        let mut context = DecodingContext::new(data.clone(), 0, data.len());
        *context.decoder.borrow_mut() = Some(ArithmeticDecoder::new(&data));
        let symbol1 = Bitmap::new(8, 8);
        let symbol2 = Bitmap::new(8, 8);

        let params = TextRegionParams {
            huffman: false,
            refinement: false,
            width: 32,
            height: 32,
            default_pixel_value: 0,
            number_of_symbol_instances: 2,
            strip_size: 1,
            input_symbols: vec![symbol1, symbol2],
            symbol_code_length: 1,
            symbol_id_limit: 2,
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
        let _ = result;
    }
}
