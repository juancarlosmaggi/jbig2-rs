use crate::error::Jbig2Error;

// Bitmap is represented as Vec<Vec<u8>> where each inner vec is a row
pub type Bitmap = Vec<Vec<u8>>;

pub struct ContextCache {
    contexts: std::collections::HashMap<String, Vec<i8>>,
}

impl Default for ContextCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextCache {
    pub fn new() -> Self {
        ContextCache {
            contexts: std::collections::HashMap::new(),
        }
    }

    pub fn get_contexts(&mut self, id: &str) -> &mut Vec<i8> {
        self.contexts.entry(id.to_string()).or_insert_with(|| vec![0; 1 << 16])
    }
}

pub struct DecodingContext {
    pub data: Vec<u8>,
    pub start: usize,
    pub end: usize,
    pub context_cache: ContextCache,
    // decoder would be ArithmeticDecoder, but we need to implement that
}

impl DecodingContext {
    pub fn new(data: Vec<u8>, start: usize, end: usize) -> Self {
        DecodingContext {
            data,
            start,
            end,
            context_cache: ContextCache::new(),
        }
    }
}

// Placeholder for ArithmeticDecoder
pub struct ArithmeticDecoder;

impl ArithmeticDecoder {
    pub fn new(_data: &[u8], _start: usize, _end: usize) -> Self {
        ArithmeticDecoder
    }

    pub fn read_bit(&mut self, _contexts: &mut Vec<i8>, _prev: usize) -> u8 {
        // TODO: implement
        0
    }
}

// Coding templates from the JS
pub fn get_coding_template(index: usize) -> &'static [(i8, i8)] {
    match index {
        0 => &[
            (-1, -2), (0, -2), (1, -2), (-2, -1), (-1, -1), (0, -1), (1, -1), (2, -1), (-4, 0), (-3, 0), (-2, 0), (-1, 0),
        ],
        1 => &[
            (-1, -2), (0, -2), (1, -2), (2, -2), (-2, -1), (-1, -1), (0, -1), (1, -1), (2, -1), (-3, 0), (-2, 0), (-1, 0),
        ],
        2 => &[
            (-1, -2), (0, -2), (1, -2), (-2, -1), (-1, -1), (0, -1), (1, -1), (-2, 0), (-1, 0),
        ],
        3 => &[
            (-3, -1), (-2, -1), (-1, -1), (0, -1), (1, -1), (-4, 0), (-3, 0), (-2, 0), (-1, 0),
        ],
        _ => &[],
    }
}

#[derive(Clone)]
pub struct RefinementTemplate {
    pub coding: Vec<(i8, i8)>,
    pub reference: Vec<(i8, i8)>,
}

pub fn get_refinement_template(index: usize) -> RefinementTemplate {
    match index {
        0 => RefinementTemplate {
            coding: vec![(0, -1), (1, -1), (-1, 0)],
            reference: vec![(0, -1), (1, -1), (-1, 0), (0, 0), (1, 0), (-1, 1), (0, 1), (1, 1)],
        },
        1 => RefinementTemplate {
            coding: vec![(-1, -1), (0, -1), (1, -1), (-1, 0)],
            reference: vec![(0, -1), (-1, 0), (0, 0), (1, 0), (0, 1), (1, 1)],
        },
        _ => RefinementTemplate {
            coding: vec![],
            reference: vec![],
        },
    }
}

pub const REUSED_CONTEXTS: [u16; 4] = [
    0x9b25, // 10011 0110010 0101
    0x0795, // 0011 110010 101
    0x00e5, // 001 11001 01
    0x0195, // 011001 0101
];

pub const REFINEMENT_REUSED_CONTEXTS: [u16; 2] = [
    0x0020, // '000' + '0' (coding) + '00010000' + '0' (reference)
    0x0008, // '0000' + '001000'
];

// Placeholder implementations
#[allow(clippy::too_many_arguments)]
pub fn decode_bitmap(
    _mmr: bool,
    _width: usize,
    _height: usize,
    _template_index: usize,
    _prediction: bool,
    _skip: Option<&Bitmap>,
    _at: Vec<(i8, i8)>,
    _decoding_context: &mut DecodingContext,
) -> Result<Bitmap, Jbig2Error> {
    // TODO: implement full decodeBitmap
    Err(Jbig2Error::new("decode_bitmap not implemented"))
}

#[allow(clippy::too_many_arguments)]
pub fn decode_refinement(
    _width: usize,
    _height: usize,
    _template_index: usize,
    _reference_bitmap: &Bitmap,
    _offset_x: i32,
    _offset_y: i32,
    _prediction: bool,
    _at: Vec<(i8, i8)>,
    _decoding_context: &mut DecodingContext,
) -> Result<Bitmap, Jbig2Error> {
    // TODO: implement
    Err(Jbig2Error::new("decode_refinement not implemented"))
}