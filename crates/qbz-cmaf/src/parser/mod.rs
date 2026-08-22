//! Pure ISO-BMFF/CMAF box-walking parser for Qobuz's proprietary
//! FLAC-in-CMAF container: extracts the FLAC STREAMINFO header +
//! per-segment table from the init segment, and per-frame crypto info
//! (offsets/IVs) from audio segments.

use crate::error::CmafError;

mod boxes;
mod init_payload;
mod segment_payload;

#[cfg(test)]
mod tests;

use boxes::{find_uuid_box, read_box_size};
use init_payload::parse_init_uuid_payload;
use segment_payload::parse_segment_uuid_payload;

const QBZ_INIT_UUID: [u8; 16] = [
    0xc7, 0xc7, 0x5d, 0xf0, 0xfd, 0xd9, 0x51, 0xe9,
    0x8f, 0xc2, 0x29, 0x71, 0xe4, 0xac, 0xf8, 0xd2,
];
const QBZ_SEGMENT_UUID: [u8; 16] = [
    0x3b, 0x42, 0x12, 0x92, 0x56, 0xf3, 0x5f, 0x75,
    0x92, 0x36, 0x63, 0xb6, 0x9a, 0x1f, 0x52, 0xb2,
];
const FLAC_MAGIC: &[u8; 4] = b"fLaC";

/// Info about one segment from the init segment's segment table.
#[derive(Debug, Clone)]
pub struct SegmentTableEntry {
    /// Byte size of this segment's decrypted FLAC frame data.
    pub byte_len: u32,
    /// Number of audio samples in this segment.
    pub sample_count: u32,
}

/// FLAC header and segment table extracted from the init segment.
pub struct InitInfo {
    pub flac_header: Vec<u8>,
    /// Per-segment sizes (indices 0..n_segments-1 correspond to segments 1..n_segments).
    pub segment_table: Vec<SegmentTableEntry>,
}

/// One frame entry from the segment's QBZ_SEGMENT_UUID box.
pub struct FrameEntry {
    pub size: u32,
    pub flags: u16,
    pub iv: [u8; 8],
}

/// Parsed crypto info from a segment's QBZ_SEGMENT_UUID box.
pub struct SegmentCrypto {
    /// Offset to the start of audio frame data (usually mdat payload).
    pub data_offset: usize,
    /// End of the mdat box content. Data between the last frame entry and this
    /// offset is unencrypted trailing audio that must be included in output.
    pub mdat_end: usize,
    pub entries: Vec<FrameEntry>,
}

/// Parse the init segment (segment 0) to extract the FLAC header and segment table.
pub fn parse_init_segment(data: &[u8]) -> Result<InitInfo, CmafError> {
    let (payload_start, box_end) = find_uuid_box(data, &QBZ_INIT_UUID)
        .ok_or_else(|| CmafError::ParseError("init segment: QBZ_INIT_UUID box not found".into()))?;

    let payload = &data[payload_start..box_end];
    parse_init_uuid_payload(payload)
}

/// Parse an audio segment to extract per-frame crypto info.
pub fn parse_segment_crypto(data: &[u8]) -> Result<SegmentCrypto, CmafError> {
    // Walk all top-level boxes to find both UUID and mdat boxes.
    let mut uuid_box_start: Option<usize> = None;
    let mut mdat_end = data.len();

    let mut pos = 0;
    while pos + 8 <= data.len() {
        let size = read_box_size(data, pos);
        if size < 8 || pos + size > data.len() {
            break;
        }
        let box_type = &data[pos + 4..pos + 8];
        if box_type == b"uuid" && pos + 24 <= data.len() {
            if &data[pos + 8..pos + 24] == QBZ_SEGMENT_UUID.as_ref() {
                uuid_box_start = Some(pos);
            }
        } else if box_type == b"mdat" {
            mdat_end = pos + size;
        }
        pos += size;
    }

    let box_start = uuid_box_start
        .ok_or_else(|| CmafError::ParseError("audio segment: QBZ_SEGMENT_UUID box not found".into()))?;

    parse_segment_uuid_payload(data, box_start, mdat_end)
}
