use crate::error::Jbig2Error;
use crate::bitmap::Bitmap;
use crate::segment::{read_segments, process_segments};
use crate::visitor::SimpleSegmentVisitor;

#[derive(Clone)]
pub struct Jbig2Image {
    pub width: usize,
    pub height: usize,
    pub bitmap: Bitmap,
}

impl Default for Jbig2Image {
    fn default() -> Self {
        Self::new()
    }
}

impl Jbig2Image {
    pub fn new() -> Self {
        Jbig2Image {
            width: 0,
            height: 0,
            bitmap: Bitmap::new(0, 0),
        }
    }

    pub fn parse(&mut self, data: &[u8]) -> Result<(), Jbig2Error> {
        if data.len() < 8 || &data[0..8] != b"\x97\x4a\x42\x32\x0d\x0a\x1a\x0a" {
            return Err(Jbig2Error::new("invalid header"));
        }
        let pos = 8;
        let segments = read_segments(data, pos, data.len())?;
        let mut visitor = SimpleSegmentVisitor::new();
        process_segments(&segments, &mut visitor)?;
        let page_info = visitor.current_page_info.ok_or(Jbig2Error::new("no page info"))?;
        self.width = page_info.width as usize;
        self.height = page_info.height as usize;
        self.bitmap = visitor.bitmap.ok_or(Jbig2Error::new("no bitmap"))?.clone();
        Ok(())
    }
}