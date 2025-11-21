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

## Architecture

The library is organized into focused modules:

- **`segment`**: Segment parsing and processing (ITU T.88 section 7)
- **`huffman`**: Huffman decoding with standard and custom tables
- **`visitor`**: Segment handler pattern for processing decoded segments
- **`decode`**: Format-specific decoders (MMR, symbol dictionary, text region, etc.)
- **`image`**: High-level API for document and page management
- **`arithmetic`**: Arithmetic decoder implementation

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
jbig2-rs = "0.1.0"
```

### Basic Usage

Decode a JBIG2 file and access the first page:

```rust
use jbig2_rs::Jbig2Document;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read file data
    let data = fs::read("document.jb2")?;
    
    // Parse document
    let document = Jbig2Document::parse(&data)?;
    
    println!("Document has {} pages", document.page_count());
    
    // Get the first page
    if let Some(page) = document.get_page(0) {
        println!("Page dimensions: {}x{}", page.page_info.width, page.page_info.height);
        
        // Get raw bitmap data (1 bit per pixel, packed)
        let bitmap_data = page.to_image_data();
        
        // Do something with the data...
    }
    
    Ok(())
}
```

### Chunk-Based Decoding

Decode JBIG2 data embedded in PDF streams (split into chunks):

```rust
use jbig2_rs::{Jbig2Document, Jbig2Chunk};

fn decode_chunks(global_data: Vec<u8>, page_data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let chunks = vec![
        Jbig2Chunk {
            data: global_data,
            start: 0,
            end: 0, // Set to data length
        },
        Jbig2Chunk {
            data: page_data,
            start: 0,
            end: 0, // Set to data length
        },
    ];
    
    let document = Jbig2Document::parse_chunks(&chunks)?;
    // ...
    Ok(())
}
```

## Examples

The repository includes runnable examples:

```bash
# Decode a file and save raw bitmap
cargo run --example decode_file -- input.jb2 output.bin

# Run chunk decoding example
cargo run --example decode_chunks
```

## Building

```bash
cargo build
```

## Testing

```bash
cargo test
```