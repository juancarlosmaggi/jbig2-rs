# Portability Todo

Last updated: 2026-05-13

This checklist tracks the work needed to make `jbig2-rs` easy to embed in
downstream projects, including mobile apps, native PDF renderers, and other
non-CLI consumers.

The portability goal is:

- Keep the library permissively licensed and easy to audit.
- Keep the default crate graph small.
- Separate decoder-core functionality from CLI, image export, examples, and
  development tooling.
- Provide stable, bounded APIs suitable for native/mobile embedding.
- Support platform builds without requiring downstream projects to know the
  internal decoder architecture.

## Licensing And Metadata

- [ ] Add top-level license files so package registries, GitHub, and downstream
  tooling can detect the project license.
- [ ] Prefer `LICENSE-MIT` and `LICENSE-APACHE` to match the current
  `MIT OR Apache-2.0` manifest expression.
- [ ] Add repository metadata to `Cargo.toml`, including the GitHub URL.
- [ ] Add homepage/documentation metadata if useful for generated notices.
- [ ] Confirm published crate metadata reports `license = "MIT OR Apache-2.0"`.
- [ ] Add a short reusable notice entry for downstream open-source notices.

## Feature Graph

- [ ] Keep the default feature set limited to decoder-core functionality.
- [ ] Move `clap` behind a CLI-only feature or a separate binary crate.
- [ ] Move `image` behind an image-export feature or a separate example/CLI
  crate.
- [ ] Ensure library consumers can depend on `jbig2-rs` without pulling image
  encoders, AVIF/WebP/TIFF support, or CLI dependencies.
- [ ] Document the intended feature combinations:
  - [ ] decoder core only
  - [ ] CLI
  - [ ] image export
  - [ ] fuzzing
  - [ ] benchmarks
- [ ] Generate and inspect the dependency graph for the decoder-core-only build.
- [ ] Run a license audit on the decoder-core-only graph.
- [ ] Fail portability readiness if GPL, AGPL, LGPL-only, MPL, Commons Clause,
  non-commercial, custom source-available, or unknown licenses enter the
  decoder-core graph.

## Embeddable Decoder API

- [ ] Provide a stable high-level API that accepts JBIG2 page bytes and optional
  global bytes.
- [ ] Return packed 1bpp bitmap bytes.
- [ ] Return width, height, and row stride.
- [ ] Return selected page index or page id.
- [ ] Return structured error codes and messages.
- [ ] Return optional decode profile/timing metadata when profiling is enabled.
- [ ] Keep the embeddable API independent of PNG writing, filesystem access,
  CLI parsing, and image crate types.
- [ ] Add examples/tests for global segment plus page segment decoding.
- [ ] Document packed bitmap format: 8 pixels per byte, MSB-first, with
  `stride = (width + 7) / 8`.
- [ ] Document bitmap polarity and any conversion helper behavior.

## Resource Limits And Cancellation

- [ ] Add configurable max input byte limits.
- [ ] Add configurable max decoded pixel limits.
- [ ] Add configurable max page count limits.
- [ ] Add configurable max segment count limits.
- [ ] Add configurable max symbol dictionary memory limits.
- [ ] Add configurable max intermediate bitmap/allocation limits.
- [ ] Add cancellation or cooperative abort checkpoints for long decodes.
- [ ] Ensure over-budget inputs return structured errors, not panics.
- [ ] Add tests for malformed, truncated, and over-budget streams.
- [ ] Document default limits and how embedders should tune them.

## FFI And Native Embedding

- [ ] Decide which native embedding surfaces this project will maintain:
  - [ ] Rust library API only
  - [ ] stable C ABI
  - [ ] UniFFI bindings
  - [ ] generated headers
  - [ ] platform package helpers
- [ ] If a C ABI is added, define ownership rules for input buffers, output
  buffers, errors, and free functions.
- [ ] If UniFFI is added, define generated binding ownership and versioning.
- [ ] Keep bridge APIs independent from any specific app or renderer.
- [ ] Provide a small smoke-test program for the chosen FFI surface.
- [ ] Map Rust errors to stable numeric or string codes for non-Rust callers.
- [ ] Avoid extra copies where possible, but prefer stable ownership over
  premature zero-copy complexity.

## Platform Builds

- [ ] Document supported Rust toolchains.
- [ ] Document supported target triples.
- [ ] Add Android target build instructions if mobile embedding is supported.
- [ ] Add iOS simulator and device target build instructions if mobile
  embedding is supported.
- [ ] Add CI jobs or scripts for the supported target triples.
- [ ] Ensure the decoder-core build works without filesystem, CLI, or image
  export assumptions.
- [ ] Add a native smoke test that can run against a small fixture for each
  supported platform family.

## Fixtures And Correctness

- [ ] Add JBIG2 page segment fixtures without globals.
- [ ] Add JBIG2 fixtures with global segments.
- [ ] Add arithmetic-coded region fixtures.
- [ ] Add Huffman-coded region fixtures.
- [ ] Add MMR region fixtures.
- [ ] Add symbol dictionary and text region fixtures.
- [ ] Add malformed stream fixtures.
- [ ] Add truncated stream fixtures.
- [ ] Add over-budget stream fixtures.
- [ ] Add cancellation tests for long decodes.
- [ ] Add multi-page input behavior coverage.
- [ ] Compare supported fixtures against an external reference decoder where
  license and tooling constraints allow local test use.

## Documentation

- [ ] Document the crate feature model.
- [ ] Document embeddable API examples.
- [ ] Document resource limit configuration.
- [ ] Document error codes.
- [ ] Document packed bitmap output and polarity.
- [ ] Document native/FFI build instructions if maintained.
- [ ] Keep CLI documentation separate from library embedding documentation.

## Readiness Criteria

`jbig2-rs` is portable enough for downstream native embedding when:

- The default library feature graph is small and permissively licensed.
- CLI and image-export dependencies are optional.
- License files and package metadata are present.
- The embeddable API returns packed bitmap output plus dimensions, stride, and
  structured errors.
- Resource limits and cancellation/abort checkpoints exist for untrusted input.
- The chosen native embedding surface has documented ownership rules.
- Supported platform builds are documented and smoke-tested.
- Fixture coverage includes valid, malformed, truncated, over-budget, and
  multi-page cases.
