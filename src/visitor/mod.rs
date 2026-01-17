//! Segment visitor implementations and page assembly helpers.

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
