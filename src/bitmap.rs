use crate::error::Jbig2Error;
use crate::decoder::{decode_integer_context, decode_iaid_context};
use std::cell::RefCell;
#[derive(Clone)]
pub struct Bitmap {
    pub data: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub stride: usize, // bytes per row
}
impl Bitmap {
    pub fn new(width: usize, height: usize) -> Self {
        let stride = (width + 7) >> 3;
        let data = vec![0; stride * height];
        Bitmap { data, width, height, stride }
    }
    pub fn get_pixel(&self, x: usize, y: usize) -> u8 {
        if y >= self.height || x >= self.width {
            return 0;
        }
        let byte_index = y * self.stride + (x >> 3);
        let bit_index = 7 - (x & 7);
        (self.data[byte_index] >> bit_index) & 1
    }
    pub fn set_pixel(&mut self, x: usize, y: usize, value: u8) {
        let byte_index = y * self.stride + (x >> 3);
        let bit_index = 7 - (x & 7);
        if value != 0 {
            self.data[byte_index] |= 1 << bit_index;
        } else {
            self.data[byte_index] &= !(1 << bit_index);
        }
    }
}
#[derive(Clone)]
struct QeEntry {
    qe: u16,
    nmps: u8,
    nlps: u8,
    switch_flag: u8,
}
const QE_TABLE: [QeEntry; 47] = [
    QeEntry { qe: 0x5601, nmps: 1, nlps: 1, switch_flag: 1 },
    QeEntry { qe: 0x3401, nmps: 2, nlps: 6, switch_flag: 0 },
    QeEntry { qe: 0x1801, nmps: 3, nlps: 9, switch_flag: 0 },
    QeEntry { qe: 0x0ac1, nmps: 4, nlps: 12, switch_flag: 0 },
    QeEntry { qe: 0x0521, nmps: 5, nlps: 29, switch_flag: 0 },
    QeEntry { qe: 0x0221, nmps: 38, nlps: 33, switch_flag: 0 },
    QeEntry { qe: 0x5601, nmps: 7, nlps: 6, switch_flag: 1 },
    QeEntry { qe: 0x5401, nmps: 8, nlps: 14, switch_flag: 0 },
    QeEntry { qe: 0x4801, nmps: 9, nlps: 14, switch_flag: 0 },
    QeEntry { qe: 0x3801, nmps: 10, nlps: 14, switch_flag: 0 },
    QeEntry { qe: 0x3001, nmps: 11, nlps: 17, switch_flag: 0 },
    QeEntry { qe: 0x2401, nmps: 12, nlps: 18, switch_flag: 0 },
    QeEntry { qe: 0x1c01, nmps: 13, nlps: 20, switch_flag: 0 },
    QeEntry { qe: 0x1601, nmps: 29, nlps: 21, switch_flag: 0 },
    QeEntry { qe: 0x5601, nmps: 15, nlps: 14, switch_flag: 1 },
    QeEntry { qe: 0x5401, nmps: 16, nlps: 14, switch_flag: 0 },
    QeEntry { qe: 0x5101, nmps: 17, nlps: 15, switch_flag: 0 },
    QeEntry { qe: 0x4801, nmps: 18, nlps: 16, switch_flag: 0 },
    QeEntry { qe: 0x3801, nmps: 19, nlps: 17, switch_flag: 0 },
    QeEntry { qe: 0x3401, nmps: 20, nlps: 18, switch_flag: 0 },
    QeEntry { qe: 0x3001, nmps: 21, nlps: 19, switch_flag: 0 },
    QeEntry { qe: 0x2801, nmps: 22, nlps: 19, switch_flag: 0 },
    QeEntry { qe: 0x2401, nmps: 23, nlps: 20, switch_flag: 0 },
    QeEntry { qe: 0x2201, nmps: 24, nlps: 21, switch_flag: 0 },
    QeEntry { qe: 0x1c01, nmps: 25, nlps: 22, switch_flag: 0 },
    QeEntry { qe: 0x1801, nmps: 26, nlps: 23, switch_flag: 0 },
    QeEntry { qe: 0x1601, nmps: 27, nlps: 24, switch_flag: 0 },
    QeEntry { qe: 0x1401, nmps: 28, nlps: 25, switch_flag: 0 },
    QeEntry { qe: 0x1201, nmps: 29, nlps: 26, switch_flag: 0 },
    QeEntry { qe: 0x1101, nmps: 30, nlps: 27, switch_flag: 0 },
    QeEntry { qe: 0x0ac1, nmps: 31, nlps: 28, switch_flag: 0 },
    QeEntry { qe: 0x09c1, nmps: 32, nlps: 29, switch_flag: 0 },
    QeEntry { qe: 0x08a1, nmps: 33, nlps: 30, switch_flag: 0 },
    QeEntry { qe: 0x0521, nmps: 34, nlps: 31, switch_flag: 0 },
    QeEntry { qe: 0x0441, nmps: 35, nlps: 32, switch_flag: 0 },
    QeEntry { qe: 0x02a1, nmps: 36, nlps: 33, switch_flag: 0 },
    QeEntry { qe: 0x0221, nmps: 37, nlps: 34, switch_flag: 0 },
    QeEntry { qe: 0x0141, nmps: 38, nlps: 35, switch_flag: 0 },
    QeEntry { qe: 0x0111, nmps: 39, nlps: 36, switch_flag: 0 },
    QeEntry { qe: 0x0085, nmps: 40, nlps: 37, switch_flag: 0 },
    QeEntry { qe: 0x0049, nmps: 41, nlps: 38, switch_flag: 0 },
    QeEntry { qe: 0x0025, nmps: 42, nlps: 39, switch_flag: 0 },
    QeEntry { qe: 0x0015, nmps: 43, nlps: 40, switch_flag: 0 },
    QeEntry { qe: 0x0009, nmps: 44, nlps: 41, switch_flag: 0 },
    QeEntry { qe: 0x0005, nmps: 45, nlps: 42, switch_flag: 0 },
    QeEntry { qe: 0x0001, nmps: 45, nlps: 43, switch_flag: 0 },
    QeEntry { qe: 0x5601, nmps: 46, nlps: 46, switch_flag: 0 },
];
pub struct ArithmeticDecoder {
    data: *const u8,
    len: usize,
    bp: usize,
    data_end: usize,
    chigh: u32,
    clow: u32,
    ct: u8,
    a: u16,
}
impl ArithmeticDecoder {
    pub fn new(data: &[u8], start: usize, end: usize) -> Self {
        let data_ptr = data.as_ptr();
        let len = data.len();
        let mut decoder = ArithmeticDecoder {
            data: data_ptr,
            len,
            bp: start,
            data_end: end,
            chigh: unsafe { *data_ptr.add(start) as u32 },
            clow: 0,
            ct: 0,
            a: 0,
        };
        decoder.byte_in();
        decoder.chigh = ((decoder.chigh << 7) & 0xffff) | ((decoder.clow >> 9) & 0x7f);
        decoder.clow = (decoder.clow << 7) & 0xffff;
        decoder.ct = decoder.ct.wrapping_sub(7);
        decoder.a = 0x8000;
        decoder
    }
    fn byte_in(&mut self) {
        let bp = self.bp;
        unsafe {
            if *self.data.add(bp) == 0xff {
                if bp + 1 < self.len && *self.data.add(bp + 1) > 0x8f {
                    self.clow = self.clow.wrapping_add(0xff00);
                    self.ct = 8;
                } else {
                    self.bp = bp + 1;
                    self.clow = self.clow.wrapping_add((*self.data.add(bp + 1) as u32) << 9);
                    self.ct = 7;
                }
            } else {
                self.bp = bp + 1;
                self.clow = self.clow.wrapping_add(if self.bp < self.data_end { (*self.data.add(self.bp) as u32) << 8 } else { 0xff00 });
                self.ct = 8;
            }
        }
        if self.clow > 0xffff {
            self.chigh = self.chigh.wrapping_add(self.clow >> 16);
            self.clow &= 0xffff;
        }
    }
    pub fn read_bit(&mut self, contexts: &mut [i8], pos: usize) -> u8 {
        let cx_index = (contexts[pos] >> 1) as usize;
        let mut cx_mps = (contexts[pos] & 1) as u8;
        let qe_entry = &QE_TABLE[cx_index];
        let qe_icx = qe_entry.qe;
        let d;
        let new_cx_index;
        let mut a = self.a.wrapping_sub(qe_icx);
        if self.chigh < qe_icx as u32 {
            // exchangeLps
            if a < qe_icx {
                a = qe_icx;
                d = cx_mps;
                new_cx_index = qe_entry.nmps as usize;
            } else {
                a = qe_icx;
                d = 1 ^ cx_mps;
                if qe_entry.switch_flag == 1 {
                    cx_mps = d;
                }
                new_cx_index = qe_entry.nlps as usize;
            }
        } else {
            self.chigh -= qe_icx as u32;
            if (a & 0x8000) != 0 {
                self.a = a;
                return cx_mps;
            }
            // exchangeMps
            if a < qe_icx {
                d = 1 ^ cx_mps;
                if qe_entry.switch_flag == 1 {
                    cx_mps = d;
                }
                new_cx_index = qe_entry.nlps as usize;
            } else {
                d = cx_mps;
                new_cx_index = qe_entry.nmps as usize;
            }
        }
        // renormD
        loop {
            if self.ct == 0 {
                self.byte_in();
            }
            a <<= 1;
            self.chigh = ((self.chigh << 1) & 0xffff) | ((self.clow >> 15) & 1);
            self.clow = (self.clow << 1) & 0xffff;
            self.ct = self.ct.wrapping_sub(1);
            if (a & 0x8000) != 0 {
                break;
            }
        }
        self.a = a;
        contexts[pos] = ((new_cx_index as i8) << 1) | (cx_mps as i8);
        d
    }
}
const GB_INDEX: usize = 0;
const GR_INDEX: usize = 1;
const IAID_INDEX: usize = 2;
const IADH_INDEX: usize = 3;
const IADW_INDEX: usize = 4;
const IAAI_INDEX: usize = 5;
const IARI_INDEX: usize = 6;
const IARDX_INDEX: usize = 7;
const IARDY_INDEX: usize = 8;
const IARDW_INDEX: usize = 9;
const IARDH_INDEX: usize = 10;
const IAEX_INDEX: usize = 11;
const IAFS_INDEX: usize = 12;
const IADT_INDEX: usize = 13;
const IAIT_INDEX: usize = 14;
const IADS_INDEX: usize = 15;
pub struct ContextCache {
    contexts: [Vec<i8>; 16],
    initialized: [bool; 16],
}
impl Default for ContextCache {
    fn default() -> Self {
        Self::new()
    }
}
impl ContextCache {
    pub fn new() -> Self {
        ContextCache {
            contexts: Default::default(),
            initialized: [false; 16],
        }
    }
    pub fn get_contexts(&mut self, id: &str) -> &mut Vec<i8> {
        let index = match id {
            "GB" => GB_INDEX,
            "GR" => GR_INDEX,
            "IAID" => IAID_INDEX,
            "IADH" => IADH_INDEX,
            "IADW" => IADW_INDEX,
            "IAAI" => IAAI_INDEX,
            "IARI" => IARI_INDEX,
            "IARDX" => IARDX_INDEX,
            "IARDY" => IARDY_INDEX,
            "IARDW" => IARDW_INDEX,
            "IARDH" => IARDH_INDEX,
            "IAEX" => IAEX_INDEX,
            "IAFS" => IAFS_INDEX,
            "IADT" => IADT_INDEX,
            "IAIT" => IAIT_INDEX,
            "IADS" => IADS_INDEX,
            _ => panic!("unknown context id: {}", id),
        };
        if !self.initialized[index] {
            self.contexts[index] = vec![0i8; 65536];
            self.initialized[index] = true;
        }
        &mut self.contexts[index]
    }
}
pub struct DecodingContext {
    pub data: Vec<u8>,
    pub start: usize,
    pub end: usize,
    pub context_cache: RefCell<ContextCache>,
    pub decoder: RefCell<Option<ArithmeticDecoder>>,
}
impl DecodingContext {
    pub fn new(data: Vec<u8>, start: usize, end: usize) -> Self {
        DecodingContext {
            data,
            start,
            end,
            context_cache: RefCell::new(ContextCache::new()),
            decoder: RefCell::new(None),
        }
    }
    pub fn get_decoder(&self) -> std::cell::RefMut<'_, ArithmeticDecoder> {
        let mut opt = self.decoder.borrow_mut();
        if opt.is_none() {
            *opt = Some(ArithmeticDecoder::new(&self.data[self.start..self.end], 0, self.end - self.start));
        }
        std::cell::RefMut::map(opt, |o| o.as_mut().unwrap())
    }
    pub fn get_contexts(&self, id: &str) -> std::cell::RefMut<'_, Vec<i8>> {
        std::cell::RefMut::map(self.context_cache.borrow_mut(), |c| c.get_contexts(id))
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
#[allow(clippy::too_many_arguments)]
fn decode_bitmap_template0(width: usize, height: usize, decoding_context: &mut DecodingContext) -> Result<Bitmap, Jbig2Error> {
    let mut decoder = decoding_context.get_decoder();
    let mut contexts = decoding_context.get_contexts("GB");
    let mut bitmap = Bitmap::new(width, height);
    const OLD_PIXEL_MASK: u16 = 0x7bf7;
    for i in 0..height {
        let mut context_label = 0u16;
        if i >= 2 {
            let row2_y = i - 2;
            context_label |= (bitmap.get_pixel(0, row2_y) as u16) << 13;
            context_label |= (bitmap.get_pixel(1, row2_y) as u16) << 12;
            context_label |= (bitmap.get_pixel(2, row2_y) as u16) << 11;
        }
        if i >= 1 {
            let row1_y = i - 1;
            context_label |= (bitmap.get_pixel(0, row1_y) as u16) << 7;
            context_label |= (bitmap.get_pixel(1, row1_y) as u16) << 6;
            context_label |= (bitmap.get_pixel(2, row1_y) as u16) << 5;
            context_label |= (bitmap.get_pixel(3, row1_y) as u16) << 4;
        }
        for j in 0..width {
            let pixel = decoder.read_bit(contexts.as_mut(), context_label as usize);
            bitmap.set_pixel(j, i, pixel);
            let row2_contrib = if i >= 2 && j + 3 < width { (bitmap.get_pixel(j + 3, i - 2) as u16) << 11 } else { 0 };
            let row1_contrib = if i >= 1 && j + 4 < width { (bitmap.get_pixel(j + 4, i - 1) as u16) << 4 } else { 0 };
            context_label = ((context_label & OLD_PIXEL_MASK) << 1) | row2_contrib | row1_contrib | (pixel as u16);
        }
    }
    Ok(bitmap)
}
#[allow(clippy::too_many_arguments)]
pub fn decode_bitmap(
    mmr: bool,
    width: usize,
    height: usize,
    template_index: usize,
    prediction: bool,
    skip: Option<&Bitmap>,
    at: Vec<(i8, i8)>,
    decoding_context: &mut DecodingContext,
) -> Result<Bitmap, Jbig2Error> {
    if mmr {
        let mut reader = crate::reader::Reader::new(
            decoding_context.data.clone(),
            decoding_context.start,
            decoding_context.end,
        );
        return decode_mmr_bitmap(&mut reader, width, height, false);
    }
    // Use optimized version for the most common case
    if template_index == 0 && skip.is_none() && !prediction && at.len() == 4 &&
        at[0].0 == 3 && at[0].1 == -1 &&
        at[1].0 == -3 && at[1].1 == -1 &&
        at[2].0 == 2 && at[2].1 == -2 &&
        at[3].0 == -2 && at[3].1 == -2 {
        return decode_bitmap_template0(width, height, decoding_context);
    }
    let useskip = skip.is_some();
    let template = get_coding_template(template_index).iter().cloned().chain(at).collect::<Vec<_>>();
    let mut template = template;
    template.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
    let template_length = template.len();
    let mut template_x = vec![0i8; template_length];
    let mut template_y = vec![0i8; template_length];
    let mut changing_template_entries = vec![];
    let mut reuse_mask = 0u16;
    let mut min_x = i8::MAX;
    let mut max_x = i8::MIN;
    let mut min_y = i8::MAX;
    for k in 0..template_length {
        template_x[k] = template[k].0;
        template_y[k] = template[k].1;
        min_x = min_x.min(template[k].0);
        max_x = max_x.max(template[k].0);
        min_y = min_y.min(template[k].1);
        if k < template_length - 1 && template[k].1 == template[k + 1].1 && template[k].0 == template[k + 1].0 - 1 {
            reuse_mask |= 1 << (template_length - 1 - k);
        } else {
            changing_template_entries.push(k);
        }
    }
    let changing_entries_length = changing_template_entries.len();
    let mut changing_template_x = vec![0i8; changing_entries_length];
    let mut changing_template_y = vec![0i8; changing_entries_length];
    let mut changing_template_bit = vec![0u16; changing_entries_length];
    for c in 0..changing_entries_length {
        let k = changing_template_entries[c];
        changing_template_x[c] = template[k].0;
        changing_template_y[c] = template[k].1;
        changing_template_bit[c] = 1 << (template_length - 1 - k);
    }
    let sbb_left = (-min_x) as usize;
    let sbb_top = (-min_y) as usize;
    let sbb_right = width - max_x as usize;
    let pseudo_pixel_context = REUSED_CONTEXTS[template_index];
    let mut bitmap = Bitmap::new(width, height);
    let mut decoder = decoding_context.get_decoder();
    let mut contexts = decoding_context.get_contexts("GB");
    let mut ltp = 0i32;
    for i in 0..height {
        if prediction {
            let sltp = decoder.read_bit(contexts.as_mut(), pseudo_pixel_context as usize) as i32;
            ltp ^= sltp;
            if ltp != 0 {
                let src_start = (i - 1) * bitmap.stride;
                let dst_start = i * bitmap.stride;
                let (before, after) = bitmap.data.split_at_mut(dst_start);
                let src_row = &before[src_start..];
                let dst_row = &mut after[0..bitmap.stride];
                dst_row.copy_from_slice(src_row);
                continue;
            }
        }
        for j in 0..width {
            if useskip && skip.unwrap().get_pixel(j, i) != 0 {
                continue;
            }
            let context_label = if j >= sbb_left && j < sbb_right && i >= sbb_top {
                let mut context_label = 0u16;
                context_label = (context_label << 1) & reuse_mask;
                for k in 0..changing_entries_length {
                    let i0 = i as i32 + changing_template_y[k] as i32;
                    let j0 = j as i32 + changing_template_x[k] as i32;
                    if i0 >= 0 && i0 < height as i32 && j0 >= 0 && j0 < width as i32 && bitmap.get_pixel(j0 as usize, i0 as usize) != 0 {
                        context_label |= changing_template_bit[k];
                    }
                }
                context_label
            } else {
                let mut context_label = 0u16;
                let mut shift = template_length - 1;
                for k in 0..template_length {
                    let j0 = j as i32 + template_x[k] as i32;
                    if j0 >= 0 && j0 < width as i32 {
                        let i0 = i as i32 + template_y[k] as i32;
                        if i0 >= 0 && i0 < height as i32 && bitmap.get_pixel(j0 as usize, i0 as usize) != 0 {
                            context_label |= 1 << shift;
                        }
                    }
                    shift -= 1;
                }
                context_label
            };
            let pixel = decoder.read_bit(contexts.as_mut(), context_label as usize);
            bitmap.set_pixel(j, i, pixel);
        }
    }
    Ok(bitmap)
}

