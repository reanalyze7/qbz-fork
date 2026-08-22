//! Read-only accessors on `BufferedMediaSource` (status/progress queries).
//! Split out from `buffer.rs` (construction) purely to stay under the
//! per-file line budget — same type, same `impl` semantics.

use super::buffer::BufferedMediaSource;

impl BufferedMediaSource {
    /// Check if download is complete (full file in buffer)
    pub fn is_complete(&self) -> bool {
        let (lock, _) = &*self.state;
        if let Ok(state) = lock.lock() {
            state.download_complete && state.download_error.is_none()
        } else {
            false
        }
    }

    /// Get current buffer size in bytes
    pub fn buffer_size(&self) -> usize {
        let (lock, _) = &*self.state;
        if let Ok(state) = lock.lock() {
            state.data.len()
        } else {
            0
        }
    }

    /// Get the complete data if download finished successfully.
    ///
    /// Used to store in cache after streaming playback completes.
    /// Returns None if download is not complete or failed.
    ///
    /// IMPORTANT: This clones the buffer rather than moving it. Earlier
    /// attempts to use `mem::take` here regressed playback — the
    /// `Source` impl is still actively reading from `state.data` when
    /// the promotion path calls this, and zeroing the buffer out from
    /// under the reader caused immediate EOF (tracks ending 10s into a
    /// 104s file). Cloning is the safe choice; the audible hiccup at
    /// promotion that the move attempted to fix needs a different
    /// approach (shared `Arc<Vec<u8>>` ownership, or off-thread copy).
    pub fn take_complete_data(&self) -> Option<Vec<u8>> {
        let (lock, _) = &*self.state;
        if let Ok(state) = lock.lock() {
            if state.download_complete && state.download_error.is_none() {
                Some(state.data.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get a copy of the currently buffered data (for metadata extraction).
    ///
    /// Returns whatever data has been downloaded so far, even if incomplete.
    /// Useful for extracting file-level metadata (e.g., ReplayGain tags)
    /// which are typically in the first few KB of the file.
    pub fn get_buffered_data(&self) -> Option<Vec<u8>> {
        let (lock, _) = &*self.state;
        if let Ok(state) = lock.lock() {
            if state.data.is_empty() {
                None
            } else {
                Some(state.data.clone())
            }
        } else {
            None
        }
    }

    /// Get download progress as a fraction (0.0 to 1.0)
    ///
    /// Returns None if total size is unknown
    pub fn progress(&self) -> Option<f32> {
        let (lock, _) = &*self.state;
        if let Ok(state) = lock.lock() {
            state.total_size.map(|total| {
                if total == 0 {
                    1.0
                } else {
                    state.data.len() as f32 / total as f32
                }
            })
        } else {
            None
        }
    }

    /// Check if minimum buffer for playback is available
    ///
    /// Returns true when initial_buffer_bytes have been buffered
    /// or the download is complete.
    pub fn has_min_buffer(&self) -> bool {
        let (lock, _) = &*self.state;
        if let Ok(state) = lock.lock() {
            state.data.len() >= self.config.initial_buffer_bytes || state.download_complete
        } else {
            false
        }
    }

    /// Error reported by the feeder, if any. Lets waiters (the initial
    /// buffer fill loop) bail out immediately instead of sitting through
    /// the full buffer timeout when the feeder has already died.
    pub fn download_error(&self) -> Option<String> {
        let (lock, _) = &*self.state;
        lock.lock().ok().and_then(|s| s.download_error.clone())
    }
}
