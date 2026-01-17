# JBIG2 Decode Profiling Report

Profiling run across UBC fixtures using `cargo run --example decode_file --release --profile`.

Averaged over 5 run(s).

## Slowest Decoder Sections (Aggregated)

| Rank | Label | Total ms | Calls | Avg ms |
| --- | --- | --- | --- | --- |
| 1 | immediate_halftone_region | 1859.575 | 160 | 11.622 |
| 2 | immediate_generic_region | 547.316 | 21 | 26.063 |
| 3 | immediate_generic_refinement_region | 309.061 | 4 | 77.265 |
| 4 | immediate_text_region | 220.276 | 10 | 22.028 |
| 5 | symbol_dictionary | 61.214 | 18 | 3.401 |
| 6 | finalize_current_page | 17.020 | 67 | 0.254 |
| 7 | pattern_dictionary | 16.218 | 40 | 0.405 |
| 8 | intermediate_text_region | 7.713 | 4 | 1.928 |
| 9 | end_of_stripe | 3.069 | 145 | 0.021 |
| 10 | page_information | 0.537 | 67 | 0.008 |
| 11 | read_segments | 0.481 | 67 | 0.007 |

## Slowest Files (Total Decode)

| Rank | File | Total ms |
| --- | --- | --- |
| 1 | 600-6-45.jb2 | 480.011 |
| 2 | 600-lossless.jb2 | 448.447 |
| 3 | 200-3-45-stripe.jb2 | 354.274 |
| 4 | 600-10-45.jb2 | 279.561 |
| 5 | 200-4-45-stripe.jb2 | 234.949 |
| 6 | 600-6-0.jb2 | 224.628 |
| 7 | 042_21.jb2 | 187.550 |
| 8 | 042_23.jb2 | 170.169 |
| 9 | 200-5-45-stripe.jb2 | 162.350 |
| 10 | 600-10-0.jb2 | 161.867 |
| 11 | 600-20-45.jb2 | 155.049 |
| 12 | 042_24.jb2 | 150.330 |
| 13 | 200-6-45-stripe.jb2 | 149.444 |
| 14 | 042_22.jb2 | 136.543 |
| 15 | 200-3-45.jb2 | 136.047 |

## Per-File Top Hotspot (Excludes total_decode)

| Rank | File | Top Label | Total ms |
| --- | --- | --- | --- |
| 1 | 600-6-45.jb2 | immediate_halftone_region | 238.660 |
| 2 | 600-lossless.jb2 | immediate_generic_region | 222.776 |
| 3 | 200-3-45-stripe.jb2 | immediate_halftone_region | 176.637 |
| 4 | 600-10-45.jb2 | immediate_halftone_region | 138.630 |
| 5 | 200-4-45-stripe.jb2 | immediate_halftone_region | 117.101 |
| 6 | 600-6-0.jb2 | immediate_halftone_region | 111.073 |
| 7 | 042_21.jb2 | immediate_generic_refinement_region | 90.348 |
| 8 | 042_23.jb2 | immediate_generic_refinement_region | 82.042 |
| 9 | 200-5-45-stripe.jb2 | immediate_halftone_region | 80.762 |
| 10 | 600-10-0.jb2 | immediate_halftone_region | 79.385 |
| 11 | 600-20-45.jb2 | immediate_halftone_region | 75.262 |
| 12 | 200-6-45-stripe.jb2 | immediate_halftone_region | 74.189 |
| 13 | 042_24.jb2 | immediate_generic_refinement_region | 71.530 |
| 14 | 200-3-45.jb2 | immediate_halftone_region | 67.912 |
| 15 | 042_22.jb2 | immediate_generic_refinement_region | 65.142 |
