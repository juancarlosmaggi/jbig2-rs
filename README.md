# JBIG2 Decoder in Rust

A pure Rust implementation of a JBIG2 decoder, ported from Mozilla's PDF.js library.

## Description

This crate provides a permissive JBIG2 decoder written entirely in Rust. JBIG2 is a lossless/lossy compression standard for bi-level images, commonly used in PDF files for scanned documents.

## Features

- **Complete JBIG2 decoding**: Supports all major JBIG2 features including arithmetic coding, Huffman coding, and MMR compression
- **Multi-page support**: Can decode JBIG2 documents with multiple pages
- **Symbol dictionary support**: Handles both immediate and intermediate symbol dictionaries
- **Text region decoding**: Supports text regions with symbol instances
- **Halftone and generic regions**: Full support for halftone patterns and generic bitmap regions
- **Refinement regions**: Supports refinement decoding for improved quality
- **Custom Huffman tables**: Supports both standard and custom Huffman tables
- **Error handling**: Comprehensive validation and error reporting

## Motivation

The original JBIG2 implementation in PDF.js is written in JavaScript and licensed under Apache-2.0. This Rust port aims to provide a high-performance, memory-safe alternative that can be easily integrated into Rust applications, avoiding the need for JavaScript dependencies.

## Original Source

This implementation is a direct port of the JBIG2 decoder from [Mozilla's PDF.js](https://github.com/mozilla/pdf.js/blob/master/src/core/jbig2.js).

## License

Licensed under MIT OR Apache-2.0, same as the original.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
jbig2-rs = "0.1.0"
```

Example usage:

```rust
use jbig2_rs::Jbig2Document;

let data = std::fs::read("example.jbig2")?;
let document = Jbig2Document::parse(&data)?;

println!("Document has {} pages", document.page_count());

if let Some(page) = document.get_page(0) {
    println!("First page size: {}x{}", page.page_info.width, page.page_info.height);
}
```

## Building

```bash
cargo build
```

## Testing

```bash
cargo test
```

## Implementation Status

See [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md) for detailed implementation status and roadmap.