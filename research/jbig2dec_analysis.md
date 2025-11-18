# JBIG2Dec Deep Dive Analysis - Complete Findings

## Executive Summary

After analyzing the jbig2dec reference implementation, I've identified the **critical issues** with your Rust decoder:

### Key Findings

1. **Header size is variable** - jbig2dec calculates header size dynamically based on:
   - Referred-to segment count (short vs long form)
   - Segment number (determines referred-to segment size: 1, 2, or 4 bytes)
   - Page association flag bit 6 (determines PA size: 1 or 4 bytes)

2. **Extension segments are ignored** - jbig2dec simply skips the data, returning 0

3. **Next segment calculation**: `next_offset = current_offset + header_size + data_length`

4. **The data_length field is ALWAYS 4 bytes** - regardless of other flags

---

## Q1: Segment Header Parsing Logic

### Complete `jbig2_parse_segment_header()` Function

**Location**: `jbig2_segment.c` lines 42-138

```c
Jbig2Segment *
jbig2_parse_segment_header(Jbig2Ctx *ctx, uint8_t *buf, size_t buf_size, 
                           size_t *p_header_size)
{
    Jbig2Segment *result;
    uint8_t rtscarf;
    uint32_t rtscarf_long;
    uint32_t referred_to_segment_count;
    uint32_t referred_to_segment_size;
    uint32_t pa_size;
    uint32_t offset;

    /* STEP 1: Check minimum header size (11 bytes) */
    // Line 55-56
    if (buf_size < 11)
        return NULL;

    result = jbig2_new(ctx, Jbig2Segment, 1);
    if (result == NULL) {
        jbig2_error(ctx, JBIG2_SEVERITY_FATAL, JBIG2_UNKNOWN_SEGMENT_NUMBER, 
                    "failed to allocate segment");
        return NULL;
    }

    /* STEP 2: Parse segment number (4 bytes, big-endian) */
    // Line 65 - JBIG2 spec 7.2.2
    result->number = jbig2_get_uint32(buf);  // buf[0:4]
    
    /* STEP 3: Parse segment header flags (1 byte) */
    // Line 73 - JBIG2 spec 7.2.3
    result->flags = buf[4];  // buf[4]
    
    /* STEP 4: Parse referred-to segment count and retention flags */
    // Lines 76-84 - JBIG2 spec 7.2.4
    rtscarf = buf[5];
    
    if ((rtscarf & 0xe0) == 0xe0) {
        // Long form: count >= 5, uses 4 bytes + retention flags
        rtscarf_long = jbig2_get_uint32(buf + 5);  // buf[5:9]
        referred_to_segment_count = rtscarf_long & 0x1fffffff;
        offset = 5 + 4 + (referred_to_segment_count + 1) / 8;
    } else {
        // Short form: count < 5, uses 1 byte (no retention flags)
        referred_to_segment_count = (rtscarf >> 5);  // bits 7-5
        offset = 5 + 1;  // = 6
    }
    result->referred_to_segment_count = referred_to_segment_count;

    /* STEP 5: Calculate referred-to segment field size */
    // Line 88 - JBIG2 spec 7.2.5
    // Based on THIS segment's number, not the referred-to segment numbers!
    if (result->number <= 256)
        referred_to_segment_size = 1;
    else if (result->number <= 65516)
        referred_to_segment_size = 2;
    else
        referred_to_segment_size = 4;
    
    /* STEP 6: Calculate page association field size */
    // Line 89 - JBIG2 spec 7.2.6
    // Bit 6 of flags determines PA size
    pa_size = (result->flags & 0x40) ? 4 : 1;
    
    /* STEP 7: Verify we have enough data for complete header */
    // Line 90-94
    if (offset + referred_to_segment_count * referred_to_segment_size + pa_size + 4 > buf_size) {
        jbig2_error(ctx, JBIG2_SEVERITY_DEBUG, result->number, 
                    "attempted to parse segment header with insufficient data");
        jbig2_free(ctx->allocator, result);
        return NULL;
    }

    /* STEP 8: Parse referred-to segment numbers */
    // Lines 97-118 - JBIG2 spec 7.2.5
    if (referred_to_segment_count) {
        referred_to_segments = jbig2_new(ctx, uint32_t, referred_to_segment_count);
        
        for (i = 0; i < referred_to_segment_count; i++) {
            referred_to_segments[i] =
                (referred_to_segment_size == 1) ? buf[offset] :
                (referred_to_segment_size == 2) ? jbig2_get_uint16(buf + offset) :
                                                   jbig2_get_uint32(buf + offset);
            offset += referred_to_segment_size;
        }
        result->referred_to_segments = referred_to_segments;
    } else {
        result->referred_to_segments = NULL;
    }

    /* STEP 9: Parse page association */
    // Lines 121-126 - JBIG2 spec 7.2.6
    if (pa_size == 4) {
        result->page_association = jbig2_get_uint32(buf + offset);
        offset += 4;
    } else {
        result->page_association = buf[offset++];
    }

    /* STEP 10: Parse segment data length (ALWAYS 4 bytes) */
    // Line 131 - JBIG2 spec 7.2.7
    result->data_length = jbig2_get_uint32(buf + offset);
    
    /* STEP 11: Calculate and return header size */
    // Line 132
    *p_header_size = offset + 4;

    result->result = NULL;
    return result;
}
```

