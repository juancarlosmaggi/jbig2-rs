# JBIG2-RS Testing Plan

This document outlines a comprehensive testing strategy for the jbig2-rs JBIG2 decoder library.

## ✅ Completed Tasks
- [x] Created TESTING_PLAN.md with comprehensive testing roadmap
- [x] Added unit tests for decode_generic.rs (7/7 passing - coding templates, parameter validation)
- [x] Added unit tests for decode_mmr.rs (7/7 passing - MMR decoding, dimension handling, error cases)
- [x] Added unit tests for decode_text.rs (9/9 passing - text region validation, parameter checking)
- [x] Added unit tests for decode_symbol.rs (4/4 passing - symbol dictionary validation, parameter checking)
- [x] Added unit tests for decode_halftone.rs (9/9 passing - halftone pattern generation and parameters)
- [x] Added unit tests for decode_pattern.rs (9/9 passing - pattern dictionary decoding and reuse)
- [x] Added unit tests for decode_refinement.rs (10/10 passing - refinement region decoding and quality improvements)
- [x] Added integration test resources (4 JBIG2/PNG pairs for different weight/compression settings)

## 🐛 Known Issues & Plan
- **Integration test JBIG2 files**: 3/4 test files have symbol dictionary parsing issues
- **Root cause**: Test files contain invalid/corrupted symbol count data
- **Impact**: Integration framework works but needs valid JBIG2 test files
- **Plan**: Regenerate JBIG2 test files with proper symbol dictionary data

## Current Testing Status
- ✅ Basic unit tests in `lib.rs` (document creation, header validation, bitmap operations, Huffman tables, segment parsing)
- ✅ **Phase 1 Complete**: Comprehensive decode module unit tests (63/63 tests passing)
- ✅ Parameter validation and error handling thoroughly tested
- ✅ Template and configuration testing complete
- ✅ Arithmetic decoder initialization fixes for refinement and text region tests
- ✅ **Phase 3 Integration Testing**: Framework implemented and working with PNG comparison
- ⚠️ Integration test files need valid JBIG2 data regeneration (current files have parsing issues)

## Testing Strategy Overview

### Phase 1: Unit Tests for Decode Modules (Priority: High)
Add comprehensive unit tests for each decode module using mock data and controlled inputs.

- [x] **decode_generic.rs**: Test coding templates, bitmap decoding, adaptive templates, context reuse
- [x] **decode_mmr.rs**: Test MMR decoding with known bit patterns, different dimensions, error handling
- [x] **decode_text.rs**: Test text region decoding, symbol instances, combination operators
- [x] **decode_symbol.rs**: Test symbol dictionary creation, different coding methods, refinement
- [x] **decode_halftone.rs**: Test halftone pattern generation and parameters
- [x] **decode_pattern.rs**: Test pattern dictionary decoding and reuse
- [x] **decode_refinement.rs**: Test refinement region decoding and quality improvements

### Phase 2: Test Fixtures and Data (Priority: High)
Create test fixtures with minimal valid JBIG2 files for reliable testing.

- [ ] Create `tests/fixtures/` directory
- [ ] Generate minimal valid JBIG2 files for each segment type
- [ ] Include test images from JBIG2 specification
- [ ] Add sample files from PDF.js test suite

### Phase 3: Integration and Error Testing (Priority: High)
Add comprehensive integration tests and error boundary testing.

- [x] **Integration test resources**: Added 4 JBIG2/PNG test pairs for different weight/compression settings (weight_no_jbig2, weight_t085_w025, weight_t085_w050, weight_t085_w075)
- [x] **Integration test framework**: Implemented complete test framework with PNG comparison and error handling
- [x] **Integration tests**: Framework working; 1/4 test files parse successfully, others have symbol dictionary issues requiring file regeneration
- [ ] **Error handling**: Invalid headers, corrupted data, boundary conditions, memory limits
- [ ] **API testing**: Jbig2Document/Jbig2Image APIs, chunked parsing, page access

### Phase 4: Advanced Testing (Priority: Medium)
Implement fuzz testing and property-based testing for robustness.

- [ ] **Fuzz testing**: Use cargo-fuzz to test parser functions with random input
- [ ] **Property-based testing**: Use proptest for generating test cases and testing invariants

### Phase 5: Performance and Validation (Priority: Low)
Add performance benchmarks and cross-compatibility validation.

- [ ] **Performance benchmarks**: Use criterion for decode speed and memory usage testing
- [ ] **Cross-validation**: Compare output with PDF.js JBIG2 decoder and specification examples

## Testing Infrastructure

### Dependencies to Add
```toml
[dev-dependencies]
rstest = "0.18"  # Already present
proptest = "1.0"
criterion = "0.5"
```

### Directory Structure
```
tests/
├── fixtures/           # JBIG2 test files
│   ├── minimal/        # Minimal valid files
│   ├── spec/          # Specification examples
│   └── pdfjs/         # PDF.js test suite files
├── decode_generic_test.rs
├── decode_mmr_test.rs
├── decode_text_test.rs
├── decode_symbol_test.rs
├── decode_halftone_test.rs
├── decode_pattern_test.rs
├── decode_refinement_test.rs
├── integration.rs      # Enhanced integration tests
├── error_handling.rs   # Error boundary tests
└── fuzz/              # Fuzz targets
```

## Implementation Notes

- Use rstest for parameterized tests
- Create helper functions for generating test data
- Focus on edge cases and error conditions
- Ensure tests are deterministic and reproducible
- Add documentation for complex test scenarios

## Success Criteria

- Test coverage >90%
- All major JBIG2 features tested
- Error conditions properly handled
- Performance benchmarks established
- Cross-compatibility with reference implementations

## Progress Tracking

This plan will be updated as tasks are completed. Each phase builds upon the previous one, ensuring a solid testing foundation.