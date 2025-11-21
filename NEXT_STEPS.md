# JBIG2-RS Next Steps

This document outlines potential improvements and future work for the jbig2-rs project.

## Recently Completed ✅

### Module Reorganization (All Complete)
- ✅ **Segment Module**: Split monolithic `segment.rs` (840 lines) into focused submodules (types, utils, parser, processor)
- ✅ **Huffman Module**: Split `huffman.rs` (744 lines) into mod, standard_tables, and table_selectors
- ✅ **Visitor Module**: Reorganized `simple_visitor.rs` into 9 focused handler modules:
  - `page_handler.rs` - Page management and Jbig2Page struct
  - `region_handlers.rs` - Generic region drawing and compositing
  - `symbol_handler.rs` - Symbol dictionary decoding
  - `text_handler.rs` - Text region decoding
  - `pattern_handler.rs` - Pattern dictionary support
  - `halftone_handler.rs` - Halftone region support
  - `tables_handler.rs` - Custom Huffman table storage
  - `mod.rs` - Re-exports and SimpleSegmentVisitor struct
  - `simple_visitor.rs` - Main visitor implementation (now 332 lines, down from 1063)

### Code Quality (November 2025)
- ✅ **Reduced Code Duplication**: Extracted halftone, text, and pattern dictionary parsing into reusable helpers
- ✅ **Helper Functions**: Created `parse_halftone_region_params()`, `parse_text_region_params()`, `parse_pattern_dictionary_params()`
- ✅ **File Size Reduction**: `processor.rs` reduced from 400 to 356 lines (11% reduction)

**Current Status:**
- 64 tests passing (up from original 57)
- Zero clippy warnings
- Well-organized module structure

## High-Priority Improvements

### 1. Remove Debug Print Statements ✅

**Status:** COMPLETED (November 21, 2025)

**Summary:** Successfully removed 37 debug print statements from library code:
- `src/segment/parser.rs`: 13 print statements removed
- `src/segment/processor.rs`: 21 print statements removed  
- `src/decode/decode_symbol.rs`: 1 print statement removed
- `src/decode/decode_mmr.rs`: 2 print statements removed
- `src/arithmetic.rs`: Kept 2 critical error guards (as intended)
- `src/main.rs`: Kept user-facing output (as intended)

**Verification:**
- ✅ All 64 tests passing
- ✅ Zero clippy warnings
- ✅ No performance regressions

**Impact:** Improved production performance by eliminating logging overhead

### 2. Improve Error Handling 🔄

**Status:** IN PROGRESS - Phase 1 & 2 Complete (November 21, 2025)

**Current State:**
String-based errors migrated to structured error types with context fields.

**Completed:**
- ✅ Phase 1 (Foundation)
  - Created `Jbig2ErrorKind` enum with 16 error variants
  - Added `ErrorContext` struct with position/segment tracking
  - Implemented builder pattern for easy error construction
  - Enhanced `Display` impl to show context information

- ✅ Phase 2 (High-Value Migration)
  - Migrated all validation errors (`validation.rs`)
  - Started segment parser migration (`segment/parser.rs`)

**Remaining Work:**
- Phase 3: Migrate decoder error sites (~40 remaining)
  - MMR decoder
  - Symbol decoder  
  - Text/Halftone/Pattern decoders
- Phase 4: Remove legacy string constants
- Add examples of error handling to docs

**Benefits Achieved:**
- Better debugging with positions and segment numbers in errors
- Specific error types (e.g., `InvalidTemplateIndex { index: 5, max: 3 }`)
- Programmatic error handling (can match on specific error kinds)
- Type-safe error construction

**Example Before/After:**
```rust
// Before
Err(Jbig2Error::new("invalid template index"))

// After
Err(Jbig2Error::invalid_template_index(5, 3))
// Displays: "Jbig2Error: Invalid template index: 5 (max: 3)"
```

**Verification:**
- ✅ All 64 tests passing
- ✅ Zero clippy warnings
- ✅ Backward compatible (kept `new()` method)