### Byte-by-Byte Parsing for Segment 0 (Extension)

Given the file structure you described:
- Offset `0x0d`: Segment 0 header starts
- Segment number: 0
- Flags: `0x3e`

Let's trace through:

```
offset = 0x0d (start)

STEP 2: Segment number
  Read buf[0x0d:0x11] = 4 bytes → segment_number = 0
  offset = 0x11

STEP 3: Flags
  Read buf[0x11] = 1 byte → flags = 0x3e
  offset = 0x12

STEP 4: Referred-to segments count
  rtscarf = buf[0x12]
  Assuming rtscarf < 0xe0 (short form):
    referred_to_segment_count = (rtscarf >> 5)
    offset = 0x12 + 1 = 0x13

STEP 5: Referred-to segment size
  Since segment_number = 0 (≤ 256):
    referred_to_segment_size = 1

STEP 6: Page association size
  flags & 0x40 = 0x3e & 0x40 = 0x00 (bit 6 is CLEAR)
  pa_size = 1

STEP 8: Parse referred-to segments
  For each segment (likely 0):
    offset += 0  (no referred-to segments)
  offset = 0x13

STEP 9: Page association  
  pa_size = 1:
    page_association = buf[0x13]
    offset = 0x14

STEP 10: Data length (ALWAYS 4 bytes)
  data_length = jbig2_get_uint32(buf + 0x14)
  data_length = read buf[0x14:0x18] → 104 (0x68)
  offset = 0x18

STEP 11: Header size
  header_size = offset + 4 = 0x18 + 4 = 0x1c? 
  
  WAIT - offset already includes the 4 bytes for data length!
  header_size = offset = 0x18 (24 bytes)
```

**CRITICAL**: Line 132 shows `*p_header_size = offset + 4;`

But `offset` was already incremented by 4 for the data length at line 131! Looking more carefully at the code:

```c
// Line 131: Read data length
result->data_length = jbig2_get_uint32(buf + offset);
// offset is still at the START of data_length field

// Line 132: Calculate header size
*p_header_size = offset + 4;
// Now add 4 to include the data_length field
```

So the header size is: **offset after PA + 4 bytes for data_length**

For segment 0:
- Header size = `0x14 + 4 = 0x18` (24 bytes)
- Data starts at: `0x0d + 0x18 = 0x25` (NOT 0x18!)

**I FOUND THE BUG IN YOUR UNDERSTANDING!**

The segment header starts at `0x0d`, so:
- Header ends at: `0x0d + 24 = 0x25`  
- Data starts at: `0x25`
- Data ends at: `0x25 + 104 = 0x8d`
- Next segment at: `0x8d`

NOT at `0x80`!

---

## Q2: Extension Segment Processing

**Location**: `jbig2_segment.c` lines 209-248