// CCITT Group 4 (MMR) decoder implementation
struct CCITTFaxDecoder {
    data: Vec<u8>,
    position: usize,
    end: usize,
    black_is_1: bool,
}

impl CCITTFaxDecoder {
    fn new(data: Vec<u8>, start: usize, end: usize, _width: usize, _height: usize, black_is_1: bool, _end_of_block: bool) -> Self {
        CCITTFaxDecoder {
            data,
            position: start,
            end,
            black_is_1,
        }
    }

    fn read_next_char(&mut self) -> i32 {
        if self.position >= self.end {
            return -1; // EOF
        }
        let byte = self.data[self.position];
        self.position += 1;
        byte as i32
    }
}

fn decode_mmr_bitmap(input: &mut crate::reader::Reader, width: usize, height: usize, end_of_block: bool) -> Result<Bitmap, Jbig2Error> {
    // For now, implement a basic MMR decoder
    // This is a simplified implementation - full MMR decoding is quite complex
    let mut bitmap = Bitmap::new(width, height);

    // Create CCITT decoder
    let data = input.get_data().to_vec();
    let mut decoder = CCITTFaxDecoder::new(
        data,
        input.get_position(),
        input.get_end(),
        width,
        height,
        true, // black_is_1
        end_of_block,
    );

    let mut current_byte: i32 = 0;
    let mut eof = false;

    for y in 0..height {
        let mut shift = -1;
        for x in 0..width {
            if shift < 0 {
                current_byte = decoder.read_next_char();
                if current_byte == -1 {
                    current_byte = 0;
                    eof = true;
                }
                shift = 7;
            }
            let pixel = if decoder.black_is_1 {
                (current_byte >> shift) & 1
            } else {
                ((current_byte >> shift) & 1) ^ 1
            };
            bitmap.set_pixel(x, y, pixel as u8);
            shift -= 1;
        }
    }

    if end_of_block && !eof {
        // Read until EOFB is found (simplified)
        while decoder.read_next_char() != -1 {
            // Continue reading
        }
    }

    // Update input position
    input.set_position(decoder.position);

    Ok(bitmap)
}

