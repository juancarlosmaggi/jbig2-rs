//! Visitor Pattern for JBIG2 Segment Processing
//!
//! This module implements the visitor pattern for processing decoded JBIG2 segments
//! and assembling them into complete page images.
//!
//! ## Overview
//!
//! The visitor pattern separates segment parsing (handled by the [`segment`](crate::segment) module)
//! from segment processing and page assembly. This design provides:
//! - **Separation of concerns** - Parsing vs. processing logic
//! - **Extensibility** - Easy to add new segment handlers
//! - **Maintainability** - Each handler is focused on a specific segment type
//!
//! ## Visitor Pattern
//!
//! The [`SimpleSegmentVisitor`] struct acts as the central visitor, dispatching segment
//! processing to specialized handler modules. Each segment type has a corresponding
//! handler method (e.g., `on_symbol_dictionary`, `on_text_region`).
//!
//! ## Handler Modules
//!
//! Handlers are organized by segment type:
//! - **`page_handler`** - Page management and `Jbig2Page` struct
//! - **`symbol_handler`** - Symbol dictionary decoding
//! - **`text_handler`** - Text region decoding
//! - **`region_handlers`** - Generic region drawing and compositing
//! - **`pattern_handler`** - Pattern dictionary support
//! - **`halftone_handler`** - Halftone region support
//! - **`tables_handler`** - Custom Huffman table storage
//!
//! ## Page Assembly
//!
//! As segments are processed:
//! 1. **Symbol dictionaries** are decoded and stored
//! 2. **Page information** defines page dimensions
//! 3. **Region segments** are decoded and composited onto the page bitmap
//! 4. **End-of-page** triggers finalization
//!
//! ## Usage
//!
//! ```no_run
//! use jbig2_rs::visitor::SimpleSegmentVisitor;
//! use jbig2_rs::segment::{read_segments, process_segments};
//!
//! # fn example(data: &[u8]) -> Result<(), jbig2_rs::Jbig2Error> {
//! let segments = read_segments(data, 0, data.len(), true, 0, false)?;
//! let mut visitor = SimpleSegmentVisitor::new();
//! process_segments(&segments, &mut visitor)?;
//! visitor.finalize_current_page();
//! # Ok(())
//! # }
//! ```

// Handler modules
mod halftone_handler;
mod page_handler;
mod pattern_handler;
mod region_handlers;
mod symbol_handler;
mod tables_handler;
mod text_handler;

// Main visitor module
pub mod simple_visitor;

pub use simple_visitor::{Jbig2Page, SimpleSegmentVisitor};
