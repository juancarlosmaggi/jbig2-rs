# JBIG2 Decode Profiling Report

Profiling run across UBC fixtures using the release example binary (`cargo build --example decode_file --release`).

Averaged over 5 run(s).

## Slowest Decoder Sections (Aggregated)

| Rank | Label | Total ms | Calls | Avg ms |
| --- | --- | --- | --- | --- |
| 1 | immediate_halftone_region | 1356.103 | 160 | 8.476 |
| 2 | immediate_generic_region | 818.358 | 21 | 38.969 |
| 3 | immediate_generic_refinement_region | 542.598 | 4 | 135.650 |
| 4 | immediate_text_region | 295.971 | 10 | 29.597 |
| 5 | symbol_dictionary | 84.692 | 18 | 4.705 |
| 6 | finalize_current_page | 47.523 | 67 | 0.709 |
| 7 | pattern_dictionary | 32.389 | 40 | 0.810 |
| 8 | intermediate_text_region | 13.517 | 4 | 3.379 |
| 9 | end_of_stripe | 7.500 | 145 | 0.052 |
| 10 | page_information | 1.368 | 67 | 0.020 |
| 11 | read_segments | 0.985 | 67 | 0.015 |

## Slowest Files (Total Decode)

| Rank | File | Total ms |
| --- | --- | --- |
| 1 | 600-lossless.jb2 | 770.992 |
| 2 | 600-6-45.jb2 | 399.394 |
| 3 | 042_23.jb2 | 300.917 |
| 4 | 042_21.jb2 | 300.842 |
| 5 | 042_24.jb2 | 284.457 |
| 6 | 200-3-45-stripe.jb2 | 258.464 |
| 7 | 042_22.jb2 | 247.380 |
| 8 | 600-10-45.jb2 | 206.238 |
| 9 | 200-4-45-stripe.jb2 | 177.019 |
| 10 | 600-6-0.jb2 | 167.046 |
| 11 | 042_7.jb2 | 152.084 |
| 12 | 200-5-45-stripe.jb2 | 123.299 |
| 13 | 200-6-45-stripe.jb2 | 112.175 |
| 14 | 042_25.jb2 | 111.645 |
| 15 | 200-3-45.jb2 | 104.932 |

## Per-File Top Hotspot (Excludes total_decode)

| Rank | File | Top Label | Total ms |
| --- | --- | --- | --- |
| 1 | 600-lossless.jb2 | immediate_generic_region | 381.840 |
| 2 | 600-6-45.jb2 | immediate_halftone_region | 196.022 |
| 3 | 042_23.jb2 | immediate_generic_refinement_region | 144.456 |
| 4 | 042_21.jb2 | immediate_generic_refinement_region | 144.393 |
| 5 | 042_24.jb2 | immediate_generic_refinement_region | 136.193 |
| 6 | 200-3-45-stripe.jb2 | immediate_halftone_region | 128.254 |
| 7 | 042_22.jb2 | immediate_generic_refinement_region | 117.556 |
| 8 | 600-10-45.jb2 | immediate_halftone_region | 99.417 |
| 9 | 200-4-45-stripe.jb2 | immediate_halftone_region | 87.529 |
| 10 | 600-6-0.jb2 | immediate_halftone_region | 80.018 |
| 11 | 042_7.jb2 | immediate_generic_region | 75.669 |
| 12 | 200-5-45-stripe.jb2 | immediate_halftone_region | 60.663 |
| 13 | 200-6-45-stripe.jb2 | immediate_halftone_region | 54.914 |
| 14 | 200-3-45.jb2 | immediate_halftone_region | 52.093 |
| 15 | 042_5.jb2 | immediate_generic_region | 51.998 |