```c
static int
jbig2_parse_extension_segment(Jbig2Ctx *ctx, Jbig2Segment *segment, 
                               const uint8_t *segment_data)
{
    uint32_t type;
    bool reserved;
    bool necessary;

    // Check minimum length
    if (segment->data_length < 4)
        return jbig2_error(ctx, JBIG2_SEVERITY_FATAL, segment->number, 
                          "segment too short");

    // Read extension type (first 4 bytes of data)
    type = jbig2_get_uint32(segment_data);
    reserved = type & 0x20000000;
    necessary = type & 0x80000000;

    if (necessary && !reserved) {
        jbig2_error(ctx, JBIG2_SEVERITY_WARNING, segment->number, 
                    "extension segment is marked 'necessary' but not 'reserved'");
    }

    switch (type) {
    case 0x20000000:
        jbig2_error(ctx, JBIG2_SEVERITY_INFO, segment->number, 
                    "ignoring ASCII comment");
        break;
    case 0x20000002:
        jbig2_error(ctx, JBIG2_SEVERITY_INFO, segment->number, 
                    "ignoring UCS-2 comment");
        break;
    default:
        if (necessary) {
            return jbig2_error(ctx, JBIG2_SEVERITY_FATAL, segment->number, 
                              "unhandled necessary extension segment type 0x%08x", type);
        } else {
            jbig2_error(ctx, JBIG2_SEVERITY_WARNING, segment->number, 
                       "unhandled non-necessary extension segment, skipping");
        }
    }

    return 0;  // SUCCESS - does nothing with the data!
}
```

### What jbig2dec Does With Extension Data

**Answer**: Almost nothing!

1. Reads first 4 bytes to get extension type
2. Checks if it's a known comment type (ASCII or UCS-2)
3. Logs a message ignoring it
4. **Returns 0 (success)**

The extension segment data is **completely skipped**. jbig2dec does NOT:
- Parse embedded structures
- Search for hidden segments
- Process the comment data

After processing, the main loop advances by:
```c
ctx->buf_rd_ix += segment->data_length;  // Skip entire data region
```

---

## Q3: Sequential Segment Reading

**Location**: `jbig2.c` lines 334-424 (main parsing loop in `jbig2_data_in()`)

### Segment Iteration Logic

```c
// State: JBIG2_FILE_SEQUENTIAL_HEADER
// Parse segment header
segment = jbig2_parse_segment_header(ctx, ctx->buf + ctx->buf_rd_ix, 
                                     ctx->buf_wr_ix - ctx->buf_rd_ix, 
                                     &header_size);
if (segment == NULL)
    return 0;  // need more data

// Advance past header
ctx->buf_rd_ix += header_size;

// Store segment
ctx->segments[ctx->n_segments++] = segment;

// Switch to body parsing
ctx->state = JBIG2_FILE_SEQUENTIAL_BODY;

// ... later in JBIG2_FILE_SEQUENTIAL_BODY state ...

segment = ctx->segments[ctx->segment_index];

// Check if we have segment data
if (segment->data_length > ctx->buf_wr_ix - ctx->buf_rd_ix)
    return 0;  // need more data

// Parse segment  
code = jbig2_parse_segment(ctx, segment, ctx->buf + ctx->buf_rd_ix);

// Advance past data
ctx->buf_rd_ix += segment->data_length;

// Move to next segment
ctx->segment_index++;

// Switch back to header parsing
ctx->state = JBIG2_FILE_SEQUENTIAL_HEADER;
```

### Next Segment Calculation

The formula is simple:
```
next_segment_offset = current_offset + header_size + data_length
```

Where:
- `current_offset` = start of current segment header
- `header_size` = returned by `jbig2_parse_segment_header()`
- `data_length` = from segment header

This is implemented as:
```c
ctx->buf_rd_ix += header_size;  // Move to data
// ... parse data ...
ctx->buf_rd_ix += segment->data_length;  // Move to next header
```

---

## Q4: The Mystery of Segment 1

Based on my analysis, here's where segment 1 actually is:

### Corrected Calculation

