# JBIG2 Decode Profiling Report

Profiling run across UBC fixtures using the release example binary (`cargo build --example decode_file --release`).

Averaged over 5 run(s).

## Slowest Decoder Sections (Aggregated)

| Rank | Label | Total ms | Calls | Avg ms |
| --- | --- | --- | --- | --- |
| 1 | immediate_halftone_region | 779.732 | 160 | 4.873 |
| 2 | immediate_generic_region | 483.430 | 21 | 23.020 |
| 3 | immediate_generic_refinement_region | 192.480 | 4 | 48.120 |
| 4 | immediate_text_region | 123.346 | 10 | 12.335 |
| 5 | symbol_dictionary | 74.166 | 18 | 4.120 |
| 6 | finalize_current_page | 17.819 | 67 | 0.266 |
| 7 | pattern_dictionary | 17.519 | 40 | 0.438 |
| 8 | intermediate_text_region | 7.748 | 4 | 1.937 |
| 9 | end_of_stripe | 2.958 | 145 | 0.020 |
| 10 | page_information | 0.694 | 67 | 0.010 |
| 11 | read_segments | 0.532 | 67 | 0.008 |

## Slowest Files (Total Decode)

| Rank | File | Total ms |
| --- | --- | --- |
| 1 | 600-lossless.jb2 | 411.603 |
| 2 | 600-6-45.jb2 | 228.357 |
| 3 | 200-3-45-stripe.jb2 | 147.352 |
| 4 | 042_21.jb2 | 115.376 |
| 5 | 600-10-45.jb2 | 110.757 |
| 6 | 042_23.jb2 | 106.281 |
| 7 | 042_7.jb2 | 102.728 |
| 8 | 042_24.jb2 | 99.865 |
| 9 | 600-6-0.jb2 | 96.848 |
| 10 | 200-4-45-stripe.jb2 | 95.353 |
| 11 | 042_22.jb2 | 92.091 |
| 12 | 200-6-45-stripe.jb2 | 74.223 |
| 13 | 042_25.jb2 | 67.558 |
| 14 | 042_5.jb2 | 64.992 |
| 15 | 042_11.jb2 | 63.464 |

## Per-File Top Hotspot (Excludes total_decode)

| Rank | File | Top Label | Total ms |
| --- | --- | --- | --- |
| 1 | 600-lossless.jb2 | immediate_generic_region | 204.548 |
| 2 | 600-6-45.jb2 | immediate_halftone_region | 112.836 |
| 3 | 200-3-45-stripe.jb2 | immediate_halftone_region | 73.212 |
| 4 | 600-10-45.jb2 | immediate_halftone_region | 54.276 |
| 5 | 042_21.jb2 | immediate_generic_refinement_region | 53.936 |
| 6 | 042_7.jb2 | immediate_generic_region | 51.237 |
| 7 | 042_23.jb2 | immediate_generic_refinement_region | 49.614 |
| 8 | 200-4-45-stripe.jb2 | immediate_halftone_region | 47.197 |
| 9 | 600-6-0.jb2 | immediate_halftone_region | 46.706 |
| 10 | 042_24.jb2 | immediate_generic_refinement_region | 46.289 |
| 11 | 042_22.jb2 | immediate_generic_refinement_region | 42.642 |
| 12 | 200-6-45-stripe.jb2 | immediate_halftone_region | 36.436 |
| 13 | 042_5.jb2 | immediate_generic_region | 32.370 |
| 14 | 200-5-45-stripe.jb2 | immediate_halftone_region | 31.189 |
| 15 | 042_4.jb2 | immediate_generic_region | 29.617 |
