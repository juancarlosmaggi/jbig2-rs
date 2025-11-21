//! JBIG2 Decoders Module
//!
//! This module contains format-specific decoders for various JBIG2 encoding modes and
//! region types.
//!
//! ## Overview
//!
//! JBIG2 supports multiple encoding modes and region types, each with specialized decoders:
//!
//! ### Encoding Modes
//!
//! - **MMR (Modified Modified READ)** - ITU-T T.6 CCITT Group 4 fax encoding
//! - **Arithmetic Coding** - Context-based adaptive binary arithmetic coding
//! - **Huffman Coding** - Variable-length prefix coding
//!
//! ### Region Decoders
//!
//! - **Symbol Dictionary** ([`decode_symbol`]) - Reusable glyph definitions
//! - **Text Region** ([`decode_text`]) - Text composed of symbol references
//! - **Generic Region** ([`decode_generic`]) - Arbitrary bitmap data
//! - **Halftone Region** ([`decode_halftone`]) - Halftone patterns
//! - **Pattern Dictionary** ([`decode_pattern`]) - Pattern definitions for halftoning
//! - **Refinement Region** ([`decode_refinement`]) - Refinement of existing bitmaps
//!
//! ## Module Contents
//!
//! - `decode_mmr` - MMR decoder (ITU-T T.6 CCITT Group 4)
//! - `decode_symbol` - Symbol dictionary decoder
//! - `decode_text` - Text region decoder
//! - `decode_generic` - Generic region decoder
//! - `decode_halftone` - Halftone region decoder
//! - `decode_pattern` - Pattern dictionary decoder  
//! - `decode_refinement` - Refinement region decoder
//! - `decode_symbol_helpers` - Helper functions for symbol decoding
//! - `decode_utils` - Shared utilities
//! - `mmr_tables` - MMR run-length coding tables

pub mod decode_generic;
pub mod decode_halftone;
pub mod decode_mmr;
pub mod decode_pattern;
pub mod decode_refinement;
pub mod decode_symbol;
pub mod decode_symbol_helpers;
pub mod decode_text;
pub mod decode_utils;
pub mod mmr_tables;
