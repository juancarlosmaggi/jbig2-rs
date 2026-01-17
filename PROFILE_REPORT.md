# JBIG2 Decode Profiling Report

Profiling run across UBC fixtures using `cargo run --example decode_file --release --profile`.

Averaged over 5 run(s).

## Slowest Decoder Sections (Aggregated)

| Rank | Label | Total ms | Calls | Avg ms |
| --- | --- | --- | --- | --- |
| 1 | immediate_halftone_region | 1482.086 | 160 | 9.263 |
| 2 | immediate_generic_region | 521.717 | 21 | 24.844 |
| 3 | immediate_generic_refinement_region | 319.595 | 4 | 79.899 |
| 4 | immediate_text_region | 213.212 | 10 | 21.321 |
| 5 | symbol_dictionary | 71.510 | 18 | 3.973 |
| 6 | finalize_current_page | 19.116 | 67 | 0.285 |
| 7 | pattern_dictionary | 16.243 | 40 | 0.406 |
| 8 | intermediate_text_region | 7.566 | 4 | 1.892 |
| 9 | end_of_stripe | 4.413 | 145 | 0.030 |
| 10 | read_segments | 0.688 | 67 | 0.010 |
| 11 | page_information | 0.681 | 67 | 0.010 |

## Slowest Files (Total Decode)

| Rank | File | Total ms |
| --- | --- | --- |
| 1 | 600-6-45.jb2 | 452.739 |
| 2 | 600-lossless.jb2 | 414.743 |
| 3 | 200-3-45-stripe.jb2 | 308.256 |
| 4 | 600-10-45.jb2 | 237.540 |
| 5 | 200-4-45-stripe.jb2 | 228.703 |
| 6 | 042_23.jb2 | 192.639 |
| 7 | 042_21.jb2 | 185.060 |
| 8 | 042_22.jb2 | 150.655 |
| 9 | 200-5-45-stripe.jb2 | 148.264 |
| 10 | 042_24.jb2 | 137.894 |
| 11 | 200-6-45-stripe.jb2 | 134.007 |
| 12 | 200-3-45.jb2 | 122.820 |
| 13 | 600-6-0.jb2 | 122.115 |
| 14 | 042_7.jb2 | 120.497 |
| 15 | 600-20-45.jb2 | 106.423 |

## Per-File Top Hotspot (Excludes total_decode)

| Rank | File | Top Label | Total ms |
| --- | --- | --- | --- |
| 1 | 600-6-45.jb2 | immediate_halftone_region | 225.061 |
| 2 | 600-lossless.jb2 | immediate_generic_region | 206.059 |
| 3 | 200-3-45-stripe.jb2 | immediate_halftone_region | 153.694 |
| 4 | 600-10-45.jb2 | immediate_halftone_region | 117.169 |
| 5 | 200-4-45-stripe.jb2 | immediate_halftone_region | 113.858 |
| 6 | 042_23.jb2 | immediate_generic_refinement_region | 93.039 |
| 7 | 042_21.jb2 | immediate_generic_refinement_region | 88.925 |
| 8 | 200-5-45-stripe.jb2 | immediate_halftone_region | 73.636 |
| 9 | 042_22.jb2 | immediate_generic_refinement_region | 72.333 |
| 10 | 200-6-45-stripe.jb2 | immediate_halftone_region | 65.905 |
| 11 | 042_24.jb2 | immediate_generic_refinement_region | 65.299 |
| 12 | 200-3-45.jb2 | immediate_halftone_region | 61.257 |
| 13 | 042_7.jb2 | immediate_generic_region | 60.099 |
| 14 | 600-6-0.jb2 | immediate_halftone_region | 59.900 |
| 15 | 600-20-45.jb2 | immediate_halftone_region | 51.177 |

## Notes

- Avoiding skip bitmap creation when the grid is fully inside the region reduced immediate_halftone_region (1586.048 ms -> 1482.086 ms).
- immediate_generic_region stayed roughly flat (524.150 ms -> 521.717 ms).
- immediate_generic_refinement_region held steady (321.166 ms -> 319.595 ms).