#[derive(Clone)]
pub struct TextRegionParams {
    pub huffman: bool,
    pub refinement: bool,
    pub width: usize,
    pub height: usize,
    pub default_pixel_value: u8,
    pub number_of_symbol_instances: usize,
    pub strip_size: usize,
    pub input_symbols: Vec<Bitmap>,
    pub symbol_code_length: usize,
    pub transposed: bool,
    pub ds_offset: i32,
    pub reference_corner: usize,
    pub combination_operator: usize,
    pub log_strip_size: usize,
}

pub fn decode_text_region(
    params: &TextRegionParams,
    decoding_context: &mut DecodingContext,
) -> Result<Bitmap, Jbig2Error> {
    // Prepare bitmap
    let mut bitmap = Bitmap::new(params.width, params.height);
    if params.default_pixel_value != 0 {
        for y in 0..params.height {
            for x in 0..params.width {
                bitmap.set_pixel(x, y, 1);
            }
        }
    }

    let mut strip_t = -(decode_integer_context(decoding_context, "IADT")?.unwrap_or(0) as i32);
    let mut first_s = 0i32;
    let mut i = 0;

    while i < params.number_of_symbol_instances {
        let delta_t = decode_integer_context(decoding_context, "IADT")?.unwrap_or(0);
        strip_t += delta_t as i32;

        let delta_first_s = decode_integer_context(decoding_context, "IAFS")?.unwrap_or(0);
        first_s += delta_first_s as i32;
        let mut current_s = first_s;

        loop {
            let mut current_t = 0i32;
            if params.strip_size > 1 {
                current_t = decode_integer_context(decoding_context, "IAIT")?.unwrap_or(0) as i32;
            }
            let t = (params.strip_size as i32) * strip_t + current_t;
            let symbol_id = decode_iaid_context(decoding_context, params.symbol_code_length)?;
            let symbol_id = symbol_id as usize;
            if symbol_id >= params.input_symbols.len() {
                return Err(Jbig2Error::new("invalid symbol id"));
            }

            let symbol_bitmap = &params.input_symbols[symbol_id];
            let symbol_width = symbol_bitmap.width;
            let symbol_height = symbol_bitmap.height;

            let apply_refinement = if params.refinement {
                decode_integer_context(decoding_context, "IARI")?.unwrap_or(0) != 0
            } else {
                false
            };

            let (final_symbol_width, final_symbol_height, final_symbol_bitmap) = if apply_refinement {
                // For now, skip refinement - would need additional parameters
                (symbol_width, symbol_height, symbol_bitmap.clone())
            } else {
                (symbol_width, symbol_height, symbol_bitmap.clone())
            };

            let increment = if !params.transposed {
                if params.reference_corner > 1 {
                    current_s += final_symbol_width as i32 - 1;
                    final_symbol_width as i32 - 1
                } else {
                    final_symbol_width as i32 - 1
                }
            } else {
                final_symbol_height as i32 - 1
            };

            let offset_t = t - if (params.reference_corner & 1) != 0 { 0 } else { final_symbol_height as i32 - 1 };
            let offset_s = current_s - if (params.reference_corner & 2) != 0 { final_symbol_width as i32 - 1 } else { 0 };

            // Draw the symbol
            if params.transposed {
                for s2 in 0..final_symbol_height {
                    let row = offset_s + s2 as i32;
                    if row < 0 || row >= params.height as i32 {
                        continue;
                    }
                    for t2 in 0..final_symbol_width {
                        let col = offset_t + t2 as i32;
                        if col >= 0 && col < params.width as i32 {
                            let src_pixel = final_symbol_bitmap.get_pixel(t2, s2);
                            let dst_pixel = bitmap.get_pixel(col as usize, row as usize);
                            let new_pixel = match params.combination_operator {
                                0 => src_pixel, // OR
                                2 => dst_pixel ^ src_pixel, // XOR
                                _ => return Err(Jbig2Error::new("unsupported combination operator")),
                            };
                            bitmap.set_pixel(col as usize, row as usize, new_pixel);
                        }
                    }
                }
            } else {
                for t2 in 0..final_symbol_height {
                    let row = offset_t + t2 as i32;
                    if row < 0 || row >= params.height as i32 {
                        continue;
                    }
                    for s2 in 0..final_symbol_width {
                        let col = offset_s + s2 as i32;
                        if col >= 0 && col < params.width as i32 {
                            let src_pixel = final_symbol_bitmap.get_pixel(s2, t2);
                            let dst_pixel = bitmap.get_pixel(col as usize, row as usize);
                            let new_pixel = match params.combination_operator {
                                0 => src_pixel, // OR
                                2 => dst_pixel ^ src_pixel, // XOR
                                _ => return Err(Jbig2Error::new("unsupported combination operator")),
                            };
                            bitmap.set_pixel(col as usize, row as usize, new_pixel);
                        }
                    }
                }
            }

            i += 1;
            let delta_s = decode_integer_context(decoding_context, "IADS")?;
            if delta_s.is_none() {
                break; // OOB
            }
            current_s += increment + delta_s.unwrap() as i32 + params.ds_offset;
        }
    }

    Ok(bitmap)
}

