//! PROP/SND subchunk scan (FS/CHNL/CMPR) used by [`super::open`].

use crate::demux::io::{read_id, read_u64_be};
use crate::demux::DsdError;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

/// Scan the PROP chunk's SND subchunks (FS/CHNL/CMPR), seeking past each
/// (even-padded) regardless of whether it's recognized.
pub(super) fn scan_prop_subchunks(
    file: &mut File,
    prop_end: u64,
) -> Result<(Option<u32>, Option<u16>), DsdError> {
    let mut dsd_rate = None;
    let mut channels = None;
    while file.stream_position()? + 12 <= prop_end {
        let sub_id = read_id(file)?;
        let sub_size = read_u64_be(file)?;
        let sub_start = file.stream_position()?;
        match &sub_id {
            b"FS  " => {
                let mut b = [0u8; 4];
                file.read_exact(&mut b)?;
                dsd_rate = Some(u32::from_be_bytes(b));
            }
            b"CHNL" => {
                let mut b = [0u8; 2];
                file.read_exact(&mut b)?;
                channels = Some(u16::from_be_bytes(b));
            }
            b"CMPR" => {
                let cmpr = read_id(file)?;
                if &cmpr == b"DST " {
                    return Err(DsdError::UnsupportedDst);
                }
                if &cmpr != b"DSD " {
                    return Err(DsdError::Corrupt(format!(
                        "unknown DFF compression {:?}",
                        String::from_utf8_lossy(&cmpr)
                    )));
                }
            }
            _ => {}
        }
        // Subchunks are even-padded too.
        let padded = sub_size + (sub_size & 1);
        file.seek(SeekFrom::Start(sub_start + padded))?;
    }
    Ok((dsd_rate, channels))
}
