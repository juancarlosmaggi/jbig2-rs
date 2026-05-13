use crate::common::error::Jbig2Error;
use crate::common::options::DecodeOptions;
use crate::common::profile::DecodeProfile;
use crate::document::page::Jbig2Page;
use crate::parser::segment::{process_segments, read_segments};
use crate::parser::visitor::SimpleSegmentVisitor;
use std::time::Instant;

/// Represents a slice of JBIG2 data used for incremental decoding.
///
/// Chunks allow callers to decode JBIG2 content that arrives in separate buffers.
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

/// Represents a decoded JBIG2 document with one or more pages.
///
/// A document is built by parsing a full data stream or a sequence of chunks.
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
    /// Create an empty document with no pages.
    pub fn new() -> Self {
        Jbig2Document { pages: Vec::new() }
    }

    /// Parse JBIG2 data from a byte slice and return a document.
    ///
    /// The parser accepts streams with or without a file header, and handles
    /// both sequential and random-access segment ordering.
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
        Self::parse_with_options(data, &DecodeOptions::default())
    }

    /// Parse JBIG2 data with resource limits and cancellation.
    pub fn parse_with_options(data: &[u8], options: &DecodeOptions) -> Result<Self, Jbig2Error> {
        let (document, _) = Self::parse_impl(data, options, false)?;
        Ok(document)
    }

    pub fn parse_with_profile(data: &[u8]) -> Result<(Self, DecodeProfile), Jbig2Error> {
        let options = DecodeOptions::default().with_profile(true);
        Self::parse_with_options_and_profile(data, &options)
    }

    /// Parse JBIG2 data with resource limits, cancellation, and profiling.
    pub fn parse_with_options_and_profile(
        data: &[u8],
        options: &DecodeOptions,
    ) -> Result<(Self, DecodeProfile), Jbig2Error> {
        let (document, profile) = Self::parse_impl(data, options, true)?;
        Ok((document, profile.unwrap_or_default()))
    }

    fn parse_impl(
        data: &[u8],
        options: &DecodeOptions,
        collect_profile: bool,
    ) -> Result<(Self, Option<DecodeProfile>), Jbig2Error> {
        options.check_cancelled()?;
        options.limits.check_input_bytes(data.len())?;

        let total_start = Instant::now();
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

        options.limits.check_page_count(num_pages as usize)?;

        let mut visitor = if collect_profile {
            SimpleSegmentVisitor::new_with_profile_and_options(
                options.limits.clone(),
                options.cancel_flag(),
            )
        } else {
            SimpleSegmentVisitor::new_with_options(options.limits.clone(), options.cancel_flag())
        };

        let read_start = collect_profile.then(Instant::now);
        let segments = read_segments(
            data,
            pos,
            data.len(),
            sequential,
            data_start,
            has_file_header,
        )?;
        if let Some(read_start) = read_start {
            visitor.record_profile("read_segments", read_start.elapsed());
        }
        options.check_cancelled()?;
        options.limits.check_segment_count(segments.len())?;
        if segments.is_empty() {
            return Err(Jbig2Error::new("no segments found"));
        }
        process_segments(&segments, &mut visitor)?;
        visitor.finalize_current_page()?;

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
        options.limits.check_page_count(visitor.pages.len())?;

        if collect_profile {
            visitor.record_profile("total_decode", total_start.elapsed());
        }
        let profile = visitor.take_profile();

        Ok((
            Jbig2Document {
                pages: visitor.pages,
            },
            profile,
        ))
    }

    /// Parse JBIG2 data from multiple chunks.
    ///
    /// Chunks are processed sequentially and are treated as headerless data.
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
        Self::parse_chunks_with_options(chunks, &DecodeOptions::default())
    }

    /// Parse multiple JBIG2 chunks with resource limits and cancellation.
    pub fn parse_chunks_with_options(
        chunks: &[Jbig2Chunk],
        options: &DecodeOptions,
    ) -> Result<Self, Jbig2Error> {
        let (document, _) = Self::parse_chunks_impl(chunks, options, false)?;
        Ok(document)
    }

    pub fn parse_chunks_with_profile(
        chunks: &[Jbig2Chunk],
    ) -> Result<(Self, DecodeProfile), Jbig2Error> {
        let options = DecodeOptions::default().with_profile(true);
        Self::parse_chunks_with_options_and_profile(chunks, &options)
    }

    /// Parse multiple chunks with resource limits, cancellation, and profiling.
    pub fn parse_chunks_with_options_and_profile(
        chunks: &[Jbig2Chunk],
        options: &DecodeOptions,
    ) -> Result<(Self, DecodeProfile), Jbig2Error> {
        let (document, profile) = Self::parse_chunks_impl(chunks, options, true)?;
        Ok((document, profile.unwrap_or_default()))
    }

    fn parse_chunks_impl(
        chunks: &[Jbig2Chunk],
        options: &DecodeOptions,
        collect_profile: bool,
    ) -> Result<(Self, Option<DecodeProfile>), Jbig2Error> {
        options.check_cancelled()?;
        let total_input_bytes = chunks.iter().try_fold(0usize, |total, chunk| {
            if chunk.start > chunk.end || chunk.end > chunk.data.len() {
                return Err(Jbig2Error::new("invalid chunk bounds"));
            }
            total.checked_add(chunk.end - chunk.start).ok_or_else(|| {
                Jbig2Error::resource_limit_exceeded("input bytes", usize::MAX, usize::MAX)
            })
        })?;
        options.limits.check_input_bytes(total_input_bytes)?;

        let total_start = Instant::now();
        let mut visitor = if collect_profile {
            SimpleSegmentVisitor::new_with_profile_and_options(
                options.limits.clone(),
                options.cancel_flag(),
            )
        } else {
            SimpleSegmentVisitor::new_with_options(options.limits.clone(), options.cancel_flag())
        };
        let mut segment_count = 0usize;
        for chunk in chunks {
            options.check_cancelled()?;
            let read_start = collect_profile.then(Instant::now);
            let segments = read_segments(
                &chunk.data,
                chunk.start,
                chunk.end,
                true,
                chunk.start,
                false,
            )?;
            if let Some(read_start) = read_start {
                visitor.record_profile("read_segments", read_start.elapsed());
            }
            segment_count = segment_count.checked_add(segments.len()).ok_or_else(|| {
                Jbig2Error::resource_limit_exceeded("segment count", usize::MAX, usize::MAX)
            })?;
            options.limits.check_segment_count(segment_count)?;
            process_segments(&segments, &mut visitor)?;
        }
        visitor.finalize_current_page()?;
        options.limits.check_page_count(visitor.pages.len())?;
        if collect_profile {
            visitor.record_profile("total_decode", total_start.elapsed());
        }
        let profile = visitor.take_profile();
        Ok((
            Jbig2Document {
                pages: visitor.pages,
            },
            profile,
        ))
    }

    /// Return the total number of pages in the document.
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// Return a reference to a page by index.
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
