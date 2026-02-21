use crate::common::error::Jbig2Error;
use crate::parser::segment::read_segment_header;

const MAGIC: &[u8; 8] = b"\x97\x4a\x42\x32\x0d\x0a\x1a\x0a";

/// Probe raw bytes consumed by a JBIG2 stream without fully decoding pages.
///
/// Returns `Ok(Some(consumed))` when a deterministic segment boundary is found.
/// Returns `Ok(None)` when the stream is syntactically incomplete or no reliable
/// boundary can be inferred from available segment headers.
pub fn probe_stream_consumed_bytes(data: &[u8]) -> Result<Option<usize>, Jbig2Error> {
    let (has_file_header, sequential, mut pos) = parse_stream_header(data)?;
    if pos >= data.len() {
        return Ok(None);
    }

    if sequential {
        let mut any_segment = false;
        let mut consumed_end = pos;
        while pos < data.len() {
            if pos + 11 > data.len() {
                break;
            }
            let header = match read_segment_header(data, pos, has_file_header) {
                Ok(header) => header,
                Err(_) => return Ok(None),
            };
            any_segment = true;
            pos = header.header_end;
            consumed_end = pos;

            if header.segment_type == 51 {
                return Ok(Some(consumed_end));
            }

            let Some(segment_end) = pos.checked_add(header.length) else {
                return Err(Jbig2Error::new(
                    "segment length overflow while probing stream",
                ));
            };
            if segment_end > data.len() {
                return Ok(None);
            }
            pos = segment_end;
            consumed_end = segment_end;
        }
        return Ok(if any_segment {
            Some(consumed_end)
        } else {
            None
        });
    }

    let mut payload_lengths = Vec::new();
    let mut directory_end = pos;
    let mut any_segment = false;
    while pos < data.len() {
        if pos + 11 > data.len() {
            break;
        }
        let header = match read_segment_header(data, pos, has_file_header) {
            Ok(header) => header,
            Err(_) => return Ok(None),
        };
        any_segment = true;
        pos = header.header_end;
        directory_end = pos;
        if header.segment_type == 51 {
            break;
        }
        payload_lengths.push(header.length);
    }
    if !any_segment {
        return Ok(None);
    }

    let mut payload_pos = directory_end;
    for length in payload_lengths {
        let Some(next) = payload_pos.checked_add(length) else {
            return Err(Jbig2Error::new(
                "segment length overflow while probing stream",
            ));
        };
        if next > data.len() {
            return Ok(None);
        }
        payload_pos = next;
    }
    Ok(Some(payload_pos))
}

fn parse_stream_header(data: &[u8]) -> Result<(bool, bool, usize), Jbig2Error> {
    if data.len() < 8 || data.get(0..8) != Some(MAGIC.as_slice()) {
        return Ok((false, true, 0));
    }

    let mut pos = 8usize;
    if data.len() <= pos {
        return Err(Jbig2Error::new("missing file header flags in stream probe"));
    }
    let flags = data[pos];
    pos += 1;
    let sequential = (flags & 1) != 0;
    let has_num_pages = (flags & 2) == 0;
    if (flags & 0xfc) != 0 {
        return Err(Jbig2Error::new("invalid file header flags in stream probe"));
    }
    if has_num_pages {
        if data.len() < pos + 4 {
            return Err(Jbig2Error::new(
                "missing declared page count in stream probe",
            ));
        }
        pos += 4;
    }
    Ok((true, sequential, pos))
}

#[cfg(test)]
mod tests {
    use super::probe_stream_consumed_bytes;

    fn make_segment_header(number: u32, segment_type: u8, length: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&number.to_be_bytes());
        bytes.push(segment_type & 0x3f);
        bytes.push(0x00); // referred_to_count=0, retain bits in same byte
        bytes.push(0x01); // page association (1 byte)
        bytes.extend_from_slice(&length.to_be_bytes());
        bytes
    }

    #[test]
    fn test_probe_stream_consumed_bytes_headerless_sequential() {
        let mut data = Vec::new();
        data.extend_from_slice(&make_segment_header(1, 0, 0));
        data.extend_from_slice(&make_segment_header(2, 51, 0)); // EOF
        let consumed = probe_stream_consumed_bytes(&data).expect("probe result");
        assert_eq!(consumed, Some(data.len()));
    }

    #[test]
    fn test_probe_stream_consumed_bytes_with_file_header_sequential() {
        let mut data = Vec::new();
        data.extend_from_slice(b"\x97\x4a\x42\x32\x0d\x0a\x1a\x0a");
        data.push(0x03); // sequential + no num pages
        data.extend_from_slice(&make_segment_header(1, 0, 0));
        data.extend_from_slice(&make_segment_header(2, 51, 0)); // EOF
        let consumed = probe_stream_consumed_bytes(&data).expect("probe result");
        assert_eq!(consumed, Some(data.len()));
    }
}
