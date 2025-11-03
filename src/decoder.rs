use crate::error::Jbig2Error;
use crate::bitmap::ContextCache;


// Annex A. Arithmetic Integer Decoding Procedure
// A.2 Procedure for decoding values
pub fn decode_integer(context_cache: &mut ContextCache, procedure: &str, decoder: &mut crate::bitmap::ArithmeticDecoder) -> Result<i32, Jbig2Error> {
    let contexts = context_cache.get_contexts(procedure);
    let mut prev = 1;

    let read_bits = |length: u32, contexts: &mut Vec<i8>, prev: &mut usize, decoder: &mut crate::bitmap::ArithmeticDecoder| -> Result<u32, Jbig2Error> {
        let mut v = 0;
        for _ in 0..length {
            let bit = decoder.read_bit(contexts, *prev) as usize;
            *prev = if *prev < 256 { (*prev << 1) | bit } else { (((*prev << 1) | bit) & 511) | 256 };
            v = (v << 1) | bit as u32;
        }
        Ok(v)
    };

    let sign = read_bits(1, contexts, &mut prev, decoder)?;
    // The nested ternary from JS
    let value = if read_bits(1, contexts, &mut prev, decoder)? != 0 {
        if read_bits(1, contexts, &mut prev, decoder)? != 0 {
            if read_bits(1, contexts, &mut prev, decoder)? != 0 {
                if read_bits(1, contexts, &mut prev, decoder)? != 0 {
                    if read_bits(1, contexts, &mut prev, decoder)? != 0 {
                        read_bits(32, contexts, &mut prev, decoder)? + 4436
                    } else {
                        read_bits(12, contexts, &mut prev, decoder)? + 340
                    }
                } else {
                    read_bits(8, contexts, &mut prev, decoder)? + 84
                }
            } else {
                read_bits(6, contexts, &mut prev, decoder)? + 20
            }
        } else {
            read_bits(4, contexts, &mut prev, decoder)? + 4
        }
    } else {
        read_bits(2, contexts, &mut prev, decoder)?
    };

    let signed_value = if sign == 0 { value as i32 } else if value > 0 { -(value as i32) } else { 0 };

    Ok(signed_value)
}

// A.3 The IAID decoding procedure
pub fn decode_iaid(context_cache: &mut ContextCache, decoder: &mut crate::bitmap::ArithmeticDecoder, code_length: usize) -> Result<u32, Jbig2Error> {
    let contexts = context_cache.get_contexts("IAID");
    let mut prev = 1;
    for _ in 0..code_length {
        let bit = decoder.read_bit(contexts, prev);
        prev = (prev << 1) | bit as usize;
    }
    if code_length < 31 {
        Ok((prev & ((1 << code_length) - 1)) as u32)
    } else {
        Ok((prev & 0x7fffffff) as u32)
    }
}