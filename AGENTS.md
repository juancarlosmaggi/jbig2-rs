# AGENTS.md

## Current Goal
Diagnose and fix the pixel mismatch in the `text_region` test by comparing `jbig2-rs`
output to `jbig2dec`. The mismatch is currently about 15.6% of pixels, with evidence
that some symbol instances are placed at incorrect vertical positions.

## Testing and Comparison Workflow
- Decode reference output with `jbig2dec` to PBM.
- Decode with `jbig2-rs` using `examples/decode_file.rs` to raw 1bpp data.
- Compare:
  - Bit-level mismatch vs PBM (expanded to 0/1 pixels).
  - Row/column mismatch distribution to spot alignment patterns.
  - Debug traces in `jbig2-rs`:
    - `JBIG2_RS_TRACE_TEXT=1` for text region internals.
    - `JBIG2_RS_TRACE_TEXT_REF=/path/to/ref.pbm` to compare placements
      directly against the reference bitmap.
    - `JBIG2_RS_TRACE_SYMBOL=1` for symbol dictionary stats.
    - `JBIG2_RS_TRACE_MMR=1` for MMR decode issues (currently clean).

Example run:
```
jbig2dec -o /tmp/jbig2-compare/jbig2dec/text_region.pbm tests/resources/text_region.jb2
JBIG2_RS_TRACE_TEXT=1 \
JBIG2_RS_TRACE_TEXT_REF=/tmp/jbig2-compare/jbig2dec/text_region.pbm \
  cargo run --example decode_file --quiet -- \
  tests/resources/text_region.jb2 /tmp/jbig2-compare/jbig2-rs/text_region.bin \
  2> /tmp/jbig2-compare/jbig2-rs/text_region.trace
```

## Example Python Scripts Used for Comparison

### 1) PBM vs raw pixel mismatch (expanded to 0/1)
```
from pathlib import Path

pbm_path = Path("/tmp/jbig2-compare/jbig2dec/text_region.pbm")
raw_path = Path("/tmp/jbig2-compare/jbig2-rs/text_region.bin")

pbm = pbm_path.read_bytes()
idx = 2
def skip_ws(i):
    while i < len(pbm) and pbm[i] in b" \t\r\n":
        i += 1
    return i

vals = []
while len(vals) < 2:
    idx = skip_ws(idx)
    if pbm[idx:idx + 1] == b"#":
        idx = pbm.find(b"\n", idx) + 1
        continue
    end = idx
    while end < len(pbm) and pbm[end] not in b" \t\r\n":
        end += 1
    vals.append(int(pbm[idx:end]))
    idx = end
idx = skip_ws(idx)

w, h = vals
stride = (w + 7) // 8
pbm_data = pbm[idx:]
raw = raw_path.read_bytes()

raw_bits = bytearray(len(raw))
for i, b in enumerate(raw):
    raw_bits[i] = 1 if b else 0

pbm_bits = bytearray(w * h)
for y in range(h):
    base = y * stride
    row_start = y * w
    for x in range(w):
        byte = pbm_data[base + (x >> 3)]
        bit = (byte >> (7 - (x & 7))) & 1
        pbm_bits[row_start + x] = bit

mismatch = sum(1 for i in range(w * h) if pbm_bits[i] != raw_bits[i])
print("mismatch", mismatch, "percent", mismatch / (w * h) * 100)
```

### 2) Row mismatch distribution
```
row_mismatch = [0] * h
for y in range(h):
    base = y * stride
    row_start = y * w
    for x in range(w):
        byte = pbm_data[base + (x >> 3)]
        bit = (byte >> (7 - (x & 7))) & 1
        raw_bit = raw_bits[row_start + x]
        if bit != raw_bit:
            row_mismatch[y] += 1

pairs = sorted(((c, y) for y, c in enumerate(row_mismatch)), reverse=True)
print("top mismatch rows:", pairs[:10])
```

### 3) Quick shift check (sampled)
```
step = 8
def mismatch_with_shift(dx, dy):
    mismatch = 0
    count = 0
    for y in range(0, h, step):
        y2 = y + dy
        if y2 < 0 or y2 >= h:
            continue
        row_start = y * w
        row2 = y2 * w
        for x in range(0, w, step):
            x2 = x + dx
            if x2 < 0 or x2 >= w:
                continue
            if pbm_bits[row_start + x] != raw_bits[row2 + x2]:
                mismatch += 1
            count += 1
    return mismatch, count

for dy in range(-4, 5):
    mism, count = mismatch_with_shift(0, dy)
    print("dy", dy, "mismatch %", mism / count * 100)
```

## Current Findings / Suspicions
- MMR decoding errors were present before; after alignment fixes, `JBIG2_RS_TRACE_MMR`
  shows no invalid modes/runs and mismatch persists.
- The largest discrepancies are horizontal bands; two very wide symbols are decoded
  correctly but placed too low. Using `JBIG2_RS_TRACE_TEXT_REF`, their best alignment
  offsets are around `best_dy = -8` to `-9`.
- This points to a text-region placement issue, likely in the strip T progression
  (initial IADT or delta_t decoding/bit alignment) rather than bitmap content.
- Symbol dictionary appears consistent: wide symbols match their expected patterns.

## Current Plan
1) Identify where strip T drift starts by correlating mismatch bands with strip
   indices; add per-strip comparison or a compact strip log for offline diff.
2) Verify IADT/DT decoding and bit alignment after the symbol ID Huffman table
   (byte-align is used, but placement suggests an off-by-N in T).
3) If possible, instrument `jbig2dec` locally to log DT/STRIPT for direct comparison
   against `jbig2-rs` (blocked by write permissions in that repo right now).
