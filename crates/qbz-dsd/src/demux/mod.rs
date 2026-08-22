//! DSF and DFF (DSDIFF) demuxers.
//!
//! Both containers carry raw 1-bit DSD grouped in bytes; they differ in
//! layout and bit order:
//! - DSF (Sony spec): per-channel BLOCKS of `block_size` (4096) bytes,
//!   `[ch0 block][ch1 block][ch0 block]…`; bit order declared in the header
//!   (`bits_per_sample`: 1 = LSB-first, 8 = MSB-first; LSB-first in practice).
//!   Standard ID3v2 tag at the header-declared `metadata_ptr` offset.
//! - DFF (Philips DSDIFF 1.5): frame-interleaved, ONE byte per channel round
//!   robin, always MSB-first. Big-endian chunk sizes, chunks padded to even
//!   offsets. No standard tagging; a trailing nonstandard "ID3 " chunk is
//!   honored when present. DST-compressed DFF is detected and rejected.

mod dff;
mod dsf;
mod io;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Errors from DSD demuxing/conversion. `UnsupportedDst` / `UnsupportedChannels`
/// are expected user-facing cases (toast + skip), not bugs.
#[derive(Debug, thiserror::Error)]
pub enum DsdError {
    #[error("DST-compressed DFF is not supported")]
    UnsupportedDst,
    #[error("unsupported channel count: {0} (mono, stereo and up to 5.1 supported)")]
    UnsupportedChannels(u16),
    #[error("unsupported DSD rate: {0} Hz")]
    UnsupportedRate(u32),
    #[error("corrupt or invalid DSD file: {0}")]
    Corrupt(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Tag subset read from the container (DSF: embedded ID3v2; DFF: trailing
/// "ID3 " chunk when present, otherwise empty — callers fall back to
/// filename-derived metadata).
#[derive(Debug, Clone, Default)]
pub struct DsdTags {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub year: Option<i32>,
    /// First embedded picture (front cover preferred), raw bytes.
    pub artwork: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct DsdStreamInfo {
    /// DSD bit rate per channel (2 822 400 = DSD64, …).
    pub dsd_rate: u32,
    pub channels: u16,
    /// Total DSD bits per channel.
    pub sample_count: u64,
    /// Bit order inside each byte: true = LSB is temporally first (DSF
    /// default), false = MSB first (DFF, and DSF with bits_per_sample = 8).
    pub lsb_first: bool,
    pub tags: DsdTags,
}

impl DsdStreamInfo {
    pub fn duration_secs(&self) -> u64 {
        if self.dsd_rate == 0 {
            0
        } else {
            self.sample_count / self.dsd_rate as u64
        }
    }
}

pub trait DsdDemuxer: Send {
    fn info(&self) -> &DsdStreamInfo;
    /// Append up to `max_bytes_per_ch` DSD bytes per channel to `out[ch]`
    /// (planar). Returns the byte count appended to EACH channel (always
    /// equal across channels); 0 means end of stream.
    fn read_planar(
        &mut self,
        out: &mut [Vec<u8>],
        max_bytes_per_ch: usize,
    ) -> Result<usize, DsdError>;
}

/// Open a DSD file, sniffing DSF vs DFF from the leading magic.
pub fn open_dsd(path: &Path) -> Result<Box<dyn DsdDemuxer>, DsdError> {
    let mut file = File::open(path)?;
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;
    file.seek(SeekFrom::Start(0))?;
    match &magic {
        b"DSD " => Ok(Box::new(dsf::DsfReader::open(file)?)),
        b"FRM8" => Ok(Box::new(dff::DffReader::open(file)?)),
        _ => Err(DsdError::Corrupt("not a DSF or DFF file".into())),
    }
}
