#[cfg(test)]
mod tests {
    use rstest::rstest;

    #[rstest]
    #[case(0, 12)]
    #[case(1, 12)]
    #[case(2, 9)]
    #[case(3, 9)]
    #[case(4, 0)]
    #[case(99, 0)]
    fn test_get_coding_template(#[case] index: usize, #[case] expected_len: usize) {
        let template = jbig2_rs::decode::decode_generic::get_coding_template(index);
        assert_eq!(template.len(), expected_len);

        // Ensure template coordinates are within expected bounds.
        for &(x, y) in template {
            assert!((-4..=2).contains(&x));
            assert!((-2..=0).contains(&y));
        }
    }

    #[test]
    fn test_get_coding_template_template0() {
        let template = jbig2_rs::decode::decode_generic::get_coding_template(0);
        assert_eq!(template.len(), 12);
        // Spot-check specific positions.
        assert_eq!(template[0], (-1, -2));
        assert_eq!(template[1], (0, -2));
        assert_eq!(template[2], (1, -2));
        assert_eq!(template[3], (-2, -1));
        assert_eq!(template[4], (-1, -1));
        assert_eq!(template[5], (0, -1));
        assert_eq!(template[6], (1, -1));
        assert_eq!(template[7], (2, -1));
        assert_eq!(template[8], (-4, 0));
        assert_eq!(template[9], (-3, 0));
        assert_eq!(template[10], (-2, 0));
        assert_eq!(template[11], (-1, 0));
    }
}
