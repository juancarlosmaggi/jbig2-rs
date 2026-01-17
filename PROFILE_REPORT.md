# JBIG2 Decode Profiling Report

Profiling run across UBC fixtures using the release example binary (`cargo build --example decode_file --release`).

Averaged over 5 run(s).

## Slowest Decoder Sections (Aggregated)

| Rank | Label | Total ms | Calls | Avg ms |
| --- | --- | --- | --- | --- |
| 1 | immediate_halftone_region | 851.204 | 160 | 5.320 |
| 2 | immediate_generic_region | 502.796 | 21 | 23.943 |
| 3 | immediate_generic_refinement_region | 211.431 | 4 | 52.858 |
| 4 | immediate_text_region | 119.134 | 10 | 11.913 |
| 5 | symbol_dictionary | 64.123 | 18 | 3.562 |
| 6 | finalize_current_page | 18.200 | 67 | 0.272 |
| 7 | pattern_dictionary | 15.102 | 40 | 0.378 |
| 8 | intermediate_text_region | 6.866 | 4 | 1.717 |
| 9 | end_of_stripe | 3.079 | 145 | 0.021 |
| 10 | page_information | 0.544 | 67 | 0.008 |
| 11 | read_segments | 0.436 | 67 | 0.007 |

## Slowest Files (Total Decode)

| Rank | File | Total ms |
| --- | --- | --- |
| 1 | 600-lossless.jb2 | 410.907 |
| 2 | 600-6-45.jb2 | 262.339 |
| 3 | 200-3-45-stripe.jb2 | 152.766 |
| 4 | 600-10-45.jb2 | 137.781 |
| 5 | 042_23.jb2 | 122.331 |
| 6 | 042_21.jb2 | 120.379 |
| 7 | 042_7.jb2 | 118.093 |
| 8 | 042_24.jb2 | 107.010 |
| 9 | 600-6-0.jb2 | 104.729 |
| 10 | 042_22.jb2 | 97.651 |
| 11 | 200-4-45-stripe.jb2 | 97.646 |
| 12 | 042_5.jb2 | 78.071 |
| 13 | 600-10-0.jb2 | 69.249 |
| 14 | 200-5-45-stripe.jb2 | 67.344 |
| 15 | 600-20-45.jb2 | 66.990 |

## Per-File Top Hotspot (Excludes total_decode)

| Rank | File | Top Label | Total ms |
| --- | --- | --- | --- |
| 1 | 600-lossless.jb2 | immediate_generic_region | 204.173 |
| 2 | 600-6-45.jb2 | immediate_halftone_region | 129.528 |
| 3 | 200-3-45-stripe.jb2 | immediate_halftone_region | 75.973 |
| 4 | 600-10-45.jb2 | immediate_halftone_region | 67.541 |
| 5 | 042_7.jb2 | immediate_generic_region | 58.915 |
| 6 | 042_23.jb2 | immediate_generic_refinement_region | 57.773 |
| 7 | 042_21.jb2 | immediate_generic_refinement_region | 57.243 |
| 8 | 600-6-0.jb2 | immediate_halftone_region | 51.044 |
| 9 | 042_24.jb2 | immediate_generic_refinement_region | 50.466 |
| 10 | 200-4-45-stripe.jb2 | immediate_halftone_region | 48.396 |
| 11 | 042_22.jb2 | immediate_generic_refinement_region | 45.949 |
| 12 | 042_5.jb2 | immediate_generic_region | 38.900 |
| 13 | 600-10-0.jb2 | immediate_halftone_region | 33.273 |
| 14 | 200-5-45-stripe.jb2 | immediate_halftone_region | 33.225 |
| 15 | 200-6-45-stripe.jb2 | immediate_halftone_region | 32.517 |
