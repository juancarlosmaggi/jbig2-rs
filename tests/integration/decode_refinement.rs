#[cfg(test)]
mod tests {
    use jbig2_rs::arithmetic::contexts::DecodingContext;
    use jbig2_rs::bitmap::Bitmap;
    use jbig2_rs::decoders::refinement::{
        RefinementParams, decode_refinement, get_refinement_template,
    };

    #[test]
    fn test_get_refinement_template() {
        let templates = vec![0, 1, 2, 99];

        for index in templates {
            let template = get_refinement_template(index);

            if index <= 1 {
                assert!(!template.coding.is_empty());
                assert!(!template.reference.is_empty());
            } else {
                assert!(template.coding.is_empty());
                assert!(template.reference.is_empty());
            }
        }
    }

    #[test]
    fn test_get_refinement_template_template0() {
        let template = get_refinement_template(0);
        assert_eq!(template.coding.len(), 3);
        assert_eq!(template.reference.len(), 8);

        // Spot-check coding positions.
        assert_eq!(template.coding[0], (-1, 0));
        assert_eq!(template.coding[1], (1, -1));
        assert_eq!(template.coding[2], (0, -1));

        // Spot-check reference positions.
        assert_eq!(template.reference[0], (1, 1));
        assert_eq!(template.reference[1], (0, 1));
        assert_eq!(template.reference[2], (-1, 1));
        assert_eq!(template.reference[3], (1, 0));
    }

    #[test]
    fn test_get_refinement_template_template1() {
        let template = get_refinement_template(1);
        assert_eq!(template.coding.len(), 4);
        assert_eq!(template.reference.len(), 6);

        // Spot-check coding positions.
        assert_eq!(template.coding[0], (-1, 0));
        assert_eq!(template.coding[1], (1, -1));
        assert_eq!(template.coding[2], (0, -1));
        assert_eq!(template.coding[3], (-1, -1));
    }

    #[test]
    fn test_decode_refinement_invalid_dimensions() {
        let data = vec![0u8; 100];
        let mut context = DecodingContext::new(data.as_slice(), 0, data.len());
        let reference = Bitmap::new(16, 16);
        let at = [(0, 0), (0, 0)];

        let params = RefinementParams {
            width: 0,
            height: 16,
            template_index: 0,
            reference_bitmap: &reference,
            offset_x: 0,
            offset_y: 0,
            prediction: false,
            at: &at[..1],
        };

        let result = decode_refinement(&params, &mut context);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_refinement_invalid_template_index() {
        let data = vec![0u8; 100];
        let mut context = DecodingContext::new(data.as_slice(), 0, data.len());
        let reference = Bitmap::new(16, 16);
        let at = [(0, 0), (0, 0)];

        let params = RefinementParams {
            width: 16,
            height: 16,
            template_index: 99,
            reference_bitmap: &reference,
            offset_x: 0,
            offset_y: 0,
            prediction: false,
            at: &at,
        };

        let result = decode_refinement(&params, &mut context);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_refinement_with_offsets() {
        let data = vec![0u8; 1000];
        let mut context = DecodingContext::new(data.as_slice(), 0, data.len());
        let reference = Bitmap::new(32, 32);
        let at = [(0, 0), (0, 0)];

        let params = RefinementParams {
            width: 16,
            height: 16,
            template_index: 0,
            reference_bitmap: &reference,
            offset_x: 8,
            offset_y: 8,
            prediction: false,
            at: &at[..1],
        };

        let result = decode_refinement(&params, &mut context);
        let _ = result;
    }

    #[test]
    fn test_decode_refinement_with_prediction() {
        let data = vec![0u8; 1000];
        let mut context = DecodingContext::new(data.as_slice(), 0, data.len());
        let reference = Bitmap::new(16, 16);
        let at = [(0, 0), (0, 0)];

        let params = RefinementParams {
            width: 16,
            height: 16,
            template_index: 0,
            reference_bitmap: &reference,
            offset_x: 0,
            offset_y: 0,
            prediction: true,
            at: &at,
        };

        let result = decode_refinement(&params, &mut context);
        let _ = result;
    }

    #[test]
    fn test_decode_refinement_different_templates() {
        let templates = vec![0, 1];

        for template_index in templates {
            let data = vec![0u8; 1000];
            let mut context = DecodingContext::new(data.as_slice(), 0, data.len());
            let reference = Bitmap::new(16, 16);
            let at = [(0, 0), (0, 0)];

            let at = if template_index == 0 {
                &at[..1]
            } else {
                &at[..0]
            };

            let params = RefinementParams {
                width: 8,
                height: 8,
                template_index,
                reference_bitmap: &reference,
                offset_x: 0,
                offset_y: 0,
                prediction: false,
                at,
            };

            let result = decode_refinement(&params, &mut context);
            let _ = result;
        }
    }

    #[test]
    fn test_decode_refinement_with_custom_at_pixels() {
        let data = vec![0u8; 1000];
        let mut context = DecodingContext::new(data.as_slice(), 0, data.len());
        let reference = Bitmap::new(16, 16);
        let at = [(1, -1), (2, -1)];

        let params = RefinementParams {
            width: 8,
            height: 8,
            template_index: 0,
            reference_bitmap: &reference,
            offset_x: 0,
            offset_y: 0,
            prediction: false,
            at: &at,
        };

        let result = decode_refinement(&params, &mut context);
        let _ = result;
    }

    #[test]
    fn test_decode_refinement_different_sizes() {
        let sizes = vec![(4, 4), (8, 8), (16, 16), (32, 32)];

        for (width, height) in sizes {
            let data = vec![0u8; 1000];
            let mut context = DecodingContext::new(data.as_slice(), 0, data.len());
            let reference = Bitmap::new(width * 2, height * 2);
            let at = [(0, 0), (0, 0)];

            let params = RefinementParams {
                width,
                height,
                template_index: 0,
                reference_bitmap: &reference,
                offset_x: 0,
                offset_y: 0,
                prediction: false,
                at: &at[..1],
            };

            let result = decode_refinement(&params, &mut context);
            let _ = result;
        }
    }
}
