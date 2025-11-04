#[cfg(test)]
mod tests {
    use jbig2_rs::contexts::DecodingContext;
    use jbig2_rs::decode::decode_pattern::{PatternDictionaryParams, decode_pattern_dictionary};

    #[test]
    fn test_decode_pattern_dictionary_zero_patterns() {
        let data = vec![0u8; 100];
        let mut context = DecodingContext::new(data.clone(), 0, data.len());

        let params = PatternDictionaryParams {
            mmr: false,
            pattern_width: 8,
            pattern_height: 8,
            max_pattern_index: 0, // Only one pattern (index 0)
            template: 0,
        };

        let result = decode_pattern_dictionary(&params, &mut context);
        // With dummy data, this may fail, but should not panic
        let _ = result;
    }

    #[test]
    fn test_decode_pattern_dictionary_invalid_dimensions() {
        let data = vec![0u8; 100];
        let mut context = DecodingContext::new(data.clone(), 0, data.len());

        let params = PatternDictionaryParams {
            mmr: false,
            pattern_width: 0, // Invalid
            pattern_height: 8,
            max_pattern_index: 1,
            template: 0,
        };

        let result = decode_pattern_dictionary(&params, &mut context);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_pattern_dictionary_invalid_template() {
        let data = vec![0u8; 100];
        let mut context = DecodingContext::new(data.clone(), 0, data.len());

        let params = PatternDictionaryParams {
            mmr: false,
            pattern_width: 8,
            pattern_height: 8,
            max_pattern_index: 1,
            template: 99, // Invalid template
        };

        let result = decode_pattern_dictionary(&params, &mut context);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_pattern_dictionary_mmr_mode() {
        let data = vec![0u8; 1000];
        let mut context = DecodingContext::new(data.clone(), 0, data.len());

        let params = PatternDictionaryParams {
            mmr: true, // Test MMR mode
            pattern_width: 8,
            pattern_height: 8,
            max_pattern_index: 3,
            template: 0,
        };

        let result = decode_pattern_dictionary(&params, &mut context);
        // Should handle MMR mode without crashing
        let _ = result;
    }

    #[test]
    fn test_decode_pattern_dictionary_different_templates() {
        let templates = vec![0, 1, 2, 3];

        for template in templates {
            let data = vec![0u8; 1000];
            let mut context = DecodingContext::new(data.clone(), 0, data.len());

            let params = PatternDictionaryParams {
                mmr: false,
                pattern_width: 4,
                pattern_height: 4,
                max_pattern_index: 1,
                template,
            };

            let result = decode_pattern_dictionary(&params, &mut context);
            // Should handle different templates without crashing
            let _ = result;
        }
    }

    #[test]
    fn test_decode_pattern_dictionary_multiple_patterns() {
        let data = vec![0u8; 2000];
        let mut context = DecodingContext::new(data.clone(), 0, data.len());

        let params = PatternDictionaryParams {
            mmr: false,
            pattern_width: 8,
            pattern_height: 8,
            max_pattern_index: 7, // 8 patterns total
            template: 0,
        };

        let result = decode_pattern_dictionary(&params, &mut context);
        // Should handle multiple patterns without crashing
        let _ = result;
    }

    #[test]
    fn test_decode_pattern_dictionary_large_patterns() {
        let data = vec![0u8; 10000];
        let mut context = DecodingContext::new(data.clone(), 0, data.len());

        let params = PatternDictionaryParams {
            mmr: false,
            pattern_width: 32,
            pattern_height: 32,
            max_pattern_index: 1,
            template: 0,
        };

        let result = decode_pattern_dictionary(&params, &mut context);
        // Should handle large patterns without crashing
        let _ = result;
    }

    #[test]
    fn test_decode_pattern_dictionary_different_sizes() {
        let sizes = vec![(4, 4), (8, 8), (16, 16), (2, 8), (8, 2)];

        for (width, height) in sizes {
            let data = vec![0u8; 1000];
            let mut context = DecodingContext::new(data.clone(), 0, data.len());

            let params = PatternDictionaryParams {
                mmr: false,
                pattern_width: width,
                pattern_height: height,
                max_pattern_index: 1,
                template: 0,
            };

            let result = decode_pattern_dictionary(&params, &mut context);
            // Should handle different sizes without crashing
            let _ = result;
        }
    }

    #[test]
    fn test_decode_pattern_dictionary_max_pattern_index() {
        let max_indices = vec![0, 1, 15, 255];

        for max_index in max_indices {
            let data = vec![0u8; 10000];
            let mut context = DecodingContext::new(data.clone(), 0, data.len());

            let params = PatternDictionaryParams {
                mmr: false,
                pattern_width: 4,
                pattern_height: 4,
                max_pattern_index: max_index,
                template: 0,
            };

            let result = decode_pattern_dictionary(&params, &mut context);
            // Should handle different max indices without crashing
            let _ = result;
        }
    }
}
