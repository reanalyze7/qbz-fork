//! `DsfReader::open`: parses the DSF chunk header (DSD/fmt /data) and,
//! optionally, the embedded ID3v2 tag at `metadata_ptr`.

use super::DsfReader;
use crate::demux::io::{read_id, read_id3_tags, read_u32_le, read_u64_le, validate_rate};
use crate::demux::{DsdError, DsdStreamInfo, DsdTags};
use std::fs::File;
use std::io::{Seek, SeekFrom};

impl DsfReader {
    pub(in crate::demux) fn open(mut file: File) -> Result<Self, DsdError> {
        // "DSD " chunk: magic + size(28) + total file size + metadata ptr.
        let id = read_id(&mut file)?;
        if &id != b"DSD " {
            return Err(DsdError::Corrupt("missing DSD chunk".into()));
        }
        let dsd_chunk_size = read_u64_le(&mut file)?;
        if dsd_chunk_size != 28 {
            return Err(DsdError::Corrupt(format!(
                "bad DSD chunk size {dsd_chunk_size}"
            )));
        }
        let _total_size = read_u64_le(&mut file)?;
        let metadata_ptr = read_u64_le(&mut file)?;

        // "fmt " chunk.
        let id = read_id(&mut file)?;
        if &id != b"fmt " {
            return Err(DsdError::Corrupt("missing fmt chunk".into()));
        }
        let fmt_size = read_u64_le(&mut file)?;
        if fmt_size < 52 {
            return Err(DsdError::Corrupt(format!("bad fmt chunk size {fmt_size}")));
        }
        let format_version = read_u32_le(&mut file)?;
        let format_id = read_u32_le(&mut file)?;
        let _channel_type = read_u32_le(&mut file)?;
        let channel_num = read_u32_le(&mut file)?;
        let sampling_frequency = read_u32_le(&mut file)?;
        let bits_per_sample = read_u32_le(&mut file)?;
        let sample_count = read_u64_le(&mut file)?;
        let block_size = read_u32_le(&mut file)?;
        let _reserved = read_u32_le(&mut file)?;

        if format_version != 1 || format_id != 0 {
            return Err(DsdError::Corrupt(format!(
                "unsupported DSF format version {format_version} / id {format_id}"
            )));
        }
        if !(1..=6).contains(&channel_num) {
            return Err(DsdError::UnsupportedChannels(channel_num as u16));
        }
        validate_rate(sampling_frequency)?;
        let lsb_first = match bits_per_sample {
            1 => true,
            8 => false,
            other => {
                return Err(DsdError::Corrupt(format!(
                    "bad DSF bits_per_sample {other}"
                )))
            }
        };
        if block_size == 0 || block_size > (1 << 20) {
            return Err(DsdError::Corrupt(format!("bad DSF block size {block_size}")));
        }

        // "data" chunk header; sample data starts right after.
        let id = read_id(&mut file)?;
        if &id != b"data" {
            return Err(DsdError::Corrupt("missing data chunk".into()));
        }
        let _data_chunk_size = read_u64_le(&mut file)?;
        let data_start = file.stream_position()?;

        let tags = if metadata_ptr != 0 {
            let t = read_id3_tags(&mut file, metadata_ptr);
            file.seek(SeekFrom::Start(data_start))?;
            t
        } else {
            DsdTags::default()
        };

        Ok(Self {
            file,
            block_size: block_size as usize,
            remaining_per_ch: sample_count.div_ceil(8),
            info: DsdStreamInfo {
                dsd_rate: sampling_frequency,
                channels: channel_num as u16,
                sample_count,
                lsb_first,
                tags,
            },
        })
    }
}
