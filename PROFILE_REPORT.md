# JBIG2 Decode Profiling Report

Profiling run across UBC fixtures using the release example binary (`cargo build --example decode_file --release`).

Averaged over 5 run(s).

## Slowest Decoder Sections (Aggregated)

| Rank | Label | Total ms | Calls | Avg ms |
| --- | --- | --- | --- | --- |
| 1 | immediate_halftone_region | 1363.713 | 160 | 8.523 |
| 2 | immediate_generic_region | 829.089 | 21 | 39.480 |
| 3 | immediate_generic_refinement_region | 529.848 | 4 | 132.462 |
| 4 | immediate_text_region | 274.728 | 10 | 27.473 |
| 5 | symbol_dictionary | 85.060 | 18 | 4.726 |
| 6 | finalize_current_page | 44.573 | 67 | 0.665 |
| 7 | pattern_dictionary | 31.712 | 40 | 0.793 |
| 8 | intermediate_text_region | 13.897 | 4 | 3.474 |
| 9 | end_of_stripe | 7.031 | 145 | 0.048 |
| 10 | page_information | 1.335 | 67 | 0.020 |
| 11 | read_segments | 0.924 | 67 | 0.014 |

## Slowest Files (Total Decode)

| Rank | File | Total ms |
| --- | --- | --- |
| 1 | 600-lossless.jb2 | 769.558 |
| 2 | 600-6-45.jb2 | 403.628 |
| 3 | 042_24.jb2 | 314.729 |
| 4 | 042_23.jb2 | 305.468 |
| 5 | 042_21.jb2 | 273.037 |
| 6 | 200-3-45-stripe.jb2 | 269.881 |
| 7 | 042_22.jb2 | 216.375 |
| 8 | 600-10-45.jb2 | 201.376 |
| 9 | 200-4-45-stripe.jb2 | 173.785 |
| 10 | 600-6-0.jb2 | 164.265 |
| 11 | 042_7.jb2 | 152.873 |
| 12 | 200-5-45-stripe.jb2 | 122.266 |
| 13 | 200-6-45-stripe.jb2 | 115.525 |
| 14 | 042_6.jb2 | 111.691 |
| 15 | 042_25.jb2 | 111.683 |

## Per-File Top Hotspot (Excludes total_decode)

| Rank | File | Top Label | Total ms |
| --- | --- | --- | --- |
| 1 | 600-lossless.jb2 | immediate_generic_region | 381.259 |
| 2 | 600-6-45.jb2 | immediate_halftone_region | 198.568 |
| 3 | 042_24.jb2 | immediate_generic_refinement_region | 150.780 |
| 4 | 042_23.jb2 | immediate_generic_refinement_region | 146.201 |
| 5 | 200-3-45-stripe.jb2 | immediate_halftone_region | 134.044 |
| 6 | 042_21.jb2 | immediate_generic_refinement_region | 130.620 |
| 7 | 042_22.jb2 | immediate_generic_refinement_region | 102.248 |
| 8 | 600-10-45.jb2 | immediate_halftone_region | 97.291 |
| 9 | 200-4-45-stripe.jb2 | immediate_halftone_region | 86.012 |
| 10 | 600-6-0.jb2 | immediate_halftone_region | 78.752 |
| 11 | 042_7.jb2 | immediate_generic_region | 76.087 |
| 12 | 200-5-45-stripe.jb2 | immediate_halftone_region | 60.183 |
| 13 | 200-6-45-stripe.jb2 | immediate_halftone_region | 56.692 |
| 14 | 042_6.jb2 | immediate_generic_region | 55.504 |
| 15 | 200-3-45.jb2 | immediate_halftone_region | 54.539 |
