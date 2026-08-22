use crate::error::CmafError;

use super::{InitInfo, SegmentTableEntry, FLAC_MAGIC};

pub(super) fn parse_init_uuid_payload(payload: &[u8]) -> Result<InitInfo, CmafError> {
    // Payload layout:
    //   [4B padding/version]
    //   [4B track_id]
    //   [4B file_id]
    //   [4B sample_rate]
    //   [1B bits_per_sample]
    //   [1B channels + 2B padding]
    //   [6B total_samples_count]
    //   [2B raw_data_len]
    //   [raw_data_len bytes: contains FLAC header]
    //   [1B key_id_len]
    //   [key_id_len bytes: key_id]
    //   [2B segment_count]
    //   Per segment: [4B byte_len][4B sample_count]

    if payload.len() < 28 {
        return Err(CmafError::ParseError("init UUID payload too short".into()));
    }

    let mut a = 4; // skip version/padding
    a += 4; // track_id
    a += 4; // file_id
    a += 4; // sample_rate
    a += 1; // bits_per_sample
    a += 3; // channels + padding
    a += 6; // total_samples_count

    if a + 2 > payload.len() {
        return Err(CmafError::ParseError("init UUID payload truncated at raw_len".into()));
    }
    let raw_len = u16::from_be_bytes([payload[a], payload[a + 1]]) as usize;
    a += 2;

    if a + raw_len > payload.len() {
        return Err(CmafError::ParseError(format!(
            "init UUID payload truncated: need {raw_len} raw bytes, have {}",
            payload.len().saturating_sub(a)
        )));
    }
    let raw_data = &payload[a..a + raw_len];
    a += raw_len;

    let flac_pos = raw_data
        .windows(4)
        .position(|w| w == FLAC_MAGIC)
        .ok_or_else(|| CmafError::ParseError("init UUID payload: fLaC magic not found".into()))?;

    // fLaC (4) + STREAMINFO block header (4) + STREAMINFO data (34) = 42 bytes
    let header_len = 4 + 4 + 34;
    if flac_pos + header_len > raw_data.len() {
        return Err(CmafError::ParseError("init UUID payload: STREAMINFO truncated".into()));
    }

    let mut flac_header = raw_data[flac_pos..flac_pos + header_len].to_vec();
    // Set last-metadata-block flag in block header byte
    flac_header[4] |= 0x80;

    if a + 1 > payload.len() {
        return Ok(InitInfo {
            flac_header,
            segment_table: Vec::new(),
        });
    }
    let key_id_len = payload[a] as usize;
    a += 1 + key_id_len;

    let mut segment_table = Vec::new();
    if a + 2 <= payload.len() {
        let seg_count = u16::from_be_bytes([payload[a], payload[a + 1]]) as usize;
        a += 2;

        for _ in 0..seg_count {
            if a + 8 > payload.len() {
                break;
            }
            let byte_len =
                u32::from_be_bytes([payload[a], payload[a + 1], payload[a + 2], payload[a + 3]]);
            a += 4;
            let sample_count =
                u32::from_be_bytes([payload[a], payload[a + 1], payload[a + 2], payload[a + 3]]);
            a += 4;
            segment_table.push(SegmentTableEntry { byte_len, sample_count });
        }
    }

    log::debug!(
        "Init UUID: {} segments in table, FLAC header {} bytes",
        segment_table.len(),
        flac_header.len()
    );

    Ok(InitInfo { flac_header, segment_table })
}
