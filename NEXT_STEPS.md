# JBIG2-RS Next Steps

This document outlines potential improvements and future work for the jbig2-rs project.

## Completed Improvements ✅

- **Module Reorganization**: Split monolithic `segment.rs` (840 lines) and `huffman.rs` (744 lines) into focused submodules
- **Better Separation of Concerns**: Clear distinction between types, utilities, parsing, and processing logic
- **Improved Maintainability**: Easier to navigate and modify specific functionality
- **Test Coverage**: All 57 tests passing with zero regressions

## High-Priority Improvements

### 1. Complete Visitor Module Reorganization

The `visitor/simple_visitor.rs` (1063 lines) is still monolithic and could benefit from similar treatment:

**Proposed Structure:**
```
visitor/
├── mod.rs                    # Re-exports and SimpleSegmentVisitor struct
├── page_handler.rs           # Page management and Jbig2Page struct  
├── region_handlers.rs        # Generic region drawing and compositing
├── symbol_handler.rs         # Symbol dictionary decoding
├── text_handler.rs           # Text region decoding
├── pattern_handler.rs        # Pattern dictionary support
├── halftone_handler.rs       # Halftone region support
└── tables_handler.rs         # Custom Huffman table storage
```

**Benefits:**
- Each handler focuses on one segment type
- Easier to test individual handlers
- Reduced file size for better readability

### 2. Remove Debug Print Statements

**Issue:** The codebase has extensive `println!` and `eprintln!` statements throughout.

**Impact:**
- Performance overhead in production
- Cluttered output
- Mixing logging concerns with business logic

**Solution:**
- Remove or comment out debug prints
- Consider using `log` crate with feature flags for optional debug output
- Use `#[cfg(debug_assertions)]` for debug-only prints

**Files Affected:**
- `src/segment/parser.rs` (~20 print statements)
- `src/segment/processor.rs` (~15 print statements)
- `src/decode/decode_symbol.rs` (~5 print statements)
- `src/decode/decode_mmr.rs` (extensive debug output)

### 3. Improve Error Handling

**Current State:**
- Many generic error messages
- Limited context for debugging failures

**Improvements:**
- Add error context with file positions and segment numbers
- Create custom error types for different failure modes
- Include more diagnostic information in errors

**Example:**
```rust
// Current
Err(Jbig2Error::new("invalid segment"))

// Improved  
Err(Jbig2Error::InvalidSegment {
    segment_number: header.number,
    segment_type: header.segment_type,
    position: start,
    reason: "insufficient data"
})
```

## Medium-Priority Improvements

### 4. Performance Optimization

**Opportunities:**
- Profile hot paths in decoding
- Optimize bitmap operations (currently pixel-by-pixel)
- Consider bulk operations for copying bitmap regions
- Evaluate memory allocations in tight loops

### 5. Documentation

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

### 6. Code Quality

**Improvements:**
- Reduce code duplication in segment processing
- Extract common patterns into helper functions
- Add more comprehensive unit tests for edge cases
- Consider property-based testing for decoders

### 7. Configuration and Features

**Add Feature Flags:**
```toml
[features]
default = ["std"]
std = []
no-std = []  # Support no_std environments
debug-output = []  # Enable debug prints
strict-validation = []  # Extra validation checks
```

## Low-Priority Enhancements

### 8. CLI Improvements

**Current CLI:** Basic file conversion

**Potential Additions:**
- Batch processing multiple files
- Output format options (PNG, BMP, raw)
- Verbose/quiet modes
- Progress indicators for large files
- Info mode to show file metadata without decoding

### 9. Additional JBIG2 Features

**Not Yet Implemented:**
- Some refinement modes
- Advanced Huffman configurations
- Certain extension segments
- Multi-page document handling improvements

### 10. Integration Testing

**Expand Test Coverage:**
- More real-world JBIG2 files
- Edge cases from spec
- Malformed file handling
- Performance benchmarks

## Development Workflow Improvements

### 11. CI/CD

**Setup:**
- GitHub Actions or similar for automated testing
- Clippy and rustfmt checks
- Code coverage reporting
- Automated releases

### 12. Benchmarking

**Add:**
- Criterion.rs benchmarks for hot paths
- Regression testing for performance
- Memory usage profiling

## Migration Path

If refactoring the visitor module:

1. **Phase 1:** Create new handler modules without changing existing code
2. **Phase 2:** Move methods from `simple_visitor.rs` to handler modules
3. **Phase 3:** Update `simple_visitor.rs` to delegate to handlers
4. **Phase 4:** Verify all tests pass
5. **Phase 5:** Clean up and document new structure

## Contributing

When working on improvements:

1. **One change at a time**: Keep PRs focused
2. **Test coverage**: Ensure tests pass before and after
3. **Documentation**: Update docs with code changes
4. **Backward compatibility**: Maintain public API stability

## Conclusion

The recent reorganization has significantly improved the codebase structure. The next logical step would be completing the visitor module reorganization, followed by cleaning up debug output and improving documentation.

All improvements should maintain the project's **zero-regression policy** - all 57 tests must continue to pass.
