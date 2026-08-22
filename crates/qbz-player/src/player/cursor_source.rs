use super::*;

pub(crate) struct CursorMediaSource {
    inner: Cursor<Vec<u8>>,
    len: u64,
}

impl CursorMediaSource {
    pub(crate) fn new(data: Vec<u8>) -> Self {
        let len = data.len() as u64;
        Self {
            inner: Cursor::new(data),
            len,
        }
    }
}

impl MediaSource for CursorMediaSource {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        Some(self.len)
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

/// Audio specifications extracted from decoded audio
#[allow(dead_code)]
pub(crate) struct AudioSpecs {
    pub(crate) samples: SamplesBuffer,
    pub(crate) sample_rate: u32,
    pub(crate) channels: u16,
}
