# JBIG2Dec Investigation Walkthrough

## Objective
Investigate why the Rust decoder `jbig2-rs` fails to parse `minimal_valid.jb2` correctly, getting 100x100 dimensions instead of the correct 1728x2339.

## What I Investigated

I performed a comprehensive analysis of the jbig2dec reference implementation focusing on:

1. **Segment header parsing** - How jbig2dec reads segment headers
2. **Extension segment handling** - What it does with type 62 segments  
3. **Sequential reading** - How it moves from segment to segment
4. **Page info parsing** - Where dimensions are extracted

## Key Files Analyzed

### [jbig2_segment.c](file:///home/jmaggi/projects/jbig2dec/jbig2_segment.c)

**`jbig2_parse_segment_header()`** (lines 42-138)
- Parses variable-length segment headers
- Returns header size via output parameter
- Key formula: `header_size = base + ref_segments_size + pa_size + 4`

**`jbig2_parse_extension_segment()`** (lines 209-248)
- Reads first 4 bytes of extension data (extension type)
- Logs and **ignores** the data
- Returns 0 (success) - no special processing

### [jbig2.c](file:///home/jmaggi/projects/jbig2dec/jbig2.c)

**`jbig2_data_in()`** (lines 230-436)
- Main parsing loop for sequential files
- Two-phase parsing: header then body
- Advances buffer: `buf_rd_ix += header_size`, then `buf_rd_ix += data_length`

### [jbig2_page.c](file:///home/jmaggi/projects/jbig2dec/jbig2_page.c)

**`jbig2_page_info()`** (lines 62-168)
- Reads page dimensions from segment data
- Width at offset +0 (4 bytes big-endian)
- Height at offset +4 (4 bytes big-endian)

## Critical Findings

### ✓ Variable Header Size

The segment header size is **NOT fixed**. It depends on:

| Component | Size | Determined By |
|-----------|------|---------------|
| Segment number | 4 bytes | Always |
| Flags | 1 byte | Always |
| Ref count | 1 or 4+ bytes | If (rtscarf & 0xe0) == 0xe0 |
| Retention flags | 0-N bytes | (ref_count + 1) / 8 |
| Ref segments | ref_count × ref_size | ref_size based on segment number |
| Page association | 1 or 4 bytes | flags & 0x40 |
| Data length | 4 bytes | Always |

**Formula**:
```python
header_size = 4 + 1 + rtscarf_size + ref_count * ref_size + pa_size + 4
```

### ✓ Extension Segments Are Ignored

jbig2dec does **NOT**:
- Parse embedded structures in extension data
- Search for hidden segments
- Process comment contents

It simply:
1. Reads 4-byte extension type
2. Logs a message
3. Skips the entire data region

### ✓ Next Segment Calculation

The formula is straightforward:
```
next_segment_offset = current_offset + header_size + data_length
```

Both `header_size` and `data_length` come from parsing the header.

### ✓ Data Length Field is Always 4 Bytes

No flags affect the data_length field size - it's always 4 bytes (32-bit big-endian).

The flags byte only affects:
- Bit 0-5: Segment type
- Bit 6: Page association size (0=1 byte, 1=4 bytes)  
- Bit 7: Deferred/immediate flag

## The Bug in Your Understanding

You stated:
> Extension data starts at `0x18`, should end at `0x18 + 104 = 0x80`

This assumes:
- File header ends at 0x0c (13 bytes)
- Segment 0 header starts at 0x0d
- Segment 0 header is 11 bytes (0x0d to 0x18)
- Segment 0 data is at 0x18

**But wait!** The header size depends on the actual header fields. If the header has:
- Referred-to segments
- 4-byte page association

Then the header could be longer than 11 bytes!

### Example Calculation

```
Offset 0x0d: Segment 0 header start
  +0 to +3: Segment number = 0
  +4: Flags = 0x3e
  +5: rtscarf (determines referred-to count)
  
If rtscarf = 0x00 (0 referred-to segments):
  +6: Page association (1 byte if flags & 0x40 == 0)
  +7 to +10: Data length (4 bytes)
  Header size = 11 bytes
  
If rtscarf = 0x20 (1 referred-to segment):
  +6: Referred-to segment 0 (1 byte, since seg_num ≤ 256)
  +7: Page association (1 byte)
  +8 to +11: Data length (4 bytes)
  Header size = 12 bytes
```

**The actual header size changes where the data starts!**

## Solution for Your Rust Decoder

1. **Implement exact header parsing logic** from `jbig2_parse_segment_header()`
   - Handle short vs long referred-to count form
   - Calculate referred-to segment size based on THIS segment's number
   - Read page association based on flags bit 6
   - Always read 4 bytes for data_length

2. **Use the Python script** I created (`jbig2_parser.py`) to trace your actual file:
   ```bash
   python3 jbig2_parser.py tests/resources/minimal_valid.jb2
   ```

3. **Compare** the output with `jbig2dec -v minimal_valid.jb2`

4. **Verify** the exact byte offsets match

## Deliverables Created

### [jbig2dec_analysis.md](file:///home/jmaggi/.gemini/antigravity/brain/f9a32c35-88df-42b6-9179-197cd4c621ab/jbig2dec_analysis.md)
Complete analysis answering all 6 questions with:
- Annotated source code from jbig2dec
- Byte-by-byte parsing explanations
- Segment iteration algorithm
- Data length field specification

### [jbig2_parser.py](file:///home/jmaggi/.gemini/antigravity/brain/f9a32c35-88df-42b6-9179-197cd4c621ab/jbig2_parser.py)
Python script that replicates jbig2dec's exact parsing logic:
- Parses segment headers with all variations
- Calculates header sizes correctly
- Shows segment types and data ranges
- Extracts page dimensions

## Next Steps

1. Run the Python script on your `minimal_valid.jb2` file
2. Compare the output with your Rust decoder's behavior
3. Fix the header size calculation in your decoder
4. Verify that segment offsets match jbig2dec exactly

The issue is almost certainly in how you calculate the segment header size. Once that's fixed, your decoder should find segment 1 at the correct offset and extract the 1728x2339 dimensions successfully.
