use crate::arithmetic::ArithmeticDecoder;
use crate::arithmetic::contexts::ContextCache;
use crate::arithmetic::contexts::DecodingContext;
use crate::common::error::Jbig2Error;

/// Decode a signed integer using arithmetic decoding contexts.
pub fn decode_integer(
    context_cache: &mut ContextCache,
    procedure: &str,
    decoder: &mut ArithmeticDecoder<'_>,
) -> Result<Option<i32>, Jbig2Error> {
    let contexts = context_cache.get_contexts(procedure);
    let mut prev = 1;

    let read_bits = |length: u32,
                     contexts: &mut Vec<i8>,
                     prev: &mut usize,
                     decoder: &mut ArithmeticDecoder<'_>|
     -> Result<u32, Jbig2Error> {
        let mut v = 0;
        for _ in 0..length {
            let bit = decoder.read_bit(contexts, *prev)? as usize;
            *prev = if *prev < 256 {
                (*prev << 1) | bit
            } else {
                (((*prev << 1) | bit) & 511) | 256
            };
            v = (v << 1) | bit as u32;
        }
        Ok(v)
    };

    let sign = read_bits(1, contexts, &mut prev, decoder)?;

    let (n_tail, offset) = if read_bits(1, contexts, &mut prev, decoder)? != 0 {
        if read_bits(1, contexts, &mut prev, decoder)? != 0 {
            if read_bits(1, contexts, &mut prev, decoder)? != 0 {
                if read_bits(1, contexts, &mut prev, decoder)? != 0 {
                    if read_bits(1, contexts, &mut prev, decoder)? != 0 {
                        (32, 4436)
                    } else {
                        (12, 340)
                    }
                } else {
                    (8, 84)
                }
            } else {
                (6, 20)
            }
        } else {
            (4, 4)
        }
    } else {
        (2, 0)
    };

    let mut value = read_bits(n_tail, contexts, &mut prev, decoder)?;
    let offset_u32 = offset as u32;
    if value > (i32::MAX as u32).saturating_sub(offset_u32) {
        value = i32::MAX as u32;
    } else {
        value = value.saturating_add(offset_u32);
    }

    if sign != 0 && value == 0 {
        return Ok(None);
    }

    let signed_value = if sign == 0 {
        value as i32
    } else {
        -(value as i32)
    };
    Ok(Some(signed_value))
}

/// Decode an IAID value using arithmetic coding contexts.
pub fn decode_iaid(
    context_cache: &mut ContextCache,
    decoder: &mut ArithmeticDecoder<'_>,
    code_length: usize,
) -> Result<u32, Jbig2Error> {
    let contexts = context_cache.get_contexts("IAID");
    let mut prev = 1;
    for _ in 0..code_length {
        let bit = decoder.read_bit(contexts.as_mut(), prev)?;
        prev = (prev << 1) | bit as usize;
    }
    if code_length < 31 {
        Ok((prev & ((1 << code_length) - 1)) as u32)
    } else {
        Ok((prev & 0x7fffffff) as u32)
    }
}

/// Decode a signed integer from a shared decoding context.
pub fn decode_integer_context(
    decoding_context: &mut DecodingContext<'_>,
    procedure: &str,
) -> Result<Option<i32>, Jbig2Error> {
    let mut context_cache = decoding_context.context_cache.borrow_mut();
    // Initialize the decoder lazily to preserve state across calls.
    let mut decoder = decoding_context.get_decoder();
    decode_integer(&mut context_cache, procedure, &mut decoder)
}

/// Decode an IAID value from a shared decoding context.
pub fn decode_iaid_context(
    decoding_context: &mut DecodingContext<'_>,
    code_length: usize,
) -> Result<u32, Jbig2Error> {
    let mut context_cache = decoding_context.context_cache.borrow_mut();
    // Initialize the decoder lazily to preserve state across calls.
    let mut decoder = decoding_context.get_decoder();
    decode_iaid(&mut context_cache, &mut decoder, code_length)
}

/// Decode a signed integer using either Huffman or arithmetic coding.
pub fn decode_i32_huffman_or_arith<F>(
    huffman: bool,
    huffman_decode: F,
    arith_proc: &str,
    decoding_context: &mut DecodingContext<'_>,
) -> Result<i32, Jbig2Error>
where
    F: FnOnce() -> Result<i32, Jbig2Error>,
{
    if huffman {
        huffman_decode()
    } else {
        decode_integer_context(decoding_context, arith_proc).map(|opt| opt.unwrap_or(0))
    }
}

/// Decode an optional signed integer using either Huffman or arithmetic coding.
pub fn decode_option_i32_huffman_or_arith<F>(
    huffman: bool,
    huffman_decode: F,
    arith_proc: &str,
    decoding_context: &mut DecodingContext<'_>,
) -> Result<Option<i32>, Jbig2Error>
where
    F: FnOnce() -> Result<i32, Jbig2Error>,
{
    if huffman {
        huffman_decode().map(Some)
    } else {
        decode_integer_context(decoding_context, arith_proc)
    }
}

/// Decode an unsigned integer using either Huffman or arithmetic coding.
pub fn decode_u32_huffman_or_arith<F>(
    huffman: bool,
    huffman_decode: F,
    arith_code_length: usize,
    decoding_context: &mut DecodingContext<'_>,
) -> Result<u32, Jbig2Error>
where
    F: FnOnce() -> Result<i32, Jbig2Error>,
{
    if huffman {
        huffman_decode().map(|v| v as u32)
    } else {
        decode_iaid_context(decoding_context, arith_code_length)
    }
}
