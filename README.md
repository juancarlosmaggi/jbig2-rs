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

## Getting Started

Add this to your `Cargo.toml`:

```toml
[dependencies]
jbig2-rs = "0.1.0"
```

### Decode a File

```rust
use jbig2_rs::Jbig2Document;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read("document.jb2")?;
    let document = Jbig2Document::parse(&data)?;

    println!("pages: {}", document.page_count());

    if let Some(page) = document.get_page(0) {
        println!("size: {}x{}", page.page_info.width, page.page_info.height);
        let bitmap = page.to_image_data();
        println!("bitmap bytes: {}", bitmap.len());
    }

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

- `page.to_image_data()` returns a packed 1bpp bitmap
- 8 pixels per byte, MSB-first
- `stride = (width + 7) / 8`

### Write a PNG

This example expands the packed 1bpp bitmap and writes a PNG file. Add `image` to your `Cargo.toml`:

```toml
[dependencies]
image = "0.24"
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
    let packed = page.to_image_data();
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
    let packed = page.to_image_data();

    let mut out = Vec::new();
    out.extend_from_slice(b"P4\n");
    out.extend_from_slice(format!(\"{} {}\\n\", width, height).as_bytes());
    out.extend_from_slice(&packed);

    fs::write(\"page_0.pbm\", out)?;
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
cargo build --release

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

- `segment`: Segment parsing and processing
- `huffman`: Huffman decoding and table management
- `visitor`: Segment visitor and page assembly
- `decode`: Region and coding-mode decoders
- `image`: Document and page types
- `arithmetic`: Arithmetic decoder implementation

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