#[derive(Clone)]
pub struct SymbolDictionaryParams {
    pub huffman: bool,
    pub refinement: bool,
    pub symbols: Vec<Bitmap>,
    pub number_of_new_symbols: usize,
    pub number_of_exported_symbols: usize,
    pub template_index: usize,
    pub at: Vec<(i8, i8)>,
    pub refinement_template_index: usize,
    pub refinement_at: Vec<(i8, i8)>,
}

pub fn decode_symbol_dictionary(
    params: &SymbolDictionaryParams,
    decoding_context: &mut DecodingContext,
) -> Result<Vec<Bitmap>, Jbig2Error> {
    let mut new_symbols = Vec::new();
    let mut current_height = 0i32;

    let _symbol_code_length = crate::core_utils::log2((params.symbols.len() + params.number_of_new_symbols) as u32);



    while new_symbols.len() < params.number_of_new_symbols {
        let delta_height = decode_integer_context(decoding_context, "IADH")?.unwrap_or(0);
        current_height += delta_height as i32;

        let mut current_width = 0i32;
        while current_width >= 0 {
            let delta_width = decode_integer_context(decoding_context, "IADW")?;
            if delta_width.is_none() {
                break; // OOB
            }
            current_width += delta_width.unwrap() as i32;

            if params.refinement {
                // For now, skip refinement
                return Err(Jbig2Error::new("refinement not implemented"));
            } else {
                // Direct-coded symbol bitmap - simplified implementation
                let bitmap = Bitmap::new(current_width as usize, current_height as usize);
                // TODO: Implement proper bitmap decoding here
                new_symbols.push(bitmap);
            }
        }
    }

    // Exported symbols
    let mut exported_symbols = Vec::new();
    let mut flags = Vec::new();
    let total_symbols_length = params.symbols.len() + params.number_of_new_symbols;
    let mut current_flag = false;

    while flags.len() < total_symbols_length {
        let run_length = decode_integer_context(decoding_context, "IAEX")?;
        let run_length = run_length.unwrap_or(0) as usize;
        for _ in 0..run_length {
            flags.push(current_flag);
        }
        current_flag = !current_flag;
    }

    for (i, &flag) in flags.iter().enumerate() {
        if flag {
            if i < params.symbols.len() {
                exported_symbols.push(params.symbols[i].clone());
            } else {
                exported_symbols.push(new_symbols[i - params.symbols.len()].clone());
            }
        }
    }

    Ok(exported_symbols)
}

