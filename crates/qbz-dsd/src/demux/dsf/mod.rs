//! DSF (Sony DSD Stream File) reader.

mod open;

use super::{DsdDemuxer, DsdError, DsdStreamInfo};
use std::fs::File;
use std::io::Read;

pub(super) struct DsfReader {
    file: File,
    info: DsdStreamInfo,
    block_size: usize,
    /// Valid (non-padding) DSD bytes remaining per channel.
    remaining_per_ch: u64,
}

impl DsdDemuxer for DsfReader {
    fn info(&self) -> &DsdStreamInfo {
        &self.info
    }

    fn read_planar(
        &mut self,
        out: &mut [Vec<u8>],
        max_bytes_per_ch: usize,
    ) -> Result<usize, DsdError> {
        debug_assert_eq!(out.len(), self.info.channels as usize);
        if self.remaining_per_ch == 0 {
            return Ok(0);
        }
        let mut appended = 0usize;
        let mut block = vec![0u8; self.block_size];
        while appended < max_bytes_per_ch && self.remaining_per_ch > 0 {
            // One block group: block_size bytes for each channel in order.
            let valid = (self.remaining_per_ch as usize).min(self.block_size);
            for ch in 0..self.info.channels as usize {
                match self.file.read_exact(&mut block) {
                    Ok(()) => out[ch].extend_from_slice(&block[..valid]),
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                        // Truncated file: stop at what we got.
                        self.remaining_per_ch = 0;
                        return Ok(appended);
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            self.remaining_per_ch -= valid as u64;
            appended += valid;
        }
        Ok(appended)
    }
}
