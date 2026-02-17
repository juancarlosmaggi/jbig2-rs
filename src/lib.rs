#![allow(clippy::collapsible_if)]

//! # jbig2-rs
//!
//! A pure Rust implementation of the JBIG2 image compression standard (ITU-T T.88).
//!
//! JBIG2 is a lossy or lossless compression standard for bi-level (1-bit monochrome) images,
//! commonly used in PDF files and document scanning applications. This library provides
//! complete decoding capabilities for JBIG2 data streams.
//!
//! ## Features
//!
//! - **Complete JBIG2 decoding** - Supports all major segment types and encoding modes
//! - **Multiple encoding modes** - MMR, arithmetic coding, Huffman encoding
//! - **Standards compliant** - Follows ITU-T T.88 specification
//! - **Pure Rust** - No external C dependencies
//! - **Multi-page support** - Handle documents with multiple pages
//!
//! ## Quick Start
//!
//! Decode a JBIG2 file and convert to raw bitmap data:
//!
//! ```no_run
//! use jbig2_rs::{Jbig2Document, Jbig2Error};
//!
//! fn decode_jbig2(data: &[u8]) -> Result<Vec<u8>, Jbig2Error> {
//!     // Parse the JBIG2 document
//!     let document = Jbig2Document::parse(data)?;
//!     
//!     // Get the first page
//!     if let Some(page) = document.get_page(0) {
//!         Ok(page.to_image_data())
//!     } else {
//!         Err(Jbig2Error::new("no pages in document"))
//!     }
//! }
//! ```
//!
//! ## Architecture
//!
//! The library is organized into focused modules:
//!
//! ```mermaid
//! graph TD
//!     A[jbig2-rs] --> B[parser]
//!     A --> C[huffman]
//!     A --> D[decoders]
//!     A --> E[document]
//!     A --> F[bitmap]
//!     A --> G[arithmetic]
//!     
//!     B --> B1[segment]
//!     B --> B2[visitor]
//!     
//!     E --> E1[core]
//!     E --> E2[page]
//!     E --> E3[info]
//! ```
//!
//! ## Module Organization
//!
//! - **[`parser`]** - Segment parsing, processing, and visitor implementation
//! - **[`huffman`]** - Huffman decoding with standard and custom tables
//! - **[`decoders`]** - Format-specific decoders (MMR, symbol dictionary, text region, etc.)
//! - **[`document`]** - High-level API for document and page management
//! - **[`common`]** - Shared utilities and error types
//! - **[`bitmap`]** - Bitmap data structures and operations
//! - **[`arithmetic`]** - Arithmetic decoder implementation
//!
//! ## Main Types
//!
//! - [`Jbig2Document`] - Represents a complete JBIG2 document with one or more pages
//! - [`Jbig2Page`] - Represents a decoded page with bitmap and metadata
//! - [`Jbig2Image`] - Type alias for a single page (backward compatibility)
//! - [`Jbig2Error`] - Structured error type with context information
//!
//! ## Decoding Process
//!
//! 1. **Parse file header** - Detect file format and extract metadata
//! 2. **Read segments** - Parse segment headers and data
//! 3. **Process segments** - Dispatch to appropriate decoders
//! 4. **Build pages** - Assemble decoded regions into complete pages
//!
//! ## Standards Compliance
//!
//! This implementation follows:
//! - **ITU-T T.88** - JBIG2 specification
//! - **ITU-T T.6** - CCITT Group 4 (MMR) specification
//! - **ITU-T T.82** - Arithmetic coding specification references
//!
//! ## Error Handling
//!
//! The library uses structured error types that include position and segment context:
//!
//! ```no_run
//! # use jbig2_rs::Jbig2Error;
//! // Errors include helpful context
//! let err = Jbig2Error::invalid_template_index(5, 3);
//! // Displays: "Jbig2Error: Invalid template index: 5 (max: 3)"
//! ```

pub mod arithmetic;
pub mod bitmap;
pub mod common;
pub mod decoders;
pub mod document;
pub mod huffman;
pub mod parser;
pub mod probe;

pub use common::error::Jbig2Error;
pub use common::profile::DecodeProfile;
pub use document::{Jbig2Chunk, Jbig2Document, Jbig2Image};
pub use probe::probe_stream_consumed_bytes;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jbig2_document_creation() {
        let doc = Jbig2Document::new();
        assert_eq!(doc.page_count(), 0);
    }

    #[test]
    fn test_jbig2_header_validation() {
        // Reject inputs that are shorter than the minimal header.
        let invalid_data = b"\x00\x00\x00\x00";
        assert!(Jbig2Document::parse(invalid_data).is_err());

        // Reject inputs with the wrong magic bytes.
        let invalid_data2 = b"\x00\x4a\x42\x32\x0d\x0a\x1a\x0a\x00\x00\x00\x00";
        assert!(Jbig2Document::parse(invalid_data2).is_err());
    }

    #[test]
    fn test_bitmap_operations() {
        let mut bitmap = crate::bitmap::Bitmap::new(10, 10);
        bitmap.set_pixel(5, 5, 1);
        assert_eq!(bitmap.get_pixel(5, 5), 1);
        assert_eq!(bitmap.get_pixel(0, 0), 0);

        // Out-of-range reads return zero and writes are ignored.
        assert_eq!(bitmap.get_pixel(15, 15), 0);
        bitmap.set_pixel(15, 15, 1);
    }

    #[test]
    fn test_huffman_tables() {
        // Standard table lookup should succeed for a known ID.
        let table1 = crate::huffman::get_standard_table(1);
        assert!(table1.is_ok());

        // Unknown table IDs should report an error.
        let table_invalid = crate::huffman::get_standard_table(999);
        assert!(table_invalid.is_err());
    }

    #[test]
    fn test_segment_header_parsing() {
        // Minimal segment header should parse without error.
        let data = vec![
            0x00, 0x00, 0x00, 0x01, // segment number
            0x00, // flags (type 0)
            0x00, // referred flags
            0x00, 0x00, 0x00, 0x00, // page association
            0x00, 0x00, 0x00, 0x00, // length
        ];
        let header = crate::parser::segment::read_segment_header(&data, 0, false);
        assert!(header.is_ok());
        let header = header.unwrap();
        assert_eq!(header.segment_type, 0);
        assert_eq!(header.number, 1);
    }

    #[test]
    fn test_draw_symbol_at_position() {
        let mut bitmap = crate::bitmap::Bitmap::new(10, 10);
        let mut symbol = crate::bitmap::Bitmap::new(2, 2);
        symbol.set_pixel(0, 0, 1);
        symbol.set_pixel(1, 1, 1);
        crate::bitmap::utils::draw_symbol_at_position(&mut bitmap, &symbol, 1, 1, 0);
        assert_eq!(bitmap.get_pixel(1, 1), 1);
        assert_eq!(bitmap.get_pixel(2, 2), 1);
        assert_eq!(bitmap.get_pixel(0, 0), 0);
    }
}
