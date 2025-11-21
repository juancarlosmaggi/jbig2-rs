// Handler modules
mod page_handler;
mod region_handlers;
mod symbol_handler;
mod text_handler;
mod pattern_handler;
mod halftone_handler;
mod tables_handler;

// Main visitor module
pub mod simple_visitor;

pub use simple_visitor::{Jbig2Page, SimpleSegmentVisitor};
