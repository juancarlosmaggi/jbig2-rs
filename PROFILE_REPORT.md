# JBIG2 Decode Profiling Report

Profiling run across UBC fixtures using the release example binary (`cargo build --example decode_file --release`).

Averaged over 5 run(s).

## Slowest Decoder Sections (Aggregated)

| Rank | Label | Total ms | Calls | Avg ms |
| --- | --- | --- | --- | --- |
| 1 | immediate_halftone_region | 762.445 | 160 | 4.765 |
| 2 | immediate_generic_region | 498.433 | 21 | 23.735 |
| 3 | immediate_generic_refinement_region | 200.997 | 4 | 50.249 |
| 4 | immediate_text_region | 118.753 | 10 | 11.875 |
| 5 | symbol_dictionary | 63.893 | 18 | 3.550 |
| 6 | finalize_current_page | 19.634 | 67 | 0.293 |
| 7 | pattern_dictionary | 15.140 | 40 | 0.379 |
| 8 | intermediate_text_region | 7.075 | 4 | 1.769 |
| 9 | end_of_stripe | 3.169 | 145 | 0.022 |
| 10 | page_information | 0.562 | 67 | 0.008 |
| 11 | read_segments | 0.457 | 67 | 0.007 |

## Slowest Files (Total Decode)

| Rank | File | Total ms |
| --- | --- | --- |
| 1 | 600-lossless.jb2 | 412.834 |
| 2 | 600-6-45.jb2 | 221.803 |
| 3 | 200-3-45-stripe.jb2 | 143.524 |
| 4 | 042_7.jb2 | 119.859 |
| 5 | 042_23.jb2 | 117.257 |
| 6 | 600-10-45.jb2 | 116.151 |
| 7 | 042_21.jb2 | 110.833 |
| 8 | 042_24.jb2 | 105.124 |
| 9 | 200-4-45-stripe.jb2 | 100.638 |
| 10 | 042_22.jb2 | 94.136 |
| 11 | 600-6-0.jb2 | 89.717 |
| 12 | 200-6-45-stripe.jb2 | 76.595 |
| 13 | 042_5.jb2 | 75.564 |
| 14 | 200-5-45-stripe.jb2 | 67.113 |
| 15 | 042_25.jb2 | 63.282 |

## Per-File Top Hotspot (Excludes total_decode)

| Rank | File | Top Label | Total ms |
| --- | --- | --- | --- |
| 1 | 600-lossless.jb2 | immediate_generic_region | 205.009 |
| 2 | 600-6-45.jb2 | immediate_halftone_region | 109.522 |
| 3 | 200-3-45-stripe.jb2 | immediate_halftone_region | 71.352 |
| 4 | 042_7.jb2 | immediate_generic_region | 59.787 |
| 5 | 600-10-45.jb2 | immediate_halftone_region | 56.792 |
| 6 | 042_23.jb2 | immediate_generic_refinement_region | 55.362 |
| 7 | 042_21.jb2 | immediate_generic_refinement_region | 52.467 |
| 8 | 200-4-45-stripe.jb2 | immediate_halftone_region | 49.723 |
| 9 | 042_24.jb2 | immediate_generic_refinement_region | 49.273 |
| 10 | 042_22.jb2 | immediate_generic_refinement_region | 43.894 |
| 11 | 600-6-0.jb2 | immediate_halftone_region | 43.299 |
| 12 | 200-6-45-stripe.jb2 | immediate_halftone_region | 37.699 |
| 13 | 042_5.jb2 | immediate_generic_region | 37.589 |
| 14 | 200-5-45-stripe.jb2 | immediate_halftone_region | 33.127 |
| 15 | 042_4.jb2 | immediate_generic_region | 31.007 |

## Notes

- Compared to the previous sweep, `immediate_halftone_region` dropped from 851.204 ms to 762.445 ms (~-10.4%). `immediate_generic_region` and `immediate_generic_refinement_region` improved slightly (502.796 -> 498.433 ms, 211.431 -> 200.997 ms). `finalize_current_page` rose (18.200 -> 19.634 ms).
- File-level improvements: `600-6-45.jb2` 262.339 -> 221.803 ms, `600-10-45.jb2` 137.781 -> 116.151 ms, `600-6-0.jb2` 104.729 -> 89.717 ms. `600-10-0.jb2` and `600-20-45.jb2` fell out of the top-15 list; `200-6-45-stripe.jb2` and `042_25.jb2` entered.
- Per-file hotspots shifted: halftone hotspots improved on `600-6-45.jb2`, `600-10-45.jb2`, `600-6-0.jb2`; small regressions on `200-4-45-stripe.jb2` and `200-6-45-stripe.jb2`. `042_4.jb2` (generic) entered the top-15 hotspot list.
