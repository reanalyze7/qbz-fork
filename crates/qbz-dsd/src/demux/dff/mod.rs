//! DFF (Philips DSDIFF 1.5) reader.

mod open;
mod prop;

use super::{DsdDemuxer, DsdError, DsdStreamInfo};
use std::fs::File;
use std::io::Read;

pub(super) struct DffReader {
    file: File,
    info: DsdStreamInfo,
    /// Bytes (all channels interleaved) remaining in the DSD data chunk.
    remaining_total: u64,
}

impl DsdDemuxer for DffReader {
    fn info(&self) -> &DsdStreamInfo {
        &self.info
    }

    fn read_planar(
        &mut self,
        out: &mut [Vec<u8>],
        max_bytes_per_ch: usize,
    ) -> Result<usize, DsdError> {
        let ch = self.info.channels as usize;
        debug_assert_eq!(out.len(), ch);
        if self.remaining_total == 0 {
            return Ok(0);
        }
        // Whole frames only (one byte per channel).
        let want_total = (max_bytes_per_ch * ch).min(self.remaining_total as usize);
        let want_total = want_total - (want_total % ch);
        if want_total == 0 {
            self.remaining_total = 0;
            return Ok(0);
        }
        let mut buf = vec![0u8; want_total];
        let mut filled = 0usize;
        while filled < want_total {
            match self.file.read(&mut buf[filled..]) {
                Ok(0) => break,
                Ok(n) => filled += n,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e.into()),
            }
        }
        let frames = filled / ch;
        for f in 0..frames {
            for c in 0..ch {
                out[c].push(buf[f * ch + c]);
            }
        }
        self.remaining_total = if filled < want_total {
            0
        } else {
            self.remaining_total - filled as u64
        };
        Ok(frames)
    }
}
