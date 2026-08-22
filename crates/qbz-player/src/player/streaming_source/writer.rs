//! `BufferWriter` — the async-side chunk pusher that feeds a
//! `BufferedMediaSource`'s shared buffer.

use std::sync::{Arc, Condvar, Mutex};

use super::buffer::BufferState;

/// Writer half for pushing downloaded chunks from the async download task.
///
/// This is the sender side that receives data from the HTTP response
/// and makes it available to the `BufferedMediaSource` reader.
#[derive(Clone)]
pub struct BufferWriter {
    state: Arc<(Mutex<BufferState>, Condvar)>,
}

impl BufferWriter {
    pub(super) fn new(state: Arc<(Mutex<BufferState>, Condvar)>) -> Self {
        Self { state }
    }

    /// Push a chunk of downloaded data
    ///
    /// This wakes up any readers waiting for data.
    pub fn push_chunk(&self, chunk: &[u8]) -> Result<(), String> {
        let (lock, cvar) = &*self.state;
        let mut state = lock.lock().map_err(|_| "Failed to acquire buffer lock")?;

        state.data.extend_from_slice(chunk);
        cvar.notify_all();

        Ok(())
    }

    /// Mark download as complete
    ///
    /// After this is called, readers will receive EOF after reading all buffered data.
    pub fn complete(&self) -> Result<(), String> {
        let (lock, cvar) = &*self.state;
        let mut state = lock.lock().map_err(|_| "Failed to acquire buffer lock")?;

        state.download_complete = true;
        cvar.notify_all();

        Ok(())
    }

    /// Mark download as failed
    ///
    /// After this is called, readers will receive the error on next read.
    /// The first recorded error wins: it is the root cause, and the feeder
    /// fail-guards fire a generic "aborted" error on drop after a specific
    /// failure has already been recorded, which must not overwrite it.
    pub fn error(&self, err: String) -> Result<(), String> {
        let (lock, cvar) = &*self.state;
        let mut state = lock.lock().map_err(|_| "Failed to acquire buffer lock")?;

        if state.download_error.is_none() {
            state.download_error = Some(err);
        }
        cvar.notify_all();

        Ok(())
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
}
