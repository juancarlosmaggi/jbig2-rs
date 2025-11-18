# Using the JBIG2 Parser to Debug Your File

## Quick Start

The Python parser has been created and tested successfully in the jbig2dec project. To use it on your `minimal_valid.jb2` file:

### Step 1: Copy the Parser

```bash
# From the jbig2dec directory, copy to your jbig2-rs project:
cp /home/jmaggi/projects/jbig2dec/jbig2_parser.py /home/jmaggi/projects/jbig2-rs/
```

### Step 2: Run on Your Test File

```bash
cd /home/jmaggi/projects/jbig2-rs
python3 jbig2_parser.py tests/resources/minimal_valid.jb2
```

## What to Look For

The parser will show you:

1. **Each segment's exact location**:
   - Header start offset
   - Header size (in bytes)
   - Data start offset
   - Data end offset
   - Next segment offset

2. **For Page Info segments (type 48)**:
   - Page dimensions (width × height)
   - The exact bytes containing the dimensions

3. **For Extension segments (type 62)**:
   - Extension type
   - Whether jbig2dec ignores it

## Example Output

From testing on `annex-h.jbig2`, the parser correctly identified:

```
--- Parsing segment at offset 0x000d (13) ---
✓ Segment 0: type=0, flags=0x00, page=0, data_len=24
  Header size: 11 bytes
  Header range: 0x000d - 0x0018
  Data range: 0x0018 - 0x0030 (24 bytes)
  Next segment at: 0x0030 (48)

--- Parsing segment at offset 0x0030 (48) ---
✓ Segment 1: type=48, flags=0x30, page=1, data_len=19
  Header size: 11 bytes
  Header range: 0x0030 - 0x003b
  Data range: 0x003b - 0x004e (19 bytes)
  Segment type: 48 (Page Information)
  ★★★ Page dimensions: 64 x 56 ★★★
```

## Key Things to Check

When you run this on `minimal_valid.jb2`, verify:

### 1. Segment 0 Header Size

Look at the output for segment 0:
- Is the header size 11 bytes?
- Or is it larger (12, 13, 14+ bytes)?

The header size depends on:
- Number of referred-to segments
- Whether page association is 1 or 4 bytes (flags bit 6)

### 2. Where Does Segment 1 Actually Start?

The formula is:
```
segment_1_offset = 0x0d + segment_0_header_size + segment_0_data_length
```

If segment 0 has:
- Header size: 11 bytes → data starts at 0x18
- Data length: 104 bytes → data ends at 0x80
- Next segment: 0x80

But if header size is different (e.g., 12 bytes):
- Data starts at 0x19
- Data ends at 0x81
- Next segment: 0x81

### 3. Verify Page Dimensions

The parser will show:
```
★★★ Page dimensions: 1728 x 2339 ★★★
  Width bytes [+0:+4]: 000006c0 = 1728
  Height bytes [+4:+8]: 00000923 = 2339
```

This confirms:
- Segment 1 was found correctly
- Dimensions are at the right offset
- Your decoder should read from this exact location

## Compare with Your Rust Decoder

After running the parser, compare:

1. **Your decoder's segment 0 offset**: Should be 0x0d
2. **Your decoder's segment 0 header size**: Should match parser output
3. **Your decoder's segment 0 data range**: Should match parser output
4. **Your decoder's segment 1 offset**: Should match parser's "Next segment at"

If any of these differ, that's where the bug is!

## Common Issues

### Issue: "Segment 1 not found at 0x80"

**Likely cause**: Header size calculation is wrong

**Solution**: Check if there are referred-to segments or if PA size is 4 bytes

### Issue: "Getting 100×100 dimensions"

**Likely cause**: Reading from wrong offset (using fallback default)

**Solution**: Verify segment 1 is found at the correct offset first

### Issue: "Extension data length seems wrong"

**Likely cause**: Data length field is always 4 bytes, no special handling needed

**Solution**: Make sure you're reading 4 bytes big-endian for data_length

## Verification with jbig2dec

You can also run jbig2dec in verbose mode to compare:

```bash
# Build jbig2dec if not already built
cd /home/jmaggi/projects/jbig2dec
make

# Run on your file
./jbig2dec -v /home/jmaggi/projects/jbig2-rs/tests/resources/minimal_valid.jb2 2>&1 | head -20
```

The output should show segment information that matches the Python parser exactly.
