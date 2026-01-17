# JBIG2 Decode Profiling Report

Profiling run across UBC fixtures using `cargo run --example decode_file --release --profile`.

Averaged over 5 run(s).

## Slowest Decoder Sections (Aggregated)

| Rank | Label | Total ms | Calls | Avg ms |
| --- | --- | --- | --- | --- |
| 1 | immediate_halftone_region | 1393.862 | 160 | 8.712 |
| 2 | immediate_generic_region | 519.311 | 21 | 24.729 |
| 3 | immediate_generic_refinement_region | 320.840 | 4 | 80.210 |
| 4 | immediate_text_region | 204.387 | 10 | 20.439 |
| 5 | symbol_dictionary | 68.267 | 18 | 3.793 |
| 6 | finalize_current_page | 18.377 | 67 | 0.274 |
| 7 | pattern_dictionary | 15.618 | 40 | 0.390 |
| 8 | intermediate_text_region | 7.156 | 4 | 1.789 |
| 9 | end_of_stripe | 3.222 | 145 | 0.022 |
| 10 | page_information | 0.773 | 67 | 0.012 |
| 11 | read_segments | 0.285 | 67 | 0.004 |

## Slowest Files (Total Decode)

| Rank | File | Total ms |
| --- | --- | --- |
| 1 | 600-lossless.jb2 | 431.254 |
| 2 | 600-6-45.jb2 | 426.251 |
| 3 | 200-3-45-stripe.jb2 | 305.600 |
| 4 | 200-4-45-stripe.jb2 | 229.018 |
| 5 | 600-10-45.jb2 | 203.119 |
| 6 | 042_23.jb2 | 192.977 |
| 7 | 042_21.jb2 | 188.364 |
| 8 | 200-5-45-stripe.jb2 | 149.502 |
| 9 | 042_22.jb2 | 149.184 |
| 10 | 042_24.jb2 | 137.225 |
| 11 | 200-6-45-stripe.jb2 | 124.007 |
| 12 | 042_7.jb2 | 119.040 |
| 13 | 600-6-0.jb2 | 113.903 |
| 14 | 200-3-45.jb2 | 113.659 |
| 15 | 600-20-45.jb2 | 104.501 |

## Per-File Top Hotspot (Excludes total_decode)

| Rank | File | Top Label | Total ms |
| --- | --- | --- | --- |
| 1 | 600-lossless.jb2 | immediate_generic_region | 214.205 |
| 2 | 600-6-45.jb2 | immediate_halftone_region | 211.967 |
| 3 | 200-3-45-stripe.jb2 | immediate_halftone_region | 152.416 |
| 4 | 200-4-45-stripe.jb2 | immediate_halftone_region | 113.963 |
| 5 | 600-10-45.jb2 | immediate_halftone_region | 100.026 |
| 6 | 042_23.jb2 | immediate_generic_refinement_region | 92.957 |
| 7 | 042_21.jb2 | immediate_generic_refinement_region | 90.799 |
| 8 | 200-5-45-stripe.jb2 | immediate_halftone_region | 74.293 |
| 9 | 042_22.jb2 | immediate_generic_refinement_region | 71.680 |
| 10 | 042_24.jb2 | immediate_generic_refinement_region | 65.404 |
| 11 | 200-6-45-stripe.jb2 | immediate_halftone_region | 61.433 |
| 12 | 042_7.jb2 | immediate_generic_region | 59.339 |
| 13 | 200-3-45.jb2 | immediate_halftone_region | 56.668 |
| 14 | 600-6-0.jb2 | immediate_halftone_region | 55.730 |
| 15 | 600-20-45.jb2 | immediate_halftone_region | 50.058 |

## Notes
- Compared to the prior run (halftone 1475.208 ms, generic 517.998 ms, refinement 330.699 ms), halftone improves to 1393.862 ms and refinement to 320.840 ms; generic edges up to 519.311 ms.
- Biggest file improvements: 600-6-45.jb2 (~-23 ms), 600-10-45.jb2 (~-26 ms), 042_22.jb2 (~-24 ms), 600-20-45.jb2 (~-14 ms).
- Regressions: 042_23.jb2 (~+9 ms), 200-3-45-stripe.jb2 (~+7 ms), 200-4-45-stripe.jb2 (~+5 ms).
- Ranking shift: 600-lossless.jb2 is now the slowest file (previously 600-6-45.jb2); immediate_halftone_region remains the dominant hotspot.
