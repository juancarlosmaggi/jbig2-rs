# JBIG2 Decode Profiling Report

Profiling run across UBC fixtures using the release example binary (`cargo build --example decode_file --release`).

Averaged over 1 run(s).

## Slowest Decoder Sections (Aggregated)

| Rank | Label | Total ms | Calls | Avg ms |
| --- | --- | --- | --- | --- |
| 1 | immediate_halftone_region | 423.656 | 160 | 2.648 |
| 2 | immediate_generic_region | 292.545 | 21 | 13.931 |
| 3 | immediate_generic_refinement_region | 99.575 | 4 | 24.894 |
| 4 | immediate_text_region | 69.845 | 10 | 6.985 |
| 5 | symbol_dictionary | 33.683 | 18 | 1.871 |
| 6 | pattern_dictionary | 6.612 | 40 | 0.165 |
| 7 | intermediate_text_region | 3.958 | 4 | 0.989 |
| 8 | finalize_current_page | 1.455 | 67 | 0.022 |
| 9 | read_segments | 0.365 | 67 | 0.005 |
| 10 | end_of_stripe | 0.172 | 145 | 0.001 |
| 11 | page_information | 0.126 | 67 | 0.002 |

## Slowest Files (Total Decode)

| Rank | File | Total ms |
| --- | --- | --- |
| 1 | 600-lossless.jb2 | 306.349 |
| 2 | 600-6-45.jb2 | 120.395 |
| 3 | 200-3-45-stripe.jb2 | 81.122 |
| 4 | 600-6-0.jb2 | 59.519 |
| 5 | 600-10-45.jb2 | 58.633 |
| 6 | 042_24.jb2 | 56.172 |
| 7 | 042_21.jb2 | 55.533 |
| 8 | 042_23.jb2 | 55.215 |
| 9 | 200-4-45-stripe.jb2 | 53.816 |
| 10 | 042_22.jb2 | 46.714 |
| 11 | 042_1.jb2 | 44.840 |
| 12 | 200-5-45-stripe.jb2 | 37.906 |
| 13 | 042_25.jb2 | 36.829 |
| 14 | 042_7.jb2 | 36.204 |
| 15 | 200-lossless.jb2 | 35.432 |

## Per-File Top Hotspot (Excludes total_decode)

| Rank | File | Top Label | Total ms |
| --- | --- | --- | --- |
| 1 | 600-lossless.jb2 | immediate_generic_region | 153.087 |
| 2 | 600-6-45.jb2 | immediate_halftone_region | 60.095 |
| 3 | 200-3-45-stripe.jb2 | immediate_halftone_region | 40.517 |
| 4 | 600-6-0.jb2 | immediate_halftone_region | 29.637 |
| 5 | 600-10-45.jb2 | immediate_halftone_region | 29.190 |
| 6 | 200-4-45-stripe.jb2 | immediate_halftone_region | 26.861 |
| 7 | 042_24.jb2 | immediate_generic_refinement_region | 26.227 |
| 8 | 042_21.jb2 | immediate_generic_refinement_region | 25.987 |
| 9 | 042_23.jb2 | immediate_generic_refinement_region | 25.817 |
| 10 | 042_1.jb2 | immediate_generic_region | 22.299 |
| 11 | 042_22.jb2 | immediate_generic_refinement_region | 21.544 |
| 12 | 200-5-45-stripe.jb2 | immediate_halftone_region | 18.883 |
| 13 | 042_7.jb2 | immediate_generic_region | 18.089 |
| 14 | 200-lossless.jb2 | immediate_generic_region | 17.702 |
| 15 | 200-3-45.jb2 | immediate_halftone_region | 17.308 |