#[derive(Clone)]
pub struct PatternDictionaryParams {
    pub mmr: bool,
    pub pattern_width: usize,
    pub pattern_height: usize,
    pub max_pattern_index: usize,
    pub template: usize,
}

pub fn decode_pattern_dictionary(
    params: &PatternDictionaryParams,
    decoding_context: &mut DecodingContext,
) -> Result<Vec<Bitmap>, Jbig2Error> {
    let at = if !params.mmr {
        let mut at_vec = vec![(-(params.pattern_width as i8), 0i8)];
        if params.template == 0 {
            at_vec.extend(vec![
                (-3i8, -1i8),
                (2i8, -2i8),
                (-2i8, -2i8),
            ]);
        }
        at_vec
    } else {
        vec![]
    };

    let collective_width = (params.max_pattern_index + 1) * params.pattern_width;
    let collective_bitmap = decode_bitmap(
        params.mmr,
        collective_width,
        params.pattern_height,
        params.template,
        false, // prediction
        None, // skip
        at,
        decoding_context,
    )?;

    // Divide collective bitmap into patterns
    let mut patterns = Vec::new();
    for i in 0..=params.max_pattern_index {
        let mut pattern_bitmap = Vec::new();
        let x_min = params.pattern_width * i;
        let x_max = x_min + params.pattern_width;
        for y in 0..params.pattern_height {
            let row = collective_bitmap.data[y * collective_bitmap.stride..(y + 1) * collective_bitmap.stride]
                .iter()
                .skip(x_min / 8)
                .take(x_max.div_ceil(8) - x_min / 8)
                .cloned()
                .collect::<Vec<_>>();
            // For simplicity, create a new bitmap with the pattern
            // This is a simplified implementation
            let mut pattern = Bitmap::new(params.pattern_width, params.pattern_height);
            for py in 0..params.pattern_height {
                for px in 0..params.pattern_width {
                    let src_x = x_min + px;
                    let bit_index = 7 - (src_x & 7);
                    let byte_index = src_x >> 3;
                    if byte_index < row.len() {
                        let pixel = (row[byte_index] >> bit_index) & 1;
                        pattern.set_pixel(px, py, pixel);
                    }
                }
            }
            pattern_bitmap.push(pattern);
        }
        if !pattern_bitmap.is_empty() {
            patterns.push(pattern_bitmap[0].clone());
        }
    }

    Ok(patterns)
}

