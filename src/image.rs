use crate::error::Jbig2Error;

pub struct Jbig2Image {
    pub width: usize,
    pub height: usize,
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
        }
    }

    pub fn parse_chunks(&mut self, _chunks: Vec<Jbig2Chunk>) -> Result<(), Jbig2Error> {
        // TODO: implement parseJbig2Chunks
        Err(Jbig2Error::new("parse_chunks not implemented"))
    }

    pub fn parse(&mut self, _data: &[u8]) -> Result<Vec<u8>, Jbig2Error> {
        // TODO: implement parseJbig2
        Err(Jbig2Error::new("parse not implemented"))
    }
}

pub struct Jbig2Chunk {
    pub data: Vec<u8>,
    pub start: usize,
    pub end: usize,
}