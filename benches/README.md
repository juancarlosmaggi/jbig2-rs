# Benchmarking Guide

This directory contains benchmarks for the `jbig2-rs` library, designed to measure performance and detect regressions.

## Benchmark Suites

### 1. `bitmap_bench.rs` - Bitmap Operations
Tests performance of core bitmap operations:
- **draw_symbol**: Symbol drawing using bitblt (byte-aligned combine)
- **combine**: Various bitmap combine operations (OR, AND, XOR, XNOR, REPLACE)

**Current Baselines:**
- `draw_symbol`: ~3.6 µs (optimized with byte-aligned bitblt)
- `combine`: ~500 ns - 1 µs depending on operation

### 2. `decoder_bench.rs` - Arithmetic Decoder
Tests arithmetic decoder micro-operations:
- **read_bit**: Single bit reading with context updates

**Current Baselines:**
- `read_bit`: ~6.5 µs (optimized with inline and unsafe)

### 3. `full_bench.rs` - End-to-End Decoding
Tests complete file decoding performance:
- **symbol_dictionary.jb2**: Complex file with symbol dictionary and text regions
- **text_region.jb2**: Text region with Huffman encoding

**Current Baselines:**
- `symbol_dictionary.jb2`: ~54 ms
- `text_region.jb2`: ~10 ms

## Running Benchmarks

### Basic Usage

Run all benchmarks:
```bash
cargo bench
```

Run a specific benchmark suite:
```bash
cargo bench --bench bitmap_bench
cargo bench --bench decoder_bench
cargo bench --bench full_bench
```

Run a specific test within a suite:
```bash
cargo bench --bench bitmap_bench draw_symbol
```

### Baseline Comparison Workflow

Criterion supports saving and comparing against baselines to detect regressions.

#### 1. Establish a Baseline

Before making changes, save the current performance as a baseline:
```bash
cargo bench -- --save-baseline main
```

This creates a baseline named "main" with the current performance metrics.

#### 2. Make Your Changes

Edit code, apply optimizations, refactor, etc.

#### 3. Compare Against Baseline

After changes, compare performance to the baseline:
```bash
cargo bench -- --baseline main
```

Criterion will show:
- **Change**: Percentage improvement or regression
- **Color coding**: Green for improvements, red for regressions
- **Statistical significance**: Whether the change is statistically significant

#### 4. Analyze Results

Look in `target/criterion/` for detailed HTML reports with:
- Performance charts
- Statistical analysis
- Comparison graphs

## Automated Regression Detection

Use the provided script to automatically fail builds on significant regressions:

```bash
./scripts/check_performance.sh
```

**Threshold:** The script fails if any benchmark regresses by more than **10%**.

### Integration Points

#### Local Development
Run before committing performance-sensitive changes:
```bash
# Establish baseline
cargo bench -- --save-baseline main

# Make changes...

# Check for regressions
./scripts/check_performance.sh
```

#### Git Hooks (Optional)
Add to `.git/hooks/pre-commit`:
```bash
#!/bin/bash
./scripts/check_performance.sh || {
  echo "Performance regression detected!"
  exit 1
}
```

#### CI/CD (Optional)
Add to your CI pipeline:
```yaml
- name: Run benchmarks
  run: |
    cargo bench -- --save-baseline main
    ./scripts/check_performance.sh
```

## Performance History

### November 2025 - Optimization Phase

#### Bitmap Optimization
- **Before**: ~29.2 µs for draw_symbol
- **After**: ~3.6 µs for draw_symbol
- **Speedup**: ~8x improvement
- **Method**: Implemented byte-aligned bitblt operation

#### Arithmetic Decoder Optimization
- **Before**: ~7.0 µs for read_bit
- **After**: ~6.5 µs for read_bit
- **Speedup**: ~5% improvement
- **Method**: Added `#[inline(always)]` and `unsafe` unchecked access

#### Memory Optimization
- Pre-allocated vectors in `decode_symbol.rs` and `processor.rs`
- Used `Vec::with_capacity` for known sizes
- Reduced unnecessary reallocations

## Interpreting Results

### Criterion Output

```
draw_symbol             time:   [3.5901 µs 3.6127 µs 3.6376 µs]
                        change: [-0.8234% +0.3421% +1.5438%] (p = 0.56 > 0.05)
                        No change in performance detected.
```

- **time**: Median and confidence interval (95%)
- **change**: Performance change vs. baseline
- **p-value**: Statistical significance (p < 0.05 = significant)

### What to Look For

- **Regressions > 10%**: Investigate immediately
- **Regressions 5-10%**: Review and document if acceptable
- **Regressions < 5%**: May be noise, verify with multiple runs
- **Improvements**: Validate correctness with tests

## Best Practices

1. **Warm-up**: Criterion automatically warms up before measuring
2. **Multiple runs**: Each benchmark runs many iterations for statistical validity
3. **Stable environment**: Close other applications during benchmarking
4. **Reproducibility**: Run on the same hardware for meaningful comparisons
5. **Document changes**: Note any performance changes in commit messages

## Troubleshooting

### High Variance
If results vary significantly between runs:
- Close background applications
- Ensure system isn't under load
- Check for thermal throttling
- Run benchmarks multiple times

### Baseline Not Found
If you see "baseline not found" errors:
```bash
# Create a baseline first
cargo bench -- --save-baseline main
```

### Comparing Different Machines
Baselines are machine-specific. Don't compare:
- Different CPU architectures
- Different CPU generations
- Different optimization levels

## Additional Resources

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [JBIG2 Specification](https://www.itu.int/rec/T-REC-T.88)

## Contributing

When adding new benchmarks:
1. Place in appropriate benchmark suite file
2. Document expected baseline performance
3. Update this README with new benchmark details
4. Run `cargo bench` to verify benchmark works
5. Establish baseline: `cargo bench -- --save-baseline main`
