# JBIG2 Decode Profiling Report

Profiling run across UBC fixtures using `cargo run --example decode_file --release --profile`.

## Slowest Decoder Sections (Aggregated)

| Rank | Label | Total ms | Calls | Avg ms |
| --- | --- | --- | --- | --- |
| 1 | immediate_halftone_region | 2198.523 | 160 | 13.741 |
| 2 | immediate_generic_region | 559.941 | 21 | 26.664 |
| 3 | immediate_generic_refinement_region | 415.323 | 4 | 103.831 |
| 4 | immediate_text_region | 234.256 | 10 | 23.426 |
| 5 | symbol_dictionary | 70.366 | 18 | 3.909 |
| 6 | pattern_dictionary | 20.136 | 40 | 0.503 |
| 7 | finalize_current_page | 18.321 | 67 | 0.273 |
| 8 | intermediate_text_region | 7.344 | 4 | 1.836 |
| 9 | end_of_stripe | 2.637 | 145 | 0.018 |
| 10 | page_information | 0.528 | 67 | 0.008 |
| 11 | read_segments | 0.426 | 67 | 0.006 |

## Slowest Files (Total Decode)

| Rank | File | Total ms |
| --- | --- | --- |
| 1 | 600-6-45.jb2 | 573.462 |
| 2 | 600-lossless.jb2 | 441.398 |
| 3 | 200-3-45-stripe.jb2 | 368.630 |
| 4 | 600-6-0.jb2 | 341.000 |
| 5 | 600-10-45.jb2 | 319.917 |
| 6 | 200-4-45-stripe.jb2 | 307.944 |
| 7 | 042_21.jb2 | 290.867 |
| 8 | 042_23.jb2 | 267.689 |
| 9 | 200-5-45-stripe.jb2 | 178.653 |
| 10 | 600-10-0.jb2 | 170.866 |
| 11 | 200-6-45-stripe.jb2 | 166.574 |
| 12 | 600-20-45.jb2 | 158.360 |
| 13 | 200-3-45.jb2 | 156.838 |
| 14 | 042_22.jb2 | 153.956 |
| 15 | 042_7.jb2 | 151.077 |

## Per-File Top Hotspot (Excludes total_decode)

| Rank | File | Top Label | Total ms |
| --- | --- | --- | --- |
| 1 | 600-6-45.jb2 | immediate_halftone_region | 285.622 |
| 2 | 600-lossless.jb2 | immediate_generic_region | 218.407 |
| 3 | 200-3-45-stripe.jb2 | immediate_halftone_region | 183.795 |
| 4 | 600-6-0.jb2 | immediate_halftone_region | 168.831 |
| 5 | 600-10-45.jb2 | immediate_halftone_region | 158.920 |
| 6 | 200-4-45-stripe.jb2 | immediate_halftone_region | 153.611 |
| 7 | 042_21.jb2 | immediate_generic_refinement_region | 142.720 |
| 8 | 042_23.jb2 | immediate_generic_refinement_region | 131.117 |
| 9 | 200-5-45-stripe.jb2 | immediate_halftone_region | 88.954 |
| 10 | 600-10-0.jb2 | immediate_halftone_region | 84.119 |
| 11 | 200-6-45-stripe.jb2 | immediate_halftone_region | 82.793 |
| 12 | 200-3-45.jb2 | immediate_halftone_region | 78.296 |
| 13 | 600-20-45.jb2 | immediate_halftone_region | 76.915 |
| 14 | 042_7.jb2 | immediate_generic_region | 75.400 |
| 15 | 042_22.jb2 | immediate_generic_refinement_region | 73.149 |
