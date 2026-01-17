# JBIG2 Decode Profiling Report

Profiling run across UBC fixtures using `cargo run --example decode_file --release --profile`.

## Slowest Decoder Sections (Aggregated)

| Rank | Label | Total ms | Calls | Avg ms |
| --- | --- | --- | --- | --- |
| 1 | immediate_halftone_region | 1944.540 | 160 | 12.153 |
| 2 | immediate_generic_region | 566.854 | 21 | 26.993 |
| 3 | immediate_generic_refinement_region | 304.009 | 4 | 76.002 |
| 4 | immediate_text_region | 226.913 | 10 | 22.691 |
| 5 | symbol_dictionary | 68.399 | 18 | 3.800 |
| 6 | pattern_dictionary | 17.175 | 40 | 0.429 |
| 7 | finalize_current_page | 16.594 | 67 | 0.248 |
| 8 | intermediate_text_region | 6.505 | 4 | 1.626 |
| 9 | end_of_stripe | 3.535 | 145 | 0.024 |
| 10 | page_information | 0.572 | 67 | 0.009 |
| 11 | read_segments | 0.453 | 67 | 0.007 |

## Slowest Files (Total Decode)

| Rank | File | Total ms |
| --- | --- | --- |
| 1 | 600-6-45.jb2 | 517.121 |
| 2 | 600-lossless.jb2 | 458.442 |
| 3 | 200-3-45-stripe.jb2 | 336.349 |
| 4 | 600-10-45.jb2 | 307.939 |
| 5 | 600-6-0.jb2 | 260.968 |
| 6 | 200-4-45-stripe.jb2 | 252.500 |
| 7 | 042_23.jb2 | 172.118 |
| 8 | 042_21.jb2 | 170.446 |
| 9 | 600-10-0.jb2 | 158.265 |
| 10 | 600-20-45.jb2 | 156.531 |
| 11 | 200-5-45-stripe.jb2 | 150.793 |
| 12 | 200-6-45-stripe.jb2 | 146.452 |
| 13 | 042_22.jb2 | 144.236 |
| 14 | 042_24.jb2 | 143.566 |
| 15 | 200-3-45.jb2 | 132.536 |

## Per-File Top Hotspot (Excludes total_decode)

| Rank | File | Top Label | Total ms |
| --- | --- | --- | --- |
| 1 | 600-6-45.jb2 | immediate_halftone_region | 257.414 |
| 2 | 600-lossless.jb2 | immediate_generic_region | 227.979 |
| 3 | 200-3-45-stripe.jb2 | immediate_halftone_region | 167.809 |
| 4 | 600-10-45.jb2 | immediate_halftone_region | 152.902 |
| 5 | 600-6-0.jb2 | immediate_halftone_region | 128.668 |
| 6 | 200-4-45-stripe.jb2 | immediate_halftone_region | 125.712 |
| 7 | 042_23.jb2 | immediate_generic_refinement_region | 83.339 |
| 8 | 042_21.jb2 | immediate_generic_refinement_region | 82.406 |
| 9 | 600-10-0.jb2 | immediate_halftone_region | 77.713 |
| 10 | 600-20-45.jb2 | immediate_halftone_region | 75.988 |
| 11 | 200-5-45-stripe.jb2 | immediate_halftone_region | 74.997 |
| 12 | 200-6-45-stripe.jb2 | immediate_halftone_region | 72.670 |
| 13 | 042_22.jb2 | immediate_generic_refinement_region | 69.264 |
| 14 | 042_24.jb2 | immediate_generic_refinement_region | 69.000 |
| 15 | 200-3-45.jb2 | immediate_halftone_region | 66.131 |
