//! [`DopStream`]: whole-file streaming DoP word source.

use super::packer::DopPacker;
use super::dop_carrier_rate;
use crate::demux::{DsdDemuxer, DsdError};
use crate::dsd2pcm::bit_reverse;

/// Whole-file streaming DoP word source: demuxer → (bit reversal when the
/// container is LSB-first) → packer. Yields interleaved S32 samples at the
/// carrier rate. Stereo only — DoP receivers are 2-channel devices.
pub struct DopStream {
    demux: Box<dyn DsdDemuxer>,
    packer: DopPacker,
    lsb_first: bool,
    dsd_rate: u32,
    total_frames: u64,
    buf: Vec<i32>,
    idx: usize,
    done: bool,
    /// Set when demux I/O fails mid-stream (not clean EOF).
    io_error: Option<String>,
}

/// DSD bytes pulled from the demuxer per refill, per channel.
const REFILL_BYTES_PER_CH: usize = 32 * 1024;

impl DopStream {
    pub fn new(demux: Box<dyn DsdDemuxer>) -> Result<Self, DsdError> {
        let info = demux.info().clone();
        if info.channels != 2 {
            return Err(DsdError::UnsupportedChannels(info.channels));
        }
        Ok(Self {
            demux,
            packer: DopPacker::new(),
            lsb_first: info.lsb_first,
            dsd_rate: info.dsd_rate,
            total_frames: info.sample_count / 16,
            buf: Vec::new(),
            idx: 0,
            done: false,
            io_error: None,
        })
    }

    /// Mid-stream demux I/O error, if any. Clean EOF leaves this `None`.
    pub fn io_error(&self) -> Option<&str> {
        self.io_error.as_deref()
    }

    pub fn carrier_rate(&self) -> u32 {
        dop_carrier_rate(self.dsd_rate)
    }
    pub fn dsd_rate(&self) -> u32 {
        self.dsd_rate
    }
    /// Total DoP frames (per channel) this stream will yield.
    pub fn total_frames(&self) -> u64 {
        self.total_frames
    }

    fn refill(&mut self) -> bool {
        let mut planar: Vec<Vec<u8>> = vec![Vec::new(), Vec::new()];
        match self.demux.read_planar(&mut planar, REFILL_BYTES_PER_CH) {
            Ok(0) => {
                self.done = true;
                false
            }
            Err(e) => {
                log::error!("[DSD/DoP] demux I/O error (not clean EOF): {e}");
                self.io_error = Some(e.to_string());
                self.done = true;
                false
            }
            Ok(_) => {
                if self.lsb_first {
                    for chan in planar.iter_mut() {
                        for b in chan.iter_mut() {
                            *b = bit_reverse(*b);
                        }
                    }
                }
                self.buf.clear();
                self.idx = 0;
                self.packer.pack(&planar, &mut self.buf);
                !self.buf.is_empty()
            }
        }
    }
}

impl Iterator for DopStream {
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

#[cfg(test)]
#[path = "io_error_tests.rs"]
mod io_error_tests;
