# JBIG2 Decoder in Rust

A pure Rust implementation of a JBIG2 decoder.

## Overview

This crate provides a permissive JBIG2 decoder written entirely in Rust. JBIG2 is a lossless/lossy compression standard for bi-level images, commonly used in document imaging workflows.

## Features

- Complete JBIG2 decoding, including arithmetic coding, Huffman coding, and MMR compression
- Multi-page document support
- Symbol dictionary, text region, halftone region, generic region, and refinement region decoding
- Standard and custom Huffman table handling
- Validation and structured error reporting
- Bounded embedding API that returns packed 1bpp page output

## Getting Started

Add this to your `Cargo.toml`:

```toml
[dependencies]
jbig2-rs = "0.1.0"
```

The default feature set builds the decoder core only. It does not pull CLI or
image-export dependencies.

### Decode One Page For Embedding

```rust
use jbig2_rs::{DecodeOptions, decode_page};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read("document.jb2")?;
    let page = decode_page(&data, None, DecodeOptions::default())?;

    println!("size: {}x{}", page.width, page.height);
    println!("stride: {} bytes", page.stride);
    println!("packed bitmap bytes: {}", page.data.len());

    Ok(())
}
```

For PDF-style streams with global dictionaries, pass global segment bytes as
the second argument:

```rust
use jbig2_rs::{DecodeOptions, decode_page};

fn decode_pdf_jbig2_stream(
    page_bytes: &[u8],
    global_bytes: &[u8],
) -> Result<(), jbig2_rs::Jbig2Error> {
    let page = decode_page(page_bytes, Some(global_bytes), DecodeOptions::default())?;
    println!("decoded {} bytes", page.data.len());
    Ok(())
}
```

### Decode A Full Document

```rust
use jbig2_rs::Jbig2Document;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read("document.jb2")?;
    let document = Jbig2Document::parse(&data)?;

    println!("pages: {}", document.page_count());
    Ok(())
}
```

### Decode Chunks

Use chunk decoding when the input arrives in multiple buffers.

```rust
use jbig2_rs::{Jbig2Chunk, Jbig2Document};

fn decode_chunks(global_data: Vec<u8>, page_data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let chunks = vec![
        Jbig2Chunk {
            data: global_data,
            start: 0,
            end: global_data.len(),
        },
        Jbig2Chunk {
            data: page_data,
            start: 0,
            end: page_data.len(),
        },
    ];

    let document = Jbig2Document::parse_chunks(&chunks)?;
    println!("pages: {}", document.page_count());
    Ok(())
}
```

## Output Format

- `decode_page(...).data` and `Jbig2Page::packed_bitmap()` return packed 1bpp bitmap bytes
- 8 pixels per byte, MSB-first
- `stride = (width + 7) / 8`
- bit value `1` means black/foreground and bit value `0` means white/background
- `Jbig2Page::to_image_data()` is a legacy helper that expands to 8-bit grayscale bytes

### Write a PNG

This example expands the packed 1bpp bitmap and writes a PNG file. Add `image` to your `Cargo.toml`:

```toml
[dependencies]
image = "0.25"
jbig2-rs = "0.1.0"
```

```rust
use image::{GrayImage, Luma};
use jbig2_rs::Jbig2Document;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read("document.jb2")?;
    let document = Jbig2Document::parse(&data)?;
    let page = document.get_page(0).ok_or("no pages")?;

    let width = page.page_info.width as u32;
    let height = page.page_info.height as u32;
    let packed = page.packed_bitmap();
    let stride = ((width as usize) + 7) / 8;

    let mut img = GrayImage::new(width, height);
    for y in 0..height as usize {
        for x in 0..width as usize {
            let byte = packed[y * stride + (x >> 3)];
            let bit = (byte >> (7 - (x & 7))) & 1;
            let gray = if bit == 1 { 0 } else { 255 };
            img.put_pixel(x as u32, y as u32, Luma([gray]));
        }
    }

    img.save("page_0.png")?;
    Ok(())
}
```

### Write a PBM (No Extra Dependencies)

PBM is a simple 1bpp format. You can write it directly from the packed bitmap:

```rust
use jbig2_rs::Jbig2Document;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read("document.jb2")?;
    let document = Jbig2Document::parse(&data)?;
    let page = document.get_page(0).ok_or("no pages")?;

    let width = page.page_info.width as usize;
    let height = page.page_info.height as usize;
    let packed = page.packed_bitmap();

    let mut out = Vec::new();
    out.extend_from_slice(b"P4\n");
    out.extend_from_slice(format!("{} {}\n", width, height).as_bytes());
    out.extend_from_slice(packed);

    fs::write("page_0.pbm", out)?;
    Ok(())
}
```

### Expand to 8-bit Grayscale

```rust
fn expand_to_grayscale(width: usize, height: usize, packed: &[u8]) -> Vec<u8> {
    let stride = (width + 7) / 8;
    let mut pixels = vec![0u8; width * height];

    for y in 0..height {
        for x in 0..width {
            let byte = packed[y * stride + (x >> 3)];
            let bit = (byte >> (7 - (x & 7))) & 1;
            pixels[y * width + x] = if bit == 1 { 0 } else { 255 };
        }
    }

    pixels
}
```