#[allow(clippy::too_many_arguments)]
#[derive(Clone)]
pub struct HalftoneRegionParams {
    pub mmr: bool,
    pub patterns: Vec<Bitmap>,
    pub template: usize,
    pub region_width: usize,
    pub region_height: usize,
    pub default_pixel_value: u8,
    pub enable_skip: bool,
    pub combination_operator: usize,
    pub grid_width: usize,
    pub grid_height: usize,
    pub grid_offset_x: i32,
    pub grid_offset_y: i32,
    pub grid_vector_x: i16,
    pub grid_vector_y: i16,
}

pub fn decode_halftone_region(
    params: &HalftoneRegionParams,
    decoding_context: &mut DecodingContext,
) -> Result<Bitmap, Jbig2Error> {
    if params.enable_skip {
        return Err(Jbig2Error::new("skip is not supported"));
    }
    if params.combination_operator != 0 {
        return Err(Jbig2Error::new("only OR combination operator is supported"));
    }

    // Prepare bitmap
    let mut region_bitmap = Bitmap::new(params.region_width, params.region_height);
    if params.default_pixel_value != 0 {
        for y in 0..params.region_height {
            for x in 0..params.region_width {
                region_bitmap.set_pixel(x, y, 1);
            }
        }
    }

    let number_of_patterns = params.patterns.len();
    if number_of_patterns == 0 {
        return Ok(region_bitmap);
    }

    let pattern0 = &params.patterns[0];
    let _pattern_width = pattern0.width;
    let pattern_height = pattern0.height;
    let bits_per_value = crate::core_utils::log2(number_of_patterns as u32) as usize;

    let at = if !params.mmr {
        let mut at_vec = vec![(if params.template <= 1 { 3i8 } else { 2i8 }, -1i8)];
        if params.template == 0 {
            at_vec.extend(vec![
                (-3i8, -1i8),
                (2i8, -2i8),
                (-2i8, -2i8),
            ]);
        }
        at_vec
    } else {
        vec![]
    };

    // Gray-scale bit planes
    let mut gray_scale_bit_planes = Vec::new();
    for _ in (0..bits_per_value).rev() {
        let bitmap = decode_bitmap(
            params.mmr,
            params.grid_width,
            params.grid_height,
            params.template,
            false, // prediction
            None, // skip
            at.clone(),
            decoding_context,
        )?;
        gray_scale_bit_planes.push(bitmap);
    }

    // Render patterns
    for mg in 0..params.grid_height {
        for ng in 0..params.grid_width {
            let mut bit = 0u8;
            let mut pattern_index = 0usize;
            for j in (0..bits_per_value).rev() {
                let plane_bit = gray_scale_bit_planes[j].get_pixel(ng, mg);
                bit ^= plane_bit;
                pattern_index |= (bit as usize) << j;
            }
            if pattern_index >= params.patterns.len() {
                continue;
            }
            let pattern_bitmap = &params.patterns[pattern_index];
            let x = (params.grid_offset_x + mg as i32 * params.grid_vector_y as i32 + ng as i32 * params.grid_vector_x as i32) >> 8;
            let y = (params.grid_offset_y + mg as i32 * params.grid_vector_x as i32 - ng as i32 * params.grid_vector_y as i32) >> 8;

            // Draw pattern
            if x >= 0 && x + pattern_bitmap.width as i32 <= params.region_width as i32 &&
               y >= 0 && y + pattern_bitmap.height as i32 <= params.region_height as i32 {
                for i in 0..pattern_bitmap.height {
                    for j in 0..pattern_bitmap.width {
                        let src_pixel = pattern_bitmap.get_pixel(j, i);
                        let dst_pixel = region_bitmap.get_pixel((x + j as i32) as usize, (y + i as i32) as usize);
                        let new_pixel = src_pixel | dst_pixel; // OR
                        region_bitmap.set_pixel((x + j as i32) as usize, (y + i as i32) as usize, new_pixel);
                    }
                }
            } else {
                // Handle partial patterns at edges
                for i in 0..pattern_height {
                    let region_y = y + i as i32;
                    if region_y < 0 || region_y >= params.region_height as i32 {
                        continue;
                    }
                    for j in 0..pattern_bitmap.width {
                        let region_x = x + j as i32;
                        if region_x >= 0 && region_x < params.region_width as i32 {
                            let src_pixel = pattern_bitmap.get_pixel(j, i);
                            let dst_pixel = region_bitmap.get_pixel(region_x as usize, region_y as usize);
                            let new_pixel = src_pixel | dst_pixel; // OR
                            region_bitmap.set_pixel(region_x as usize, region_y as usize, new_pixel);
                        }
                    }
                }
            }
        }
    }

    Ok(region_bitmap)
}

