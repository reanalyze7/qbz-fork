//! Buffered media source for streaming playback.
//!
//! Provides two main components:
//! 1. `BufferedMediaSource` - Wraps an async HTTP response to provide a synchronous
//!    `Read + Seek` interface required by symphonia decoders.
//! 2. `IncrementalStreamingSource` - A rodio Source that decodes audio packets
//!    incrementally as they become available, allowing playback to start before
//!    the entire file is downloaded.
//!
//! # Design
//!
//! The source uses a growing buffer that accumulates data from the HTTP response.
//! - Reads block if requesting data not yet buffered
//! - Seek forward blocks until data is available
//! - Seek backward works within buffered data
//! - Seek beyond current buffer position blocks until data arrives
//!
//! # Thread Safety
//!
//! The buffer state is shared between:
//! - The reader (audio thread, synchronous)
//! - The writer (download task, async)
//!
//! Communication uses `Mutex` + `Condvar` for blocking synchronization.

mod buffer;
mod buffer_accessors;
mod buffer_io;
mod buffer_media_source;
mod cap;
mod config;
mod in_memory;
mod in_memory_decode;
mod in_memory_media_source;
mod incremental;
mod incremental_accessors;
mod incremental_decode;
mod writer;

#[cfg(test)]
mod tests;

pub use buffer::BufferedMediaSource;
pub use cap::{max_initial_buffer_bytes, set_max_initial_buffer_bytes};
pub use config::StreamingConfig;
pub use in_memory::InMemorySource;
pub use incremental::IncrementalStreamingSource;
pub use writer::BufferWriter;
