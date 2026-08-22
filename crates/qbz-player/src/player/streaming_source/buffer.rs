//! `BufferState` and the `BufferedMediaSource` struct: construction and
//! read-only accessors. The blocking `Read`/`Seek`/`MediaSource` trait
//! impls live in `buffer_io.rs`.

use std::sync::{Arc, Condvar, Mutex};

use super::config::StreamingConfig;
use super::writer::BufferWriter;

/// Internal state shared between reader and writer
pub(super) struct BufferState {
    /// Accumulated data from HTTP response
    pub(super) data: Vec<u8>,
    /// True when HTTP download is complete
    pub(super) download_complete: bool,
    /// Error from download, if any
    pub(super) download_error: Option<String>,
    /// Total expected size (from Content-Length), if known
    pub(super) total_size: Option<u64>,
}

/// A media source that buffers from an async HTTP stream.
///
/// Provides `Read + Seek` interface for decoders while data is still downloading.
/// The source is created with a `BufferWriter` that receives chunks from the
/// download task.
pub struct BufferedMediaSource {
    pub(super) state: Arc<(Mutex<BufferState>, Condvar)>,
    pub(super) config: StreamingConfig,
    /// Each reader has its own read position
    pub(super) read_pos: std::sync::atomic::AtomicU64,
}

impl BufferedMediaSource {
    /// Create a new buffered source.
    ///
    /// Returns the source and a writer for pushing downloaded chunks.
    /// The writer should be used from the async download task.
    pub fn new(config: StreamingConfig, total_size: Option<u64>) -> (Self, BufferWriter) {
        let state = Arc::new((
            Mutex::new(BufferState {
                data: Vec::with_capacity(config.initial_buffer_bytes),
                download_complete: false,
                download_error: None,
                total_size,
            }),
            Condvar::new(),
        ));

        let source = Self {
            state: Arc::clone(&state),
            config: config.clone(),
            read_pos: std::sync::atomic::AtomicU64::new(0),
        };

        let writer = BufferWriter::new(state);

        (source, writer)
    }

    /// Create a new reader that shares the same buffer but has its own read position.
    /// This is used to pass to symphonia which needs ownership of the reader.
    pub fn create_reader(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            config: self.config.clone(),
            read_pos: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Wait until initial buffer is filled or download completes.
    ///
    /// This should be called before passing the source to the decoder,
    /// to ensure enough data is available for format detection.
    ///
    /// Returns error if download fails before initial buffer is filled.
    pub fn wait_for_initial_buffer(&self) -> std::io::Result<()> {
        use std::io::{Error as IoError, ErrorKind};

        let (lock, cvar) = &*self.state;
        let mut state = lock
            .lock()
            .map_err(|_| IoError::new(ErrorKind::Other, "Failed to acquire buffer lock"))?;

        while state.data.len() < self.config.initial_buffer_bytes
            && !state.download_complete
            && state.download_error.is_none()
        {
            state = cvar
                .wait(state)
                .map_err(|_| IoError::new(ErrorKind::Other, "Condition variable wait failed"))?;
        }

        if let Some(ref err) = state.download_error {
            return Err(IoError::new(ErrorKind::Other, err.clone()));
        }

        Ok(())
    }

}