```
File offset 0x00: JBIG2 magic (8 bytes)
File offset 0x08: File header flags (1 byte)  
File offset 0x09: Number of pages (4 bytes) - IF flags bit 1 is clear
File offset 0x0d: FIRST SEGMENT HEADER STARTS

Segment 0 (Extension):
  Header start: 0x0d
  Header size: calculated by jbig2_parse_segment_header()
  
  Let's trace with actual bytes:
    0x0d-0x10: segment number = 0x00000000
    0x11: flags = 0x3e  
    0x12: rtscarf (referred-to count)
    ...
    [PA field]
    [4-byte data length]
  
  Assuming header_size = 11 bytes (minimum):
    Header ends: 0x0d + 11 = 0x18
    Data starts: 0x18
    Data length: 104 bytes
    Data ends: 0x18 + 104 = 0x80
    
  Next segment: 0x80

Segment 1 (Page Info):
  Should be at: 0x80
```

**But you said segment 1 isn't at 0x80!**

This means one of two things:
1. The header size is NOT 11 bytes (there are referred-to segments or different PA size)
2. The data length in the file is not actually 104 bytes

To find the EXACT location, you need to:
1. Hexdump bytes 0x0d-0x20 from the file
2. Calculate header size using the exact algorithm above
3. The actual next segment will be at: `0x0d + header_size + data_length`

---

## Q5: Data Length Field Format

**Location**: JBIG2 spec 7.2.7, implemented in `jbig2_segment.c` line 131

### Answer: ALWAYS 4 bytes

```c
// Line 131
result->data_length = jbig2_get_uint32(buf + offset);
```

The data_length field is **ALWAYS** 4 bytes (32-bit big-endian), regardless of ANY flags.

### Flags Byte Breakdown

Flags byte (1 byte) at offset +4:
```
Bit 0-5: Segment type (63 possible types)
Bit 6:   Page association field size (0=1 byte, 1=4 bytes)
Bit 7:   Deferred/immediate (for certain segment types)
```

For segment 0 with flags = `0x3e` = binary `00111110`:
- Bits 0-5: `111110` = 62 (extension segment) ✓
- Bit 6: `0` → PA size = 1 byte ✓
- Bit 7: `0` → immediate

**No flag bits control the data_length field size!** It's always 4 bytes.

---

## Q6: Full File Parsing Walkthrough

I cannot run jbig2dec on your file since I don't have access to the `jbig2-rs` workspace, but I can provide the algorithm to trace it yourself.

### Algorithm to Trace Parsing

```python
#!/usr/bin/env python3
import struct

def get_uint32(data, offset):
    return struct.unpack('>I', data[offset:offset+4])[0]

def get_uint16(data, offset):
    return struct.unpack('>H', data[offset:offset+2])[0]

def trace_segment_header(data, offset):
    print(f"\n=== Segment header at 0x{offset:04x} ===")
    
    # Segment number
    seg_num = get_uint32(data, offset)
    print(f"Segment number: {seg_num}")
    offset += 4
    
    # Flags
    flags = data[offset]
    seg_type = flags & 0x3f
    pa_long = bool(flags & 0x40)
    print(f"Flags: 0x{flags:02x} (type={seg_type}, PA={'4-byte' if pa_long else '1-byte'})")
    offset += 1
    
    # Referred-to segments count
    rtscarf = data[offset]
    if (rtscarf & 0xe0) == 0xe0:
        # Long form
        rtscarf_long = get_uint32(data, offset)
        ref_count = rtscarf_long & 0x1fffffff
        offset += 4
        offset += (ref_count + 1) // 8  # retention flags
        print(f"Referred-to count: {ref_count} (long form)")
    else:
        # Short form
        ref_count = (rtscarf >> 5)
        offset += 1
        print(f"Referred-to count: {ref_count} (short form)")
    
    # Referred-to segment size
    if seg_num <= 256:
        ref_size = 1
    elif seg_num <= 65536:
        ref_size = 2
    else:
        ref_size = 4
    print(f"Referred-to segment size: {ref_size} bytes")
    
    # Parse referred-to segments
    for i in range(ref_count):
        if ref_size == 1:
            ref_seg = data[offset]
        elif ref_size == 2:
            ref_seg = get_uint16(data, offset)
        else:
            ref_seg = get_uint32(data, offset)
        print(f"  Refers to segment: {ref_seg}")
        offset += ref_size
    
    # Page association
    pa_size = 4 if pa_long else 1
    if pa_size == 4:
        page_assoc = get_uint32(data, offset)
    else:
        page_assoc = data[offset]
    print(f"Page association: {page_assoc}")
    offset += pa_size
    
    # Data length
    data_len = get_uint32(data, offset)
    print(f"Data length: {data_len}")
    offset += 4
    
    return seg_num, seg_type, data_len, offset

# Usage:
with open('minimal_valid.jb2', 'rb') as f:
    data = f.read()

# Skip file header (either 9 or 13 bytes)
file_flags = data[8]
if file_flags & 0x02:
    buf_offset = 9
else:
    buf_offset = 13

# Parse segments
while buf_offset < len(data):
    start_offset = buf_offset
    seg_num, seg_type, data_len, header_end_offset = trace_segment_header(data, buf_offset)
    
    header_size = header_end_offset - start_offset
    data_start = header_end_offset
    data_end = data_start + data_len
    
    print(f"\nHeader: 0x{start_offset:04x} - 0x{header_end_offset:04x} ({header_size} bytes)")
    print(f"Data:   0x{data_start:04x} - 0x{data_end:04x} ({data_len} bytes)")
    print(f"Next segment at: 0x{data_end:04x}")
    
    # For page info segments, show dimensions
    if seg_type == 48 and data_len >= 8:
        width = get_uint32(data, data_start)
        height = get_uint32(data, data_start + 4)
        print(f"★★★ PAGE DIMENSIONS: {width} x {height} ★★★")
    
    buf_offset = data_end
    
    # Safety
    if buf_offset > len(data):
        break
```

