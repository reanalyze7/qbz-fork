//! Native DSD packing for ALSA's DSD_U32 formats (DSD plan Phase 3).
//!
//! One 32-bit sample per channel per frame carries 4 consecutive DSD bytes
//! (MSB-first bit order within each byte — same normalization the DoP path
//! uses). ALSA's frame rate is therefore dsd_rate / 32 (DSD64 → 88 200).
//!
//! Endianness (values below are what an i32 must hold on a little-endian
//! machine so the bytes land in the right memory order):
//! - `DSD_U32_BE` (what the kernel's generic DSD quirk grants USB DACs):
//!   temporally-first byte first in memory → `i32::from_le_bytes([b0..b3])`.
//! - `DSD_U32_LE`: first byte in the value's MSB, little-endian storage →
//!   `i32::from_be_bytes([b0..b3])`.
//!
//! Native DSD silence is the byte 0x69 in every lane — conveniently the
//! same i32 (0x69696969) in both layouts.

mod refill;
#[cfg(test)]
mod tests;

use crate::demux::{DsdDemuxer, DsdError};

/// ALSA frame rate for a DSD bit rate in U32 containers.
pub const fn native_u32_rate(dsd_rate: u32) -> u32 {
    dsd_rate / 32
}

/// Native DSD silence sample (0x69 in all four byte lanes).
pub const NATIVE_DSD_SILENCE_U32: i32 = 0x6969_6969;

/// Whole-file streaming native-DSD word source: demuxer → (bit reversal for
/// LSB-first containers) → 4-byte packing. Yields interleaved i32 samples at
/// [`native_u32_rate`]. Stereo only, mirroring the DoP path.
pub struct NativeDsdStream {
    demux: Box<dyn DsdDemuxer>,
    lsb_first: bool,
    little_endian: bool,
    dsd_rate: u32,
    total_frames: u64,
    buf: Vec<i32>,
    idx: usize,
    /// Carry-over bytes per channel when a demux read isn't a multiple of 4.
    carry: [Vec<u8>; 2],
    done: bool,
    /// Set when demux I/O fails mid-stream (not clean EOF).
    io_error: Option<String>,
}

const REFILL_BYTES_PER_CH: usize = 32 * 1024;

impl NativeDsdStream {
    /// `little_endian` selects DSD_U32_LE packing; false = DSD_U32_BE.
    pub fn new(demux: Box<dyn DsdDemuxer>, little_endian: bool) -> Result<Self, DsdError> {
        let info = demux.info().clone();
        if info.channels != 2 {
            return Err(DsdError::UnsupportedChannels(info.channels));
        }
        Ok(Self {
            demux,
            lsb_first: info.lsb_first,
            little_endian,
            dsd_rate: info.dsd_rate,
            total_frames: info.sample_count / 32,
            buf: Vec::new(),
            idx: 0,
            carry: [Vec::new(), Vec::new()],
            done: false,
            io_error: None,
        })
    }

    /// Mid-stream demux I/O error, if any. Clean EOF leaves this `None`.
    pub fn io_error(&self) -> Option<&str> {
        self.io_error.as_deref()
    }

    pub fn rate(&self) -> u32 {
        native_u32_rate(self.dsd_rate)
    }
    pub fn dsd_rate(&self) -> u32 {
        self.dsd_rate
    }
    pub fn total_frames(&self) -> u64 {
        self.total_frames
    }

    fn pack_word(&self, b: [u8; 4]) -> i32 {
        if self.little_endian {
            i32::from_be_bytes(b)
        } else {
            i32::from_le_bytes(b)
        }
    }
}

impl Iterator for NativeDsdStream {
    type Item = i32;
    fn next(&mut self) -> Option<i32> {
        loop {
            if self.idx < self.buf.len() {
                let v = self.buf[self.idx];
                self.idx += 1;
                return Some(v);
            }
            if self.done || !self.refill() {
                return None;
            }
        }
    }
}
