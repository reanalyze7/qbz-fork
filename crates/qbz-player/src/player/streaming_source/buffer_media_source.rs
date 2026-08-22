//! `symphonia::core::io::MediaSource` impl for `BufferedMediaSource`.

use symphonia::core::io::MediaSource;

use super::buffer::BufferedMediaSource;

impl MediaSource for BufferedMediaSource {
    fn is_seekable(&self) -> bool {
        // We support seeking within buffered data
        true
    }

    fn byte_len(&self) -> Option<u64> {
        let (lock, _) = &*self.state;
        if let Ok(state) = lock.lock() {
            state.total_size
        } else {
            None
        }
    }
}