#[allow(clippy::too_many_arguments)]
pub fn decode_refinement(
    width: usize,
    height: usize,
    template_index: usize,
    reference_bitmap: &Bitmap,
    offset_x: i32,
    offset_y: i32,
    prediction: bool,
    at: Vec<(i8, i8)>,
    decoding_context: &mut DecodingContext,
) -> Result<Bitmap, Jbig2Error> {
    if prediction {
        return Err(Jbig2Error::new("prediction is not supported"));
    }
    let mut coding_template = get_refinement_template(template_index).coding;
    if template_index == 0 {
        coding_template.push(at[0]);
    }
    let coding_template_length = coding_template.len();
    let coding_template_x = coding_template.iter().map(|&(x, _)| x as i32).collect::<Vec<_>>();
    let coding_template_y = coding_template.iter().map(|&(_, y)| y as i32).collect::<Vec<_>>();
    let mut reference_template = get_refinement_template(template_index).reference;
    if template_index == 0 {
        reference_template.push(at[1]);
    }
    let reference_template_length = reference_template.len();
    let reference_template_x = reference_template.iter().map(|&(x, _)| x as i32).collect::<Vec<_>>();
    let reference_template_y = reference_template.iter().map(|&(_, y)| y as i32).collect::<Vec<_>>();
    let reference_width = reference_bitmap.width;
    let reference_height = reference_bitmap.height;
    let mut contexts = decoding_context.get_contexts("GR");
    let mut decoder = decoding_context.get_decoder();
    let mut bitmap = Bitmap::new(width, height);
    for i in 0..height {
        for j in 0..width {
            let mut context_label = 0u16;
            for k in 0..coding_template_length {
                let i0 = i as i32 + coding_template_y[k];
                let j0 = j as i32 + coding_template_x[k];
                if i0 < 0 || j0 < 0 || j0 >= width as i32 {
                    context_label <<= 1;
                } else {
                    context_label = (context_label << 1) | (bitmap.get_pixel(j0 as usize, i0 as usize) as u16);
                }
            }
            for k in 0..reference_template_length {
                let i0 = i as i32 + reference_template_y[k] - offset_y;
                let j0 = j as i32 + reference_template_x[k] - offset_x;
                if i0 < 0 || i0 >= reference_height as i32 || j0 < 0 || j0 >= reference_width as i32 {
                    context_label <<= 1;
                } else {
                    context_label = (context_label << 1) | (reference_bitmap.get_pixel(j0 as usize, i0 as usize) as u16);
                }
            }
            let pixel = decoder.read_bit(contexts.as_mut(), context_label as usize);
            bitmap.set_pixel(j, i, pixel);
        }
    }
    Ok(bitmap)
}