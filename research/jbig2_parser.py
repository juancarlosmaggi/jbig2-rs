#!/usr/bin/env python3
"""
JBIG2 Segment Parser - Exact jbig2dec Logic Replication

This script replicates the exact parsing logic from jbig2dec to understand
how it parses JBIG2 files and finds page dimensions.

Based on:
- jbig2_parse_segment_header() from jbig2_segment.c lines 42-138
- jbig2_data_in() main loop from jbig2.c lines 230-436  
- jbig2_page_info() from jbig2_page.c lines 62-168
"""

import struct
import sys
from dataclasses import dataclass
from typing import List, Tuple, Optional


@dataclass
class Jbig2Segment:
    """Represents a JBIG2 segment"""
    number: int
    flags: int
    page_association: int
    data_length: int
    referred_to_segment_count: int
    referred_to_segments: List[int]
    
    @property
    def segment_type(self) -> int:
        """Extract segment type from flags (bits 0-5)"""
        return self.flags & 0x3f
    
    def __str__(self):
        return (f"Segment {self.number}: type={self.segment_type}, "
                f"flags=0x{self.flags:02x}, page={self.page_association}, "
                f"data_len={self.data_length}")


def get_uint16(data: bytes, offset: int) -> int:
    """Read big-endian 16-bit unsigned integer"""
    return (data[offset] << 8) | data[offset + 1]


def get_uint32(data: bytes, offset: int) -> int:
    """Read big-endian 32-bit unsigned integer - matches jbig2_get_uint32()"""
    return (get_uint16(data, offset) << 16) | get_uint16(data, offset + 2)


def parse_segment_header(buf: bytes, buf_offset: int = 0) -> Tuple[Optional[Jbig2Segment], int]:
    """
    Parse segment header - EXACT replica of jbig2_parse_segment_header()
    
    Returns: (segment, header_size) or (None, 0) if insufficient data
    
    This follows the exact logic from jbig2_segment.c lines 42-138
    """
    # Minimum possible size of a jbig2 segment header (line 55-56)
    if len(buf) - buf_offset < 11:
        return None, 0
    
    offset = buf_offset
    
    # 7.2.2 - segment number (lines 65)
    segment_number = get_uint32(buf, offset)
    offset += 4
    
    # 7.2.3 - segment header flags (line 73)
    flags = buf[offset]
    offset += 1
    
    # 7.2.4 - referred-to segments (lines 76-84)
    rtscarf = buf[offset]
    if (rtscarf & 0xe0) == 0xe0:
        # Long form: 4 bytes
        rtscarf_long = get_uint32(buf, offset)
        referred_to_segment_count = rtscarf_long & 0x1fffffff
        offset += 4
        offset += (referred_to_segment_count + 1) // 8  # retention flags
    else:
        # Short form: 1 byte
        referred_to_segment_count = (rtscarf >> 5)
        offset += 1
    
    # Compute referred-to segment size (line 88)
    if segment_number <= 256:
        referred_to_segment_size = 1
    elif segment_number <= 65536:
        referred_to_segment_size = 2
    else:
        referred_to_segment_size = 4
    
    # Page association size (line 89)
    pa_size = 4 if (flags & 0x40) else 1
    
    # Check if we have enough data (line 90-94)
    needed = offset - buf_offset + (referred_to_segment_count * referred_to_segment_size) + pa_size + 4
    if buf_offset + needed > len(buf):
        print(f"  [DEBUG] Need {needed} bytes, have {len(buf) - buf_offset}")
        return None, 0
    
    # 7.2.5 - parse referred-to segments (lines 97-118)
    referred_to_segments = []
    for i in range(referred_to_segment_count):
        if referred_to_segment_size == 1:
            ref_seg = buf[offset]
        elif referred_to_segment_size == 2:
            ref_seg = get_uint16(buf, offset)
        else:
            ref_seg = get_uint32(buf, offset)
        referred_to_segments.append(ref_seg)
        offset += referred_to_segment_size
    
    # 7.2.6 - page association (lines 121-126)
    if pa_size == 4:
        page_association = get_uint32(buf, offset)
        offset += 4
    else:
        page_association = buf[offset]
        offset += 1
    
    # 7.2.7 - segment data length (line 131)
    data_length = get_uint32(buf, offset)
    offset += 4
    
    # Calculate header size (line 132)
    header_size = offset - buf_offset
    
    segment = Jbig2Segment(
        number=segment_number,
        flags=flags,
        page_association=page_association,
        data_length=data_length,
        referred_to_segment_count=referred_to_segment_count,
        referred_to_segments=referred_to_segments
    )
    
    return segment, header_size


def parse_page_info(segment_data: bytes) -> Tuple[int, int, int, int]:
    """
    Parse page information segment - matches jbig2_page_info()
    
    Returns: (width, height, x_res, y_res)
    
    This follows jbig2_page.c lines 116-120
    """
    width = get_uint32(segment_data, 0)
    height = get_uint32(segment_data, 4)
    x_resolution = get_uint32(segment_data, 8)
    y_resolution = get_uint32(segment_data, 12)
    
    return width, height, x_resolution, y_resolution


