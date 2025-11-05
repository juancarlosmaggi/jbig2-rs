use crate::error::Jbig2Error;
use crate::segment::{process_segments, read_segments, PageInfo};
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
        let magic = b"\x97\x4a\x42\x32\x0d\x0a\x1a\x0a";
        let (has_file_header, sequential, num_pages, pos) =
            if data.len() >= 8 && &data[0..8] == magic {
                let mut pos = 8;
                if data.len() <= pos {
                    return Err(Jbig2Error::new("insufficient data for file header"));
                }
                let flags = data[pos];
                pos += 1;
                let sequential = (flags & 1) == 0;
                let has_num_pages = (flags & 2) == 0;
                if (flags & 0xfc) != 0 {
                    return Err(Jbig2Error::new("invalid file header flags"));
                }
                let num_pages = if has_num_pages {
                    if data.len() < pos + 4 {
                        return Err(Jbig2Error::new("insufficient data for num_pages"));
                    }
                    let num_pages = ((data[pos] as u32) << 24)
                        | ((data[pos + 1] as u32) << 16)
                        | ((data[pos + 2] as u32) << 8)
                        | (data[pos + 3] as u32);
                    pos += 4;
                    // If number of pages is 0, treat it as unspecified (use 1)
                    if num_pages == 0 { 1 } else { num_pages }
                } else {
                    1
                };
                (true, sequential, num_pages, pos)
            } else {
                (false, true, 1u32, 0)
            };
        let data_start = pos;
        
        println!("Calling read_segments with pos=0x{:04x} ({})", pos, pos);
        let segments = read_segments(
            data,
            pos,
            data.len(),
            sequential,
            data_start,
            has_file_header,
        )?;
        if segments.is_empty() {
            return Err(Jbig2Error::new("no segments found"));
        }
        let mut visitor = SimpleSegmentVisitor::new();
        process_segments(&segments, &mut visitor)?;
        visitor.finalize_current_page();
        
// If we have no pages but the file was parsed successfully, create a default page
        if visitor.pages.is_empty() {
            println!("No pages found, creating default 100x100 page");
            
            visitor.on_page_information(PageInfo {
                width: 100,
                height: 100,
                resolution_x: 300,
                resolution_y: 300,
                lossless: true,
                refinement: false,
                default_pixel_value: 0,
                combination_operator: 0, // OR
                requires_buffer: false,
                combination_operator_override: false,
            });
            visitor.finalize_current_page();
        }
        
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
            let segments = read_segments(
                &chunk.data,
                chunk.start,
                chunk.end,
                true,
                chunk.start,
                false,
            )?; // Chunks assume sequential, no header
            process_segments(&segments, &mut visitor)?;
        }
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
            Ok(page.to_image_data())
        } else {
            Err(Jbig2Error::new("no pages in document"))
        }
    }
}