## Examples

```bash
# Decode a file and save raw bitmap output
cargo run --example decode_file -- input.jb2 output.bin

# Run chunk decoding example
cargo run --example decode_chunks
```

## CLI

This repository includes a CLI tool that decodes a JBIG2 file into PNG pages.

```bash
# Build the CLI binary
cargo build --release --features cli

# Decode to PNG files in the current directory
./target/release/jbig2-rs --input input.jb2

# Decode to PNG files in a specific directory with a custom prefix
./target/release/jbig2-rs --input input.jb2 --output-dir out --prefix doc
```

Flags:

- `--input`, `-i`: Input `.jb2` file path (required)
- `--output-dir`, `-o`: Output directory (default `.`)
- `--prefix`, `-p`: Output filename prefix (default is input file stem)

## Architecture

- `parser`: Segment parsing and processing
- `huffman`: Huffman decoding and table management
- `parser::visitor`: Segment visitor and page assembly
- `decoders`: Region and coding-mode decoders
- `document`: Document and page types
- `portable`: Embeddable packed-page API
- `arithmetic`: Arithmetic decoder implementation

## Feature Model

- Decoder core only: default, or `--no-default-features`; no normal dependencies.
- CLI: `--features cli`; enables `clap` and PNG export.
- Image export: `--features image-export`; enables the `image` crate with PNG support only.
- FFI: `--features ffi`; exports the stable C ABI in addition to the Rust API.
- Fuzzing: use the separate `fuzz/` crate and `cargo fuzz`.
- Benchmarks: Criterion benchmarks live under `benches/` and use dev-dependencies only.

Run `python3 scripts/check_portability.py` to print the decoder-core dependency
tree and fail if a prohibited or unknown license enters the default graph.

## Embedding API

`decode_page(page_bytes, global_bytes, options)` is the preferred API for
native/mobile consumers. It returns `DecodedPage` with:

- `data`: packed 1bpp bitmap bytes
- `width`, `height`, and `stride`
- `page_index` and `page_id`
- `polarity = BitmapPolarity::OneIsBlack`
- optional `DecodeProfile` when `DecodeOptions::with_profile(true)` is used

Errors expose stable codes through `Jbig2Error::code()` and
`Jbig2Error::code_name()`, plus a display message suitable for logs.

## Resource Limits

`DecodeOptions::default()` applies bounded defaults for untrusted input:

- input bytes: 128 MiB
- decoded pixels: 250,000,000
- pages: 256
- segments: 250,000
- retained symbol dictionary bytes: 128 MiB
- intermediate bitmap bytes: 128 MiB

Use `DecodeOptions::with_limits(DecodeLimits { ... })` to tune these budgets.
Set an individual limit to `None` to disable it. Long decodes can be aborted by
passing an `Arc<AtomicBool>` with `DecodeOptions::with_cancel_flag`; when the
flag is set, decode returns `Jbig2ErrorCode::Cancelled`.

## Native C ABI

Enable `--features ffi` to expose a stable C ABI for iOS, Android, and other
native callers. The ABI is renderer-agnostic and uses borrowed input buffers.
Output buffers and strings are owned by Rust and must be released with
`jbig2_ffi_result_free`.

Primary symbols:

- `jbig2_ffi_decode_options_default`
- `jbig2_ffi_decode_page`
- `jbig2_ffi_result_free`
- `jbig2_ffi_error_code_name`

The matching C header is `include/jbig2_rs.h`.
The crate builds `rlib`, `staticlib`, and `cdylib` artifacts. Use `staticlib`
for iOS packaging and `cdylib` for Android JNI/native packaging.

## Platform Builds

Supported Rust toolchain: stable Rust 1.85 or newer, because the crate uses
edition 2024.

Supported mobile target triples:

- iOS device: `aarch64-apple-ios`
- iOS simulator: `aarch64-apple-ios-sim`, `x86_64-apple-ios`
- Android: `aarch64-linux-android`, `armv7-linux-androideabi`,
  `x86_64-linux-android`, `i686-linux-android`

Install targets with `rustup target add <triple>`, then build:

```bash
cargo build --release --no-default-features --target aarch64-apple-ios
cargo build --release --no-default-features --features ffi --target aarch64-linux-android
```

Run `scripts/build_mobile_targets.sh` to smoke-test the Rust embedding tests,
the FFI tests, and decoder-core/FFI compilation for all supported mobile
triples installed in the local toolchain.

## Building

```bash
cargo build
```

## Testing

```bash
cargo test
```

## Performance & Profiling

### Batch Profiling
To aggregate profiling data across the UBC test fixtures and generate `PROFILE_REPORT.md`:
```bash
python3 scripts/profile_ubc.py
```

### Regression Testing
To compare current performance against a saved baseline (uses `criterion`):
```bash
# First establish a baseline
cargo bench -- --save-baseline main

# Then check for regressions
./scripts/check_performance.sh main
```

## Fuzzing

This project uses `cargo-fuzz` for robustness testing. Fuzz targets are located in the `fuzz/` directory.

To start fuzzing (requires `cargo-fuzz`):
```bash
cargo fuzz run fuzz_reader
```

## License

Licensed under MIT OR Apache-2.0.
