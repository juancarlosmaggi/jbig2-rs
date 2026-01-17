# JBIG2 Decode Profiling Report

Profiling run across UBC fixtures using the release example binary (`cargo build --example decode_file --release`).

Averaged over 5 run(s).

## Slowest Decoder Sections (Aggregated)

| Rank | Label | Total ms | Calls | Avg ms |
| --- | --- | --- | --- | --- |
| 1 | immediate_halftone_region | 719.246 | 160 | 4.495 |
| 2 | immediate_generic_region | 493.591 | 21 | 23.504 |
| 3 | immediate_generic_refinement_region | 199.874 | 4 | 49.969 |
| 4 | immediate_text_region | 119.106 | 10 | 11.911 |
| 5 | symbol_dictionary | 61.719 | 18 | 3.429 |
| 6 | finalize_current_page | 16.444 | 67 | 0.245 |
| 7 | pattern_dictionary | 15.059 | 40 | 0.376 |
| 8 | intermediate_text_region | 7.151 | 4 | 1.788 |
| 9 | end_of_stripe | 3.080 | 145 | 0.021 |
| 10 | page_information | 0.572 | 67 | 0.009 |
| 11 | read_segments | 0.442 | 67 | 0.007 |

## Slowest Files (Total Decode)

| Rank | File | Total ms |
| --- | --- | --- |
| 1 | 600-lossless.jb2 | 422.970 |
| 2 | 600-6-45.jb2 | 214.153 |
| 3 | 200-3-45-stripe.jb2 | 136.992 |
| 4 | 042_7.jb2 | 115.301 |
| 5 | 042_23.jb2 | 114.308 |
| 6 | 600-10-45.jb2 | 114.009 |
| 7 | 042_21.jb2 | 112.934 |
| 8 | 042_24.jb2 | 100.951 |
| 9 | 042_22.jb2 | 97.694 |
| 10 | 200-4-45-stripe.jb2 | 88.614 |
| 11 | 600-6-0.jb2 | 87.007 |
| 12 | 042_5.jb2 | 72.322 |
| 13 | 042_25.jb2 | 63.348 |
| 14 | 200-5-45-stripe.jb2 | 60.744 |
| 15 | 200-3-45.jb2 | 59.838 |

## Per-File Top Hotspot (Excludes total_decode)

| Rank | File | Top Label | Total ms |
| --- | --- | --- | --- |
| 1 | 600-lossless.jb2 | immediate_generic_region | 210.230 |
| 2 | 600-6-45.jb2 | immediate_halftone_region | 105.919 |
| 3 | 200-3-45-stripe.jb2 | immediate_halftone_region | 68.117 |
| 4 | 042_7.jb2 | immediate_generic_region | 57.506 |
| 5 | 600-10-45.jb2 | immediate_halftone_region | 55.639 |
| 6 | 042_23.jb2 | immediate_generic_refinement_region | 53.784 |
| 7 | 042_21.jb2 | immediate_generic_refinement_region | 53.394 |
| 8 | 042_24.jb2 | immediate_generic_refinement_region | 46.852 |
| 9 | 042_22.jb2 | immediate_generic_refinement_region | 45.844 |
| 10 | 200-4-45-stripe.jb2 | immediate_halftone_region | 43.902 |
| 11 | 600-6-0.jb2 | immediate_halftone_region | 42.390 |
| 12 | 042_5.jb2 | immediate_generic_region | 36.035 |
| 13 | 200-5-45-stripe.jb2 | immediate_halftone_region | 29.901 |
| 14 | 200-3-45.jb2 | immediate_halftone_region | 29.786 |
| 15 | 200-6-45-stripe.jb2 | immediate_halftone_region | 28.792 |

## Notes

- Aggregate changes vs prior sweep: `immediate_halftone_region` 745.665 -> 719.246 ms, `immediate_generic_region` 476.666 -> 493.591 ms, `immediate_generic_refinement_region` 196.798 -> 199.874 ms, `finalize_current_page` 17.558 -> 16.444 ms, `pattern_dictionary` 15.371 -> 15.059 ms.
- File-level improvements: `200-3-45-stripe.jb2` 145.143 -> 136.992 ms, `200-4-45-stripe.jb2` 98.033 -> 88.614 ms, and `600-10-45.jb2` 123.309 -> 114.009 ms. Regressions: `600-lossless.jb2` 410.821 -> 422.970 ms, `042_7.jb2` 104.653 -> 115.301 ms, and `042_22.jb2` 86.879 -> 97.694 ms.
- Per-file hotspot shifts: `200-3-45-stripe.jb2` halftone 72.152 -> 68.117 ms (improved), `600-10-45.jb2` halftone 60.402 -> 55.639 ms (improved), `042_7.jb2` generic 52.205 -> 57.506 ms (regressed).
- Ranking changes: `200-3-45.jb2` entered the slowest-files list while `600-20-45.jb2` dropped out.
