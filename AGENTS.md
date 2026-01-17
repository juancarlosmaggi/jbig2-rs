# AGENTS.md

## Current Goal
Profile `jbig2-rs` decoding, identify the slowest paths, and optimize without
breaking tests or output correctness.

## Profiling Workflow
- Use `examples/decode_file.rs` with `--profile` to decode fixtures and print timing.
- Aggregate results into `PROFILE_REPORT.md` for hotspots and slow files.
  The batch runner prints a single-line progress bar with ETA.

Example single file:
```
cargo run --example decode_file --release --quiet -- --profile \
  tests/resources/ubc/600-6-45.jb2 /tmp/jbig2-profile/600-6-45.bin
```

Example UBC batch (preferred):
```
python3 scripts/profile_ubc.py
```

## Performance Targets
- Prioritize `immediate_halftone_region`, `immediate_generic_region`, and
  `finalize_current_page` hotspots.
- Validate changes with full test suite and UBC hash tests.

## Testing Requirements
- Always keep `cargo test` passing.
- Re-run `cargo test jbig2dec_hashes` after performance changes.
- If touching decode logic, validate at least one large UBC fixture profile run.

## Reporting
- Update `PROFILE_REPORT.md` after each profiling sweep.
- Note which files regress or improve and which labels move in the rankings.

## Current Findings / Status
- Profiling shows `immediate_halftone_region` dominates aggregate time across
  UBC fixtures, especially 600-dpi inputs.
- `finalize_current_page` and `immediate_generic_region` are secondary costs.

## Current Plan
1) Add finer-grained timers inside halftone decoding (MMR plane decode, pattern
   expansion, grid placement) to isolate the root cause.
2) Implement targeted optimizations and re-run profiling and tests.
