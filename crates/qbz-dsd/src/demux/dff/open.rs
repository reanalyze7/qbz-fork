//! `DffReader::open`: walks the FRM8 top-level chunk list (PROP/SND, DSD,
//! DST, ID3) and, within PROP, the FS/CHNL/CMPR subchunks.

use super::prop::scan_prop_subchunks;
use super::DffReader;
use crate::demux::io::{read_id, read_id3_tags, read_u64_be, validate_rate};
use crate::demux::{DsdError, DsdStreamInfo, DsdTags};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

impl DffReader {
    pub(in crate::demux) fn open(mut file: File) -> Result<Self, DsdError> {
        let id = read_id(&mut file)?;
        if &id != b"FRM8" {
            return Err(DsdError::Corrupt("missing FRM8 chunk".into()));
        }
        let _form_size = read_u64_be(&mut file)?;
        let form_type = read_id(&mut file)?;
        if &form_type != b"DSD " {
            return Err(DsdError::Corrupt("FRM8 form type is not DSD".into()));
        }

        let mut dsd_rate: Option<u32> = None;
        let mut channels: Option<u16> = None;
        let mut data: Option<(u64, u64)> = None; // (offset, size)
        let mut id3_offset: Option<u64> = None;

        // Top-level chunk scan (seek past payloads; chunks are even-padded).
        loop {
            let mut idbuf = [0u8; 4];
            match file.read_exact(&mut idbuf) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            }
            let size = read_u64_be(&mut file)?;
            let payload_start = file.stream_position()?;
            match &idbuf {
                b"PROP" => {
                    // Property container: "SND " + subchunks.
                    let prop_type = read_id(&mut file)?;
                    if &prop_type != b"SND " {
                        return Err(DsdError::Corrupt("PROP type is not SND".into()));
                    }
                    let (rate, ch) = scan_prop_subchunks(&mut file, payload_start + size)?;
                    dsd_rate = dsd_rate.or(rate);
                    channels = channels.or(ch);
                }
                b"DSD " => {
                    data = Some((payload_start, size));
                }
                b"DST " => return Err(DsdError::UnsupportedDst),
                b"ID3 " => {
                    id3_offset = Some(payload_start);
                }
                _ => {}
            }
            let padded = size + (size & 1);
            file.seek(SeekFrom::Start(payload_start + padded))?;
        }

        let dsd_rate = dsd_rate.ok_or_else(|| DsdError::Corrupt("DFF missing FS".into()))?;
        let channels = channels.ok_or_else(|| DsdError::Corrupt("DFF missing CHNL".into()))?;
        let (data_offset, data_size) =
            data.ok_or_else(|| DsdError::Corrupt("DFF missing DSD data".into()))?;
        validate_rate(dsd_rate)?;
        if !(1..=6).contains(&channels) {
            return Err(DsdError::UnsupportedChannels(channels));
        }

        let tags = match id3_offset {
            Some(off) => read_id3_tags(&mut file, off),
            None => DsdTags::default(),
        };

        file.seek(SeekFrom::Start(data_offset))?;
        let sample_count = data_size / channels as u64 * 8;

        Ok(Self {
            file,
            remaining_total: data_size,
            info: DsdStreamInfo {
                dsd_rate,
                channels,
                sample_count,
                lsb_first: false,
                tags,
            },
        })
    }
}
