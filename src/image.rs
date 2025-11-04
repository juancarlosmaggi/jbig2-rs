use crate::error::Jbig2Error;
use crate::segment::{process_segments, read_segments};
use crate::visitor::{Jbig2Page, SimpleSegmentVisitor};
#[derive(Clone)]
pub struct Jbig2Chunk {
    pub data: Vec<u8>,
    pub start: usize,
    pub end: usize,
}
#[derive(Clone)]
pub struct Jbig2Document {
    pub pages: Vec<Jbig2Page>,
}
impl Default for Jbig2Document {
    fn default() -> Self {
        Self::new()
    }
}
impl Jbig2Document {
    pub fn new() -> Self {
        Jbig2Document { pages: Vec::new() }
    }
    pub fn parse(data: &[u8]) -> Result<Self, Jbig2Error> {
        if data.len() < 8 || &data[0..8] != b"\x97\x4a\x42\x32\x0d\x0a\x1a\x0a" {
            return Err(Jbig2Error::new("invalid header"));
        }
        let mut pos = 8;
        let mut has_file_header = true;
        let mut sequential = true;
        let mut num_pages = 1u32; // Default for no header or sequential
        let mut data_start = pos;
        // Parse optional file header
        if data.len() > pos {
            let flags = data[pos];
            pos += 1;
            sequential = (flags & 1) != 0;
            let has_num_pages = (flags & 2) == 0;
            // Reserved bits must be 0
            if (flags & 0xfc) != 0 {
                return Err(Jbig2Error::new("invalid file header flags"));
            }
            if !has_num_pages {
                num_pages = 1;
            } else if data.len() >= pos + 4 {
                num_pages = ((data[pos] as u32) << 24)
                    | ((data[pos + 1] as u32) << 16)
                    | ((data[pos + 2] as u32) << 8)
                    | (data[pos + 3] as u32);
                pos += 4;
            } else {
                return Err(Jbig2Error::new("insufficient data for num_pages"));
            }
            data_start = pos;
        } else {
            has_file_header = false;
            // No header: implicit sequential, single-page
        }
        let segments = read_segments(data, pos, data.len(), sequential, data_start)?;
        let mut visitor = SimpleSegmentVisitor::new();
        process_segments(&segments, &mut visitor)?;
        // Finalize any remaining page
        visitor.finalize_current_page();
        // Validate page count matches header if present
        if has_file_header
            && !sequential
            && num_pages != 0
            && num_pages as usize != visitor.pages.len()
        {
            return Err(Jbig2Error::new("page count mismatch"));
        }
        Ok(Jbig2Document {
            pages: visitor.pages,
        })
    }
    pub fn parse_chunks(chunks: &[Jbig2Chunk]) -> Result<Self, Jbig2Error> {
        let mut visitor = SimpleSegmentVisitor::new();
        for chunk in chunks {
            if chunk.start > chunk.end || chunk.end > chunk.data.len() {
                return Err(Jbig2Error::new("invalid chunk bounds"));
            }
            let segments = read_segments(&chunk.data, chunk.start, chunk.end, true, chunk.start)?; // Chunks assume sequential
            process_segments(&segments, &mut visitor)?;
        }
        // Finalize any remaining page
        visitor.finalize_current_page();
        Ok(Jbig2Document {
            pages: visitor.pages,
        })
    }
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }
    pub fn get_page(&self, index: usize) -> Option<&Jbig2Page> {
        self.pages.get(index)
    }
}
// For backward compatibility, keep Jbig2Image as an alias for single-page documents
pub type Jbig2Image = Jbig2Page;
impl Jbig2Image {
    pub fn parse(data: &[u8]) -> Result<Vec<u8>, Jbig2Error> {
        let doc = Jbig2Document::parse(data)?;
        if let Some(page) = doc.get_page(0) {
            Ok(page.bit_packed_data.clone())
        } else {
            Err(Jbig2Error::new("no pages in document"))
        }
    }
}
