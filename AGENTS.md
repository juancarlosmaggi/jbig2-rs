# AGENTS.md

## Current Goal
Validate `jbig2-rs` output against `jbig2dec` using the full UBC fixture set
in `tests/resources/ubc`, confirm the halftone fix, and decide whether to keep
or trim debug dumps.

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
    - `JBIG2_RS_TRACE_HALFTONE=1` for halftone grid/pattern parameters.
    - `JBIG2_RS_DUMP_PATTERNS=/tmp/dir` to dump pattern dictionaries (P1 PBM).
    - `JBIG2_RS_DUMP_HALFTONE_GRID=/tmp/file` to dump halftone grid indices.
  - Use `JBIG2_RS_NAIVE_COMBINE=1` to sanity-check bitmap composition
    if a mismatch appears.

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

### 4) Batch compare all .jb2 files
```
import subprocess
from pathlib import Path

root = Path("/home/jmaggi/projects/jbig2-rs")
dec = Path("/home/jmaggi/projects/jbig2dec/jbig2dec")
tests = root / "tests" / "resources"
out_ref = Path("/tmp/jbig2-compare/jbig2dec_all")
out_rs = Path("/tmp/jbig2-compare/jbig2-rs_all")
out_ref.mkdir(parents=True, exist_ok=True)
out_rs.mkdir(parents=True, exist_ok=True)

def compare(pbm_path, raw_path):
    pbm = pbm_path.read_bytes()
    idx = 2
    def skip_ws(i):
        while i < len(pbm) and pbm[i] in b" \t\r\n":
            i += 1
        return i
    vals = []
    while len(vals) < 2:
        idx2 = skip_ws(idx)
        if pbm[idx2:idx2 + 1] == b"#":
            idx2 = pbm.find(b"\n", idx2) + 1
            idx = idx2
            continue
        end = idx2
        while end < len(pbm) and pbm[end] not in b" \t\r\n":
            end += 1
        vals.append(int(pbm[idx2:end]))
        idx = end
    idx = skip_ws(idx)
    w, h = vals
    stride = (w + 7) // 8
    pbm_data = pbm[idx:]
    raw = raw_path.read_bytes()
    raw_bits = bytearray(len(raw))
    for i, b in enumerate(raw):
        raw_bits[i] = 1 if b else 0
    mismatch = 0
    for y in range(h):
        base = y * stride
        row_start = y * w
        for x in range(w):
            byte = pbm_data[base + (x >> 3)]
            bit = (byte >> (7 - (x & 7))) & 1
            if bit != raw_bits[row_start + x]:
                mismatch += 1
    return mismatch, w * h

for jb2 in sorted(tests.glob("*.jb2")):
    pbm = out_ref / (jb2.stem + ".pbm")
    raw = out_rs / (jb2.stem + ".bin")
    subprocess.run([str(dec), "-o", str(pbm), str(jb2)], check=True)
    subprocess.run(
        ["cargo", "run", "--example", "decode_file", "--quiet", "--", str(jb2), str(raw)],
        cwd=root,
        check=True,
    )
    mism, total = compare(pbm, raw)
    pct = mism * 100 / total if total else 0
    print(jb2.name, "mismatch", mism, "percent", pct)
```

## Current Findings / Status
- Symbol dictionary parsing now captures SDAT/SDRAT AT pixels; `decode_symbol`
  uses them directly (no backward parsing).
- Export run-length alignment fixed by consuming the OOB IADW after the final
  symbol in a height class; IAEX now matches jbig2dec.
- Refinement OOB handling now mirrors jbig2dec: arithmetic OOB in IARDW/H/DX/DY
  is treated as an error.
- Intermediate text regions are stored in `bitmaps` for later refinement
  reference (fixes `ubc/042_21`/`042_22`/`042_24` blank output).
- Generic refinement region flags parsing fixed (template bit, TPGRON), and
  refinement decoding now implements TPGRON implicit-value logic with correct
  context ordering and start context values (0x100/0x40).
- MMR decoder no longer treats EOFB as a mode code; it now consumes EOFB after
  completing rows. Halftone MMR plane consumption matches jbig2dec.
- `cargo test jbig2dec_hashes` now passes across UBC fixtures.
- New debug/env knobs:
  - `JBIG2_RS_TRACE_INT` (symbol dict IADH/IADW), `JBIG2_RS_TRACE_REFINE`
    (refinement IDs), `JBIG2_RS_TRACE_CLASS` (per-class symbol offsets),
    `JBIG2_RS_TRACE_EXPORT_OFFSET`, `JBIG2_RS_TRACE_SYMDICT_OFFSET`,
    `JBIG2_RS_TRACE_HALFTONE_BYTES`.
  - `JBIG2_RS_TRACE_ERRORS` to log lenient decode stops; `JBIG2_RS_STRICT=1`
    to revert to hard errors.
  - `JBIG2_RS_DISABLE_TEMPLATE0_FAST=1` to disable generic template-0 fast path.
- jbig2dec instrumentation added for export/agg/int traces, symbol dict AT,
  refinement IDs, halftone plane bytes, and arithmetic byte offsets.

## Current Plan
1) Decide whether to keep or trim the new debug gates and jbig2dec
   instrumentation now that parity is green.
2) Optionally add regression tests for TPGRON refinement and halftone MMR plane
   consumption.