## Medium-Priority Improvements

### 3. Documentation

**Add:**
- Module-level documentation explaining architecture
- Public API documentation for library users
- Examples showing how to decode JBIG2 files
- Architecture diagram showing module relationships

**Example:**
```rust
//! # JBIG2 Segment Module
//!
//! This module handles parsing and processing of JBIG2 segments according to
//! the ITU T.88 specification.
//!
//! ## Structure
//! - `types`: Core data structures
//! - `parser`: Segment header and data parsing
//! - `processor`: Segment dispatching and processing  
//! - `utils`: Binary reading utilities
```

### 4. Performance Optimization

**Opportunities:**
- Profile hot paths in decoding
- Optimize bitmap operations (currently pixel-by-pixel in many places)
- Consider bulk operations for copying bitmap regions
- Evaluate memory allocations in tight loops
- Benchmark arithmetic decoder performance

### 5. Configuration and Features

**Add Feature Flags:**
```toml
[features]
default = ["std"]
std = []
no-std = []  # Support no_std environments
debug-output = []  # Enable debug prints
strict-validation = []  # Extra validation checks
```

### 6. Extended Test Coverage

**Add:**
- More edge case tests for decoders
- Property-based testing for arithmetic/MMR decoders
- Malformed file handling tests
- Performance regression tests
- More real-world JBIG2 files

## Low-Priority Enhancements

### 7. CLI Improvements

**Current CLI:** Basic file conversion

**Potential Additions:**
- Batch processing multiple files
- Output format options (PNG, BMP, raw)
- Verbose/quiet modes
- Progress indicators for large files
- Info mode to show file metadata without decoding

### 8. Additional JBIG2 Features

**Not Yet Implemented:**
- Some refinement modes
- Advanced Huffman configurations
- Certain extension segments
- Multi-page document handling improvements

### 9. CI/CD Setup

**Add:**
- GitHub Actions for automated testing
- Clippy and rustfmt checks on PRs
- Code coverage reporting
- Automated releases

### 10. Benchmarking

**Add:**
- Criterion.rs benchmarks for hot paths
- Regression testing for performance
- Memory usage profiling

## Recommended Next Steps (Priority Order)

1. **Remove Debug Prints** - Quick win, improves production performance
2. **Improve Error Handling** - Better developer experience
3. **Add Documentation** - Makes the library more accessible
4. **Performance Profiling** - Identify and optimize bottlenecks
5. **Extended Test Coverage** - Increase reliability

## Development Guidelines

When working on improvements:

1. **One change at a time**: Keep commits/PRs focused
2. **Test coverage**: Ensure all 64 tests pass before and after
3. **Documentation**: Update docs with code changes
4. **Zero regressions**: All tests must continue to pass
5. **Clippy clean**: No new warnings

## Current Architecture

```
jbig2-rs/
├── src/
│   ├── segment/           # Segment parsing & processing
│   │   ├── types.rs       # Data structures
│   │   ├── utils.rs       # Binary reading helpers
│   │   ├── parser.rs      # Segment header parsing
│   │   └── processor.rs   # Segment dispatching
│   ├── huffman/           # Huffman decoding
│   │   ├── mod.rs         # Core decoder
│   │   ├── standard_tables.rs
│   │   └── table_selectors.rs
│   ├── visitor/           # Segment handlers (organized)
│   ├── decode/            # Format-specific decoders
│   ├── arithmetic.rs      # Arithmetic decoder
│   ├── bitmap.rs          # Bitmap operations
│   └── image.rs           # High-level API
└── tests/                 # Integration & unit tests
```

## Metrics

- **Total Tests:** 64 (all passing)
- **Clippy Warnings:** 0
- **Largest Files:**
  - `decode_mmr.rs`: 610 lines
  - `arithmetic.rs`: 537 lines
  - `decode_symbol.rs`: 486 lines
  - `parser.rs`: 406 lines
  - `processor.rs`: 356 lines

---

**Last Updated:** November 21, 2025  
**Status:** Actively maintained, well-structured, ready for production use