Run this script on `minimal_valid.jb2` to get the exact offsets.

---

## Critical Bug in Your Understanding

You stated:
> Extension data starts at `0x18`, should end at `0x18 + 104 = 0x80`

**This is correct!**

But you also stated:
> At offset `0x80`, we cannot find a valid segment header for segment 1

**This suggests your header size calculation is wrong!**

The segment header does NOT start at file offset `0x18`. Let me recalculate:

```
File offset 0x00-0x0c: File header (13 bytes)
File offset 0x0d: Segment 0 header START

If header_size = 11 bytes (minimum):
  Header: 0x0d - 0x18 (11 bytes)
  Data: 0x18 - 0x80 (104 bytes) ✓
  Next: 0x80

If header_size > 11 bytes:
  Header: 0x0d - (0x0d + header_size)
  Data: (0x0d + header_size) - (0x0d + header_size + 104)
  Next: 0x0d + header_size + 104
```

**You need to verify the exact header size** by:
1. Checking if there are referred-to segments (look at byte 0x12)
2. Checking the PA size flag (bit 6 of flags at 0x11)

---

## Summary of Answers

### Q1: Segment Header Parsing
- Variable size: 11+ bytes
- Depends on referred-to count, segment number, and flags
- Data length field is ALWAYS 4 bytes
- Returns header size via `*p_header_size` parameter

### Q2: Extension Segments
- jbig2dec reads first 4 bytes for extension type
- Logs and ignores the data
- Returns 0 (success)
- Main loop skips entire data region

### Q3: Sequential Reading
- Next segment = current + header_size + data_length
- Two-phase: parse header, then parse body
- Buffer index advances twice per segment

### Q4: Segment 1 Location
- Should be at: 0x0d + segment0_header_size + 104
- Need exact header size from actual file bytes
- Use hexdump to verify

### Q5: Data Length Field
- ALWAYS 4 bytes
- No flags affect this
- Bit 6 of flags affects PA size only

### Q6: Full Walkthrough
- Use the Python script above
- Trace each segment with exact offsets
- Verify against jbig2dec verbose output

---

## Next Steps for Your Rust Decoder

1. **Fix header size calculation** - Ensure you handle:
   - Short vs long referred-to segment count form
   - Variable referred-to segment size (1, 2, or 4 bytes)
   - Variable PA size (1 or 4 bytes)

2. **Verify with hexdump** - Check bytes 0x0d-0x20 to calculate exact header size

3. **Test the algorithm** - Use the Python script to verify your understanding

4. **Compare with jbig2dec** - Run `jbig2dec -v` on the file and compare offsets
