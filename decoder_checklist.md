# JBIG2 Decoder Checklist

The following is a list of items to check in the `jbig2-rs` decoder implementation to address the reported decoding issues.

## 1. MMR Decoder (`src/decode/decode_mmr.rs`)
- [ ] **EOFB Handling**: Verify that `read_mode_code` correctly identifies the End of File Block (EOFB) code (`0x001001` - 24 bits) when `end_of_block` is true. Ensure it doesn't consume bits speculatively that might belong to the next segment if EOFB is not found.
- [ ] **Error Tolerance**: Confirm that `decode_2d_line` gracefully handles invalid mode codes by terminating the line decoding and filling the remainder with 0 (White), rather than returning a hard error. This mimics the behavior of `jbig2dec`.
- [ ] **Zero Dimensions**: Ensure `decode_mmr_bitmap` handles `width=0` or `height=0` without panicking or entering infinite loops.
- [ ] **Run Length Decoding**: Check `decode_run_length` for correct handling of "make-up" codes and "terminating" codes, ensuring they are summed correctly.

## 2. Arithmetic Decoder (`src/arithmetic.rs`)
- [ ] **Initialization**: Verify that `ArithmeticDecoder::new` correctly consumes the first 2 bytes of the stream to initialize the `A`, `C`, and `CT` registers as per the spec (Annex E).
- [ ] **Byte Consumption**: Check that `read_byte` handles `0xFF` stuffing correctly (skipping the `0x00` stuffer byte).
- [ ] **Renormalization**: Ensure `renorm_d` correctly shifts `A` and `C` and reads new bytes when needed.
- [ ] **Context Initialization**: Verify that context tables (IAx, IADW, etc.) are initialized to the correct sizes and zeroed out.

## 3. Symbol Dictionary (`src/decode/decode_symbol.rs`)
- [ ] **Collective Bitmap**: Check if the collective bitmap (containing all symbols) is decoded correctly using either MMR or Arithmetic coding based on the flags.
- [ ] **Symbol Splitting**: Verify the logic for splitting the collective bitmap into individual glyphs. Ensure the `x` coordinates and widths are calculated correctly.
- [ ] **Refinement**: If refinement is used, check that the refinement bitmap is correctly combined with the base symbol.

## 4. Text Region (`src/decode/decode_text.rs`)
- [ ] **Huffman Tables**: Verify that standard Huffman tables (B.1 to B.15) are implemented correctly in `src/huffman/standard_tables.rs`.
- [ ] **Table Selection**: Check that the correct Huffman table is selected based on the segment flags (e.g., `SBHUFFFS`, `SBHUFFRDX`, etc.).
- [ ] **Refinement**: Verify mixed Huffman/Arithmetic refinement decoding.
- [ ] **Striping**: Check if "striping" (decoding in strips) is handled correctly if applicable.

## 5. Bitmap Operations (`src/bitmap.rs`)
- [ ] **Stride**: Ensure that the bitmap stride (bytes per row) is calculated correctly (usually `(width + 7) / 8`).
- [ ] **Combination Operators**: Verify that all combination operators (OR, AND, XOR, XNOR, REPLACE) are implemented correctly, especially for edge cases (negative coordinates, out of bounds).
- [ ] **Raster Order**: Confirm that bits are packed MSB first (or LSB first as per spec - JBIG2 is usually MSB first).

## 6. General
- [ ] **Segment Parsing**: Verify that segment headers are parsed correctly, especially the `Retain Bits` and `Page Association` fields.
- [ ] **Data Buffering**: Ensure that the `Reader` does not read past the end of the available data for a segment.

---

**Note on Spec File**:
I have located the spec file at `research/jbig2_spec.pdf`. As I cannot read PDF files directly, I have based this checklist on general JBIG2 knowledge, the codebase structure, and common issues found in similar implementations. If you can provide a text version or specific sections of the spec, I can review them more detailedly.
