# Fuzz Testing for jbig2-rs

This directory contains fuzz targets for testing jbig2-rs with random inputs using `cargo-fuzz` and `libFuzzer`.

## Prerequisites

**Fuzzing requires Rust nightly:**

```bash
# Install nightly toolchain
rustup install nightly

# Use nightly for this project
rustup override set nightly

# Or prefix commands with +nightly
cargo +nightly fuzz build
```

## Fuzz Targets

- **fuzz_reader** - Tests `Reader` operations (reading bits, bytes, positioning)
- **fuzz_arithmetic** - Tests `ArithmeticDecoder` with random data and contexts
- **fuzz_bitmap** - Tests `Bitmap` pixel operations and combine functions
- **fuzz_mmr** - Tests MMR decoder with random encoded data

## Running Fuzz Tests

### Quick Test (30 seconds each target)
```bash
cargo fuzz run fuzz_reader -- -max_total_time=30
cargo fuzz run fuzz_arithmetic -- -max_total_time=30
cargo fuzz run fuzz_bitmap -- -max_total_time=30
cargo fuzz run fuzz_mmr -- -max_total_time=30
```

### Extended Fuzzing
```bash
# Run for 5 minutes
cargo fuzz run fuzz_reader -- -max_total_time=300

# Run indefinitely (Ctrl+C to stop)
cargo fuzz run fuzz_reader
```

### With Multiple Jobs
```bash
# Use all CPU cores
cargo fuzz run fuzz_reader -- -jobs=$(nproc)
```

## Crash Investigation

If a crash is found, it will be saved in `fuzz/artifacts/<target_name>/`:

```bash
# Reproduce a crash
cargo fuzz run fuzz_reader fuzz/artifacts/fuzz_reader/crash-<hash>

# Debug with verbose output
cargo fuzz run fuzz_reader fuzz/artifacts/fuzz_reader/crash-<hash> -- -rss_limit_mb=0
```

## Coverage Reporting

```bash
# Generate coverage report
cargo fuzz coverage fuzz_reader
```

## Tips

- Start with short durations (30-60s) to verify targets work
- Use `-max_len=N` to limit input size for faster fuzzing
- Monitor CPU/memory usage, especially for bitmap/MMR targets
- Crashes are automatically saved for reproduction
