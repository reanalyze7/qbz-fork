//! `Cursor`-backed `MediaSource` for fully-in-memory audio data, used by
//! `InMemorySource` to feed Symphonia.

use std::io::{Cursor, Read, Result as IoResult, Seek, SeekFrom};

use symphonia::core::io::MediaSource;

/// Cursor-backed MediaSource for in-memory audio data.
pub(super) struct InMemoryMediaSource {
    inner: Cursor<Vec<u8>>,
    len: u64,
}

impl InMemoryMediaSource {
    pub(super) fn new(data: Vec<u8>) -> Self {
        let len = data.len() as u64;
        Self {
            inner: Cursor::new(data),
            len,
        }
    }
}

impl Read for InMemoryMediaSource {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.inner.read(buf)
    }
}

impl Seek for InMemoryMediaSource {
    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
        self.inner.seek(pos)
    }
}

impl MediaSource for InMemoryMediaSource {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        Some(self.len)
    }
}
