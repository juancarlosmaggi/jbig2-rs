#[cfg(test)]
mod tests {
    use jbig2_rs::arithmetic::contexts::DecodingContext;
    use jbig2_rs::decoders::symbol::{SymbolDictionaryParams, decode_symbol_dictionary};

    #[test]
    fn test_decode_symbol_dictionary_zero_new_symbols() {
        let data = vec![0u8; 100];
        let mut context = DecodingContext::new(data.as_slice(), 0, data.len());

        let params = SymbolDictionaryParams {
            huffman: false,
            refinement: false,
            symbols: vec![],
            number_of_new_symbols: 0,
            number_of_exported_symbols: 0,
            template_index: 0,
            at: vec![],
            refinement_template_index: 0,
            refinement_at: &[],
            huffman_tables: None,
        };

        let result = decode_symbol_dictionary(&params, &mut context, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_symbol_dictionary_invalid_template_index() {
        let data = vec![0u8; 100];
        let mut context = DecodingContext::new(data.as_slice(), 0, data.len());

        let params = SymbolDictionaryParams {
            huffman: false,
            refinement: false,
            symbols: vec![],
            number_of_new_symbols: 1,
            number_of_exported_symbols: 0,
            template_index: 99,
            at: vec![],
            refinement_template_index: 0,
            refinement_at: &[],
            huffman_tables: None,
        };

        let result = decode_symbol_dictionary(&params, &mut context, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_symbol_dictionary_too_many_symbols() {
        let data = vec![0u8; 100];
        let mut context = DecodingContext::new(data.as_slice(), 0, data.len());

        let params = SymbolDictionaryParams {
            huffman: false,
            refinement: false,
            symbols: vec![],
            number_of_new_symbols: 20000000,
            number_of_exported_symbols: 0,
            template_index: 0,
            at: vec![],
            refinement_template_index: 0,
            refinement_at: &[],
            huffman_tables: None,
        };

        let result = decode_symbol_dictionary(&params, &mut context, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_symbol_dictionary_with_huffman_no_tables() {
        let data = vec![0u8; 1000];
        let mut context = DecodingContext::new(data.as_slice(), 0, data.len());

        let params = SymbolDictionaryParams {
            huffman: true,
            refinement: false,
            symbols: vec![],
            number_of_new_symbols: 1,
            number_of_exported_symbols: 1,
            template_index: 0,
            at: vec![],
            refinement_template_index: 0,
            refinement_at: &[],
            huffman_tables: None,
        };

        let result = decode_symbol_dictionary(&params, &mut context, None);
        assert!(result.is_err());
    }
}
