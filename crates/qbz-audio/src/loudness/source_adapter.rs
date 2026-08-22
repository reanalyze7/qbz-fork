//! Wrapper to make Cursor<Vec<u8>> implement MediaSource

use std::io::{Cursor, Read, Seek, SeekFrom};
use symphonia::core::io::MediaSource;

pub(super) struct CursorMediaSource {
    inner: Cursor<Vec<u8>>,
}

impl CursorMediaSource {
    pub(super) fn new(data: Vec<u8>) -> Self {
        Self {
            inner: Cursor::new(data),
        }
    }
}

impl Read for CursorMediaSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl Seek for CursorMediaSource {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(pos)
    }
}

impl MediaSource for CursorMediaSource {
    fn is_seekable(&self) -> bool {
        true
    }
    fn byte_len(&self) -> Option<u64> {
        Some(self.inner.get_ref().len() as u64)
    }
}
