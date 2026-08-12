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

pub use crate::document::Jbig2Page;
pub use simple_visitor::SimpleSegmentVisitor;

use crate::bitmap::Bitmap;
use crate::document::PageInfo;
use crate::huffman::HuffmanTable;
use std::collections::HashMap;

/// Page bitmap the current segment should composite onto.
pub(super) struct PageComposeTarget<'a> {
    pub page_info: &'a mut Option<PageInfo>,
    pub bitmap: &'a mut Option<Bitmap>,
}

/// Byte range of a segment payload inside the source buffer.
#[derive(Clone, Copy)]
pub(super) struct SegmentSlice<'a> {
    pub data: &'a [u8],
    pub start: usize,
    pub end: usize,
}

impl<'a> SegmentSlice<'a> {
    pub(super) fn as_slice(self) -> &'a [u8] {
        &self.data[self.start..self.end]
    }
}

/// Dictionaries and intermediate bitmaps accumulated while decoding a page.
pub(super) struct IntermediateResources<'a> {
    pub symbols: &'a HashMap<u32, Vec<Bitmap>>,
    pub patterns: &'a HashMap<u32, Vec<Bitmap>>,
    pub custom_tables: &'a HashMap<u32, HuffmanTable>,
    pub bitmaps: &'a mut HashMap<u32, Bitmap>,
}
