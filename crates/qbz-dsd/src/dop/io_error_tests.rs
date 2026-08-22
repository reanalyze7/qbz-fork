//! Mid-stream demux I/O error handling for [`super::DopStream`].

use super::*;
use crate::demux::{DsdDemuxer, DsdError, DsdStreamInfo};
use std::io;

struct FailAfter {
    left: usize,
    info: DsdStreamInfo,
}

impl DsdDemuxer for FailAfter {
    fn info(&self) -> &DsdStreamInfo {
        &self.info
    }
    fn read_planar(
        &mut self,
        out: &mut [Vec<u8>],
        max_bytes_per_ch: usize,
    ) -> Result<usize, DsdError> {
        if self.left == 0 {
            return Err(DsdError::Io(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "simulated mid-file I/O error",
            )));
        }
        self.left -= 1;
        let n = max_bytes_per_ch.min(4);
        for ch in out.iter_mut().take(2) {
            ch.extend(std::iter::repeat(0x69).take(n));
        }
        Ok(n)
    }
}

#[test]
fn demux_io_error_sets_sticky_flag_not_clean_eof() {
    let info = DsdStreamInfo {
        channels: 2,
        dsd_rate: 2_822_400,
        sample_count: 1_000_000,
        lsb_first: false,
        tags: Default::default(),
    };
    let demux = FailAfter { left: 1, info };
    let mut stream = DopStream::new(Box::new(demux)).unwrap();
    // First refill succeeds; drain some samples.
    let _ = stream.next();
    // Force more refills until error.
    for _ in 0..100_000 {
        if stream.next().is_none() {
            break;
        }
    }
    assert!(stream.io_error().is_some(), "I/O error must be sticky");
    assert!(stream
        .io_error()
        .unwrap()
        .contains("simulated mid-file I/O error"));
}
