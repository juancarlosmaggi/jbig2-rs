use crate::error::Jbig2Error;
use crate::segment::{process_segments, read_segments};
use crate::visitor::{Jbig2Page, SimpleSegmentVisitor};

/// Represents a chunk of JBIG2 data for incremental or embedded decoding.
///
/// Chunks are useful for processing JBIG2 data that is split across multiple
/// buffers, such as when embedded in PDF streams.
///
/// # Fields
///
/// - `data` - The raw JBIG2 data bytes
/// - `start` - Starting offset within the data
/// - `end` - Ending offset (exclusive) within the data
#[derive(Clone)]
pub struct Jbig2Chunk {
    pub data: Vec<u8>,
    pub start: usize,
    pub end: usize,
}

/// Represents a complete JBIG2 document with one or more pages.
///
/// This is the main entry point for decoding JBIG2 data. A document can contain
/// multiple pages and is created by parsing JBIG2 file data or processing chunks.
///
/// # Examples
///
/// ```no_run
/// use jbig2_rs::{Jbig2Document, Jbig2Error};
/// use std::fs;
///
/// fn decode_file(path: &str) -> Result<(), Jbig2Error> {
///     let data = fs::read(path).unwrap();
///     let document = Jbig2Document::parse(&data)?;
///     
///     println!("Document has {} pages", document.page_count());
///     
///     for i in 0..document.page_count() {
///         if let Some(page) = document.get_page(i) {
///             println!("Page {}: {}x{}", i, page.page_info.width, page.page_info.height);
///         }
///     }
///     
///     Ok(())
/// }
/// ```
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
    /// Creates a new empty JBIG2 document.
    pub fn new() -> Self {
        Jbig2Document { pages: Vec::new() }
    }
    
    /// Parses JBIG2 data from a byte slice and returns a document.
    ///
    /// This method handles both file-header and random-access JBIG2 formats.
    /// File-header format includes a magic signature and metadata,
    /// while random-access format starts directly with segments.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw JBIG2 data bytes
    ///
    /// # Returns
    ///
    /// - `Ok(Jbig2Document)` - Successfully parsed document
    /// - `Err(Jbig2Error)` - Parsing error with context
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use jbig2_rs::Jbig2Document;
    ///
    /// let data = std::fs::read("document.jb2").unwrap();
    /// let doc = Jbig2Document::parse(&data)?;
    /// # Ok::<(), jbig2_rs::Jbig2Error>(())
    /// ```
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
                let sequential = (flags & 1) != 0;
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

        if visitor.pages.is_empty() {
            return Err(Jbig2Error::new(
                "no pages created after processing segments",
            ));
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
    
    /// Parses JBIG2 data from multiple chunks (useful for embedded JBIG2 in PDFs).
    ///
    /// Chunks are processed sequentially, assuming no file header.
    ///
    /// # Arguments
    ///
    /// * `chunks` - Array of JBIG2 data chunks
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use jbig2_rs::{Jbig2Document, Jbig2Chunk};
    ///
    /// let chunk = Jbig2Chunk {
    ///     data: vec![/* segment data */],
    ///     start: 0,
    ///     end: 100,
    /// };
    /// let doc = Jbig2Document::parse_chunks(&[chunk])?;
    /// # Ok::<(), jbig2_rs::Jbig2Error>(())
    /// ```
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
    
    /// Returns the total number of pages in the document.
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }
    
    /// Gets a reference to a specific page by index.
    ///
    /// # Arguments
    ///
    /// * `index` - Page index (0-based)
    ///
    /// # Returns
    ///
    /// - `Some(&Jbig2Page)` - Page reference if index is valid
    /// - `None` - If index is out of bounds
    pub fn get_page(&self, index: usize) -> Option<&Jbig2Page> {
        self.pages.get(index)
    }
}

/// Type alias for a single JBIG2 page (backward compatibility).
///
/// For new code, prefer using [`Jbig2Document`] and accessing pages via [`Jbig2Document::get_page`].
pub type Jbig2Image = Jbig2Page;

impl Jbig2Image {
    /// Convenience method to parse a JBIG2 file and return the first page's image data.
    ///
    /// This is a simplified API for single-page documents.
    ///
    /// # Returns
    ///
    /// Raw bitmap data as a vector of bytes (1 bit per pixel, packed).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use jbig2_rs::Jbig2Image;
    ///
    /// let data = std::fs::read("page.jb2").unwrap();
    /// let image_data = Jbig2Image::parse(&data)?;
    /// # Ok::<(), jbig2_rs::Jbig2Error>(())
    /// ```
    pub fn parse(data: &[u8]) -> Result<Vec<u8>, Jbig2Error> {
        let doc = Jbig2Document::parse(data)?;
        if let Some(page) = doc.get_page(0) {
            Ok(page.to_image_data())
        } else {
            Err(Jbig2Error::new("no pages in document"))
        }
    }
}
