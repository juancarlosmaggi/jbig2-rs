#[cfg(test)]
mod tests {
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
            number_of_new_symbols: 20000000, // Too many (> 2^24-1)
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


}