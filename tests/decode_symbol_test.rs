#[cfg(test)]
mod tests {
    use jbig2_rs::bitmap::Bitmap;
    use jbig2_rs::contexts::DecodingContext;
    use jbig2_rs::decode::decode_symbol::{decode_symbol_dictionary, SymbolDictionaryParams};

    #[test]
    fn test_decode_symbol_dictionary_zero_new_symbols() {
        let data = vec![0u8; 100];
        let mut context = DecodingContext::new(data.clone(), 0, data.len());

        let params = SymbolDictionaryParams {
            huffman: false,
            refinement: false,
            symbols: vec![],
            number_of_new_symbols: 0, // Invalid
            number_of_exported_symbols: 0,
            template_index: 0,
            at: vec![],
            refinement_template_index: 0,
            refinement_at: vec![],
            huffman_tables: None,
        };

        let result = decode_symbol_dictionary(&params, &mut context, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_symbol_dictionary_invalid_template_index() {
        let data = vec![0u8; 100];
        let mut context = DecodingContext::new(data.clone(), 0, data.len());

        let params = SymbolDictionaryParams {
            huffman: false,
            refinement: false,
            symbols: vec![],
            number_of_new_symbols: 1,
            number_of_exported_symbols: 0,
            template_index: 99, // Invalid
            at: vec![],
            refinement_template_index: 0,
            refinement_at: vec![],
            huffman_tables: None,
        };

        let result = decode_symbol_dictionary(&params, &mut context, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_symbol_dictionary_too_many_symbols() {
        let data = vec![0u8; 100];
        let mut context = DecodingContext::new(data.clone(), 0, data.len());

        let params = SymbolDictionaryParams {
            huffman: false,
            refinement: false,
            symbols: vec![],
            number_of_new_symbols: 1000000, // Too many
            number_of_exported_symbols: 0,
            template_index: 0,
            at: vec![],
            refinement_template_index: 0,
            refinement_at: vec![],
            huffman_tables: None,
        };

        let result = decode_symbol_dictionary(&params, &mut context, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_symbol_dictionary_with_existing_symbols() {
        let data = vec![0u8; 1000];
        let mut context = DecodingContext::new(data.clone(), 0, data.len());
        let existing_symbols = vec![
            Bitmap::new(8, 8),
            Bitmap::new(16, 16),
        ];

        let params = SymbolDictionaryParams {
            huffman: false,
            refinement: false,
            symbols: existing_symbols,
            number_of_new_symbols: 2,
            number_of_exported_symbols: 2,
            template_index: 0,
            at: vec![],
            refinement_template_index: 0,
            refinement_at: vec![],
            huffman_tables: None,
        };

        let result = decode_symbol_dictionary(&params, &mut context, None);
        // With dummy data, this may fail, but should not panic
        let _ = result;
    }

    #[test]
    fn test_decode_symbol_dictionary_with_refinement() {
        let data = vec![0u8; 1000];
        let mut context = DecodingContext::new(data.clone(), 0, data.len());
        let existing_symbols = vec![Bitmap::new(8, 8)];

        let params = SymbolDictionaryParams {
            huffman: false,
            refinement: true,
            symbols: existing_symbols,
            number_of_new_symbols: 1,
            number_of_exported_symbols: 1,
            template_index: 0,
            at: vec![],
            refinement_template_index: 0,
            refinement_at: vec![],
            huffman_tables: None,
        };

        let result = decode_symbol_dictionary(&params, &mut context, None);
        // Should handle refinement without crashing
        let _ = result;
    }

    #[test]
    fn test_decode_symbol_dictionary_with_huffman_no_tables() {
        let data = vec![0u8; 1000];
        let mut context = DecodingContext::new(data.clone(), 0, data.len());

        let params = SymbolDictionaryParams {
            huffman: true,
            refinement: false,
            symbols: vec![],
            number_of_new_symbols: 1,
            number_of_exported_symbols: 1,
            template_index: 0,
            at: vec![],
            refinement_template_index: 0,
            refinement_at: vec![],
            huffman_tables: None, // Missing tables for Huffman
        };

        let result = decode_symbol_dictionary(&params, &mut context, None);
        // Should fail because Huffman requires tables
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_symbol_dictionary_minimal_valid() {
        let data = vec![0u8; 1000];
        let mut context = DecodingContext::new(data.clone(), 0, data.len());

        let params = SymbolDictionaryParams {
            huffman: false,
            refinement: false,
            symbols: vec![],
            number_of_new_symbols: 1,
            number_of_exported_symbols: 1,
            template_index: 0,
            at: vec![],
            refinement_template_index: 0,
            refinement_at: vec![],
            huffman_tables: None,
        };

        let result = decode_symbol_dictionary(&params, &mut context, None);
        // With dummy data, this may fail, but should not panic
        let _ = result;
    }

    #[test]
    fn test_decode_symbol_dictionary_different_templates() {
        let templates = vec![0, 1, 2, 3];

        for template_index in templates {
            let data = vec![0u8; 1000];
            let mut context = DecodingContext::new(data.clone(), 0, data.len());

            let params = SymbolDictionaryParams {
                huffman: false,
                refinement: false,
                symbols: vec![],
                number_of_new_symbols: 1,
                number_of_exported_symbols: 1,
                template_index,
                at: vec![],
                refinement_template_index: 0,
                refinement_at: vec![],
                huffman_tables: None,
            };

            let result = decode_symbol_dictionary(&params, &mut context, None);
            // Should handle different templates without crashing
            let _ = result;
        }
    }

    #[test]
    fn test_decode_symbol_dictionary_with_custom_at_pixels() {
        let data = vec![0u8; 1000];
        let mut context = DecodingContext::new(data.clone(), 0, data.len());

        let params = SymbolDictionaryParams {
            huffman: false,
            refinement: false,
            symbols: vec![],
            number_of_new_symbols: 1,
            number_of_exported_symbols: 1,
            template_index: 0,
            at: vec![(1, -1), (2, -1)], // Custom AT pixels
            refinement_template_index: 0,
            refinement_at: vec![],
            huffman_tables: None,
        };

        let result = decode_symbol_dictionary(&params, &mut context, None);
        // Should handle custom AT pixels without crashing
        let _ = result;
    }
}