def parse_jbig2_file(filepath: str, verbose: bool = True):
    """
    Parse JBIG2 file following exact jbig2dec logic
    
    This replicates the main parsing loop from jbig2_data_in() in jbig2.c
    """
    with open(filepath, 'rb') as f:
        data = f.read()
    
    print(f"=" * 80)
    print(f"JBIG2 File Parser - Replicating jbig2dec Logic")
    print(f"File: {filepath}")
    print(f"Size: {len(data)} bytes")
    print(f"=" * 80)
    print()
    
    # Check file header (jbig2.c line 288)
    jbig2_id = bytes([0x97, 0x4a, 0x42, 0x32, 0x0d, 0x0a, 0x1a, 0x0a])
    if data[:8] != jbig2_id:
        print("ERROR: Not a valid JBIG2 file!")
        return
    
    print("✓ Valid JBIG2 file header")
    
    # File header flags (line 301)
    file_header_flags = data[8]
    print(f"File header flags: 0x{file_header_flags:02x}")
    
    # Determine file organization (line 326-332)
    sequential = bool(file_header_flags & 1)
    print(f"File organization: {'Sequential' if sequential else 'Random-access'}")
    
    # Check for number of pages (line 312-324)
    if file_header_flags & 2:
        n_pages = 0
        buf_rd_ix = 9
        print("Number of pages: unknown")
    else:
        n_pages = get_uint32(data, 9)
        buf_rd_ix = 13
        print(f"Number of pages: {n_pages}")
    
    print()
    print("=" * 80)
    print("SEGMENT PARSING")
    print("=" * 80)
    print()
    
    # Parse segments (sequential mode - lines 334-424)
    segment_index = 0
    segments = []
    
    while buf_rd_ix < len(data):
        print(f"\n--- Parsing segment at offset 0x{buf_rd_ix:04x} ({buf_rd_ix}) ---")
        
        # Parse segment header (line 336)
        segment, header_size = parse_segment_header(data, buf_rd_ix)
        
        if segment is None:
            print(f"✗ Not enough data to parse segment header")
            break
        
        print(f"✓ {segment}")
        print(f"  Header size: {header_size} bytes")
        print(f"  Header range: 0x{buf_rd_ix:04x} - 0x{buf_rd_ix + header_size:04x}")
        
        # Advance buffer past header (line 339)
        buf_rd_ix += header_size
        
        data_start = buf_rd_ix
        data_end = buf_rd_ix + segment.data_length
        
        print(f"  Data range: 0x{data_start:04x} - 0x{data_end:04x} ({segment.data_length} bytes)")
        
        # Check if we have segment data (line 413-414)
        if segment.data_length > len(data) - buf_rd_ix:
            print(f"✗ Not enough data for segment body (need {segment.data_length}, have {len(data) - buf_rd_ix})")
            break
        
        # Extract segment data
        segment_data = data[data_start:data_end]
        
        # Process based on segment type (line 340 in jbig2_segment.c)
        seg_type = segment.segment_type
        
        if seg_type == 48:  # Page information (line 362-363 in jbig2_segment.c)
            print(f"  Segment type: 48 (Page Information)")
            if segment.data_length >= 19:
                width, height, x_res, y_res = parse_page_info(segment_data)
                print(f"  ★★★ Page dimensions: {width} x {height} ★★★")
                if x_res == 0:
                    print(f"  Resolution: unknown")
                else:
                    print(f"  Resolution: {x_res} x {y_res}")
                
                # Show raw bytes where dimensions are stored
                print(f"  Width bytes [+0:+4]: {segment_data[0:4].hex()} = {width}")
                print(f"  Height bytes [+4:+8]: {segment_data[4:8].hex()} = {height}")
        
        elif seg_type == 62:  # Extension segment (line 378-379 in jbig2_segment.c)
            print(f"  Segment type: 62 (Extension)")
            if segment.data_length >= 4:
                ext_type = get_uint32(segment_data, 0)
                print(f"  Extension type: 0x{ext_type:08x}")
                
                if ext_type == 0x20000000:
                    print(f"  → ASCII comment (ignored by jbig2dec)")
                elif ext_type == 0x20000002:
                    print(f"  → UCS-2 comment (ignored by jbig2dec)")
                else:
                    print(f"  → Unknown extension type")
        
        elif seg_type == 49:  # End of page
            print(f"  Segment type: 49 (End of Page)")
        
        elif seg_type == 51:  # End of file
            print(f"  Segment type: 51 (End of File)")
        
        else:
            print(f"  Segment type: {seg_type}")
        
        # Advance buffer past data (line 417)
        buf_rd_ix += segment.data_length
        
        segments.append(segment)
        segment_index += 1
        
        print(f"  Next segment at: 0x{buf_rd_ix:04x} ({buf_rd_ix})")
        
        # Safety limit
        if segment_index > 100:
            print("\nReached safety limit of 100 segments")
            break
    
    print()
    print("=" * 80)
    print(f"SUMMARY: Parsed {len(segments)} segments")
    print("=" * 80)
    
    for i, seg in enumerate(segments):
        type_name = {
            48: "Page Info",
            49: "End of Page", 
            51: "End of File",
            62: "Extension"
        }.get(seg.segment_type, f"Type {seg.segment_type}")
        
        print(f"Segment {i}: #{seg.number} - {type_name} - {seg.data_length} bytes")
    
    print()


if __name__ == "__main__":
    if len(sys.argv) > 1:
        filepath = sys.argv[1]
    else:
        print("Usage: python3 jbig2_parser.py <jbig2_file>")
        sys.exit(1)
    
    parse_jbig2_file(filepath)
