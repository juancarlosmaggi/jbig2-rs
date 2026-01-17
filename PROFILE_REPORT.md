# JBIG2 Decode Profiling Report

Profiling run across UBC fixtures using `cargo run --example decode_file --release --profile`.

Averaged over 5 run(s).

## Slowest Decoder Sections (Aggregated)

| Rank | Label | Total ms | Calls | Avg ms |
| --- | --- | --- | --- | --- |
| 1 | immediate_halftone_region | 1321.384 | 160 | 8.259 |
| 2 | immediate_generic_region | 491.955 | 21 | 23.426 |
| 3 | immediate_generic_refinement_region | 205.619 | 4 | 51.405 |
| 4 | immediate_text_region | 117.951 | 10 | 11.795 |
| 5 | symbol_dictionary | 65.586 | 18 | 3.644 |
| 6 | finalize_current_page | 17.504 | 67 | 0.261 |
| 7 | pattern_dictionary | 16.113 | 40 | 0.403 |
| 8 | intermediate_text_region | 7.484 | 4 | 1.871 |
| 9 | end_of_stripe | 3.121 | 145 | 0.022 |
| 10 | page_information | 0.581 | 67 | 0.009 |
| 11 | read_segments | 0.564 | 67 | 0.008 |

## Slowest Files (Total Decode)

| Rank | File | Total ms |
| --- | --- | --- |
| 1 | 600-6-45.jb2 | 405.801 |
| 2 | 600-lossless.jb2 | 398.429 |
| 3 | 200-3-45-stripe.jb2 | 288.164 |
| 4 | 600-10-45.jb2 | 204.175 |
| 5 | 200-4-45-stripe.jb2 | 188.512 |
| 6 | 200-5-45-stripe.jb2 | 138.010 |
| 7 | 042_23.jb2 | 123.993 |
| 8 | 042_7.jb2 | 118.879 |
| 9 | 200-6-45-stripe.jb2 | 118.129 |
| 10 | 042_21.jb2 | 116.857 |
| 11 | 200-3-45.jb2 | 110.719 |
| 12 | 042_24.jb2 | 102.241 |
| 13 | 600-6-0.jb2 | 101.688 |
| 14 | 200-4-45.jb2 | 96.531 |
| 15 | 042_22.jb2 | 96.424 |

## Per-File Top Hotspot (Excludes total_decode)

| Rank | File | Top Label | Total ms |
| --- | --- | --- | --- |
| 1 | 600-6-45.jb2 | immediate_halftone_region | 201.761 |
| 2 | 600-lossless.jb2 | immediate_generic_region | 198.011 |
| 3 | 200-3-45-stripe.jb2 | immediate_halftone_region | 143.598 |
| 4 | 600-10-45.jb2 | immediate_halftone_region | 100.881 |
| 5 | 200-4-45-stripe.jb2 | immediate_halftone_region | 93.780 |
| 6 | 200-5-45-stripe.jb2 | immediate_halftone_region | 68.516 |
| 7 | 042_7.jb2 | immediate_generic_region | 59.239 |
| 8 | 042_23.jb2 | immediate_generic_refinement_region | 58.748 |
| 9 | 200-6-45-stripe.jb2 | immediate_halftone_region | 58.542 |
| 10 | 200-3-45.jb2 | immediate_halftone_region | 55.214 |
| 11 | 042_21.jb2 | immediate_generic_refinement_region | 54.226 |
| 12 | 600-6-0.jb2 | immediate_halftone_region | 49.561 |
| 13 | 200-4-45.jb2 | immediate_halftone_region | 48.121 |
| 14 | 042_24.jb2 | immediate_generic_refinement_region | 48.115 |
| 15 | 600-20-45.jb2 | immediate_halftone_region | 45.750 |

## Notes

- Compared to the previous run (immediate_halftone_region 1368.529 ms, immediate_generic_region 503.022 ms, immediate_generic_refinement_region 215.092 ms), halftone drops to 1321.384 ms (-47.145), generic drops to 491.955 ms (-11.067), refinement drops to 205.619 ms (-9.473), and text drops to 117.951 ms (-2.055).
- Biggest file improvements: 600-6-45.jb2 428.073 -> 405.801, 200-4-45-stripe.jb2 210.074 -> 188.512, 200-5-45-stripe.jb2 156.904 -> 138.010, 600-lossless.jb2 407.952 -> 398.429.
- Regressions: 600-10-45.jb2 198.603 -> 204.175, 042_7.jb2 117.996 -> 118.879, 042_21.jb2 114.109 -> 116.857, pattern_dictionary 15.353 -> 16.113, intermediate_text_region 6.826 -> 7.484.
- Ranking shifts: 600-10-45.jb2 moved to #4 total decode; 200-4-45.jb2 entered the top-15 while 600-20-45.jb2 dropped out.
