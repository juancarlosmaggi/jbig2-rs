# JBIG2 Decoder in Rust

A pure Rust implementation of a JBIG2 decoder, ported from Mozilla's PDF.js library.

## Description

This crate provides a permissive JBIG2 decoder written entirely in Rust. JBIG2 is a lossless/lossy compression standard for bi-level images, commonly used in PDF files for scanned documents.

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
use jbig2_rs::Jbig2Image;

let data = std::fs::read("example.jbig2")?;
let image = Jbig2Image::parse(&data)?;
println!("Image size: {}x{}", image.width, image.height);
```

## Building

```bash
cargo build
```