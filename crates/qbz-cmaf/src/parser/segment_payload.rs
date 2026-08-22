use crate::error::CmafError;

use super::{FrameEntry, SegmentCrypto};

pub(super) fn parse_segment_uuid_payload(
    data: &[u8],
    uuid_box_start: usize,
    mdat_end: usize,
) -> Result<SegmentCrypto, CmafError> {
    // Layout after box header (8) + UUID (16) = offset 24 from uuid_box_start:
    //   [4B version/padding]
    //   [4B data_offset_raw]   — offset from uuid_box_start to audio data
    //   [1B iv_size]
    //   [3B frame_count (24-bit BE)]
    //   Per frame: [4B size][2B skip][2B flags][iv_size bytes IV]

    let base = uuid_box_start + 24; // start of payload after UUID
    if base + 12 > data.len() {
        return Err(CmafError::ParseError(
            "segment UUID payload too short for header".into(),
        ));
    }

    let mut a = base + 4; // skip 4-byte version/padding

    let data_offset_raw = u32::from_be_bytes([data[a], data[a + 1], data[a + 2], data[a + 3]]);
    let data_offset = uuid_box_start + data_offset_raw as usize;
    a += 4;

    let iv_size = data[a] as usize;
    a += 1;

    let frame_count =
        ((data[a] as usize) << 16) | ((data[a + 1] as usize) << 8) | (data[a + 2] as usize);
    a += 3;

    let entry_size = 4 + 2 + 2 + iv_size; // size + skip + flags + iv
    if a + frame_count * entry_size > data.len() {
        return Err(CmafError::ParseError(format!(
            "segment UUID: not enough data for {frame_count} entries of {entry_size} bytes"
        )));
    }

    let mut entries = Vec::with_capacity(frame_count);
    for _ in 0..frame_count {
        let size = u32::from_be_bytes([data[a], data[a + 1], data[a + 2], data[a + 3]]);
        a += 4;
        a += 2; // skip 2 unknown bytes
        let flags = u16::from_be_bytes([data[a], data[a + 1]]);
        a += 2;

        let mut iv = [0u8; 8];
        let copy_len = iv_size.min(8);
        iv[..copy_len].copy_from_slice(&data[a..a + copy_len]);
        a += iv_size;

        entries.push(FrameEntry { size, flags, iv });
    }

    Ok(SegmentCrypto { data_offset, mdat_end, entries })
}
