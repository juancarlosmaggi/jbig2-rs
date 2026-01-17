# JBIG2 Decode Profiling Report

Profiling run across UBC fixtures using the release example binary (`cargo build --example decode_file --release`).

Averaged over 5 run(s).

## Slowest Decoder Sections (Aggregated)

| Rank | Label | Total ms | Calls | Avg ms |
| --- | --- | --- | --- | --- |
| 1 | immediate_halftone_region | 730.690 | 160 | 4.567 |
| 2 | immediate_generic_region | 480.763 | 21 | 22.893 |
| 3 | immediate_generic_refinement_region | 194.247 | 4 | 48.562 |
| 4 | immediate_text_region | 110.735 | 10 | 11.073 |
| 5 | symbol_dictionary | 58.752 | 18 | 3.264 |
| 6 | finalize_current_page | 17.306 | 67 | 0.258 |
| 7 | pattern_dictionary | 11.631 | 40 | 0.291 |
| 8 | intermediate_text_region | 7.669 | 4 | 1.917 |
| 9 | end_of_stripe | 3.251 | 145 | 0.022 |
| 10 | page_information | 0.548 | 67 | 0.008 |
| 11 | read_segments | 0.401 | 67 | 0.006 |

## Slowest Files (Total Decode)

| Rank | File | Total ms |
| --- | --- | --- |
| 1 | 600-lossless.jb2 | 422.232 |
| 2 | 600-6-45.jb2 | 211.206 |
| 3 | 200-3-45-stripe.jb2 | 137.617 |
| 4 | 600-10-45.jb2 | 111.009 |
| 5 | 042_21.jb2 | 110.266 |
| 6 | 042_23.jb2 | 107.236 |
| 7 | 042_7.jb2 | 106.668 |
| 8 | 042_24.jb2 | 106.281 |
| 9 | 200-4-45-stripe.jb2 | 94.402 |
| 10 | 042_22.jb2 | 91.476 |
| 11 | 600-6-0.jb2 | 90.284 |
| 12 | 200-5-45-stripe.jb2 | 67.698 |
| 13 | 042_5.jb2 | 67.013 |
| 14 | 042_25.jb2 | 60.406 |
| 15 | 042_4.jb2 | 59.108 |

## Per-File Top Hotspot (Excludes total_decode)

| Rank | File | Top Label | Total ms |
| --- | --- | --- | --- |
| 1 | 600-lossless.jb2 | immediate_generic_region | 209.951 |
| 2 | 600-6-45.jb2 | immediate_halftone_region | 104.384 |
| 3 | 200-3-45-stripe.jb2 | immediate_halftone_region | 68.420 |
| 4 | 600-10-45.jb2 | immediate_halftone_region | 53.734 |
| 5 | 042_7.jb2 | immediate_generic_region | 53.192 |
| 6 | 042_21.jb2 | immediate_generic_refinement_region | 52.186 |
| 7 | 042_23.jb2 | immediate_generic_refinement_region | 50.251 |
| 8 | 042_24.jb2 | immediate_generic_refinement_region | 49.315 |
| 9 | 200-4-45-stripe.jb2 | immediate_halftone_region | 46.702 |
| 10 | 600-6-0.jb2 | immediate_halftone_region | 43.889 |
| 11 | 042_22.jb2 | immediate_generic_refinement_region | 42.496 |
| 12 | 200-5-45-stripe.jb2 | immediate_halftone_region | 33.390 |
| 13 | 042_5.jb2 | immediate_generic_region | 33.385 |
| 14 | 042_4.jb2 | immediate_generic_region | 29.395 |
| 15 | 200-3-45.jb2 | immediate_halftone_region | 29.012 |

## Notes

- Compared to the previous sweep, `symbol_dictionary` improved 63.108 -> 58.752 ms and `pattern_dictionary` edged down 11.663 -> 11.631 ms. `immediate_generic_region` (470.717 -> 480.763 ms) and `immediate_generic_refinement_region` (184.283 -> 194.247 ms) regressed, while `immediate_halftone_region` stayed essentially flat (731.029 -> 730.690 ms).
- File-level improvements: `600-6-45.jb2` 214.951 -> 211.206 ms, `200-3-45-stripe.jb2` 143.463 -> 137.617 ms, and `600-10-45.jb2` 113.548 -> 111.009 ms. Regressions: `600-lossless.jb2` 404.488 -> 422.232 ms, `042_21.jb2` 105.111 -> 110.266 ms, and `042_24.jb2` 97.903 -> 106.281 ms.
- Per-file hotspots improved for `600-6-45.jb2` (halftone 106.117 -> 104.384 ms) and `600-10-45.jb2` (halftone 55.270 -> 53.734 ms). Regressions for `600-lossless.jb2` (generic 200.837 -> 209.951 ms) and `042_21.jb2` (refinement 49.525 -> 52.186 ms).
