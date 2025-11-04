use std::cell::RefCell;

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
    pub decoder: RefCell<Option<crate::arithmetic::ArithmeticDecoder>>,
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

    pub fn get_decoder(&self) -> std::cell::RefMut<'_, crate::arithmetic::ArithmeticDecoder> {
        let mut opt = self.decoder.borrow_mut();
        if opt.is_none() {
            *opt = Some(crate::arithmetic::ArithmeticDecoder::new(
                &self.data[self.start..self.end],
            ));
        }
        std::cell::RefMut::map(opt, |o| o.as_mut().unwrap())
    }

    pub fn get_contexts(&self, id: &str) -> std::cell::RefMut<'_, Vec<i8>> {
        std::cell::RefMut::map(self.context_cache.borrow_mut(), |c| c.get_contexts(id))
    }
}
