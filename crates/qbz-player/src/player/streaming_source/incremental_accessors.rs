//! Read-only accessors and `seek_to` on `IncrementalStreamingSource`.
//! Split out from `incremental.rs` purely for line budget — same type,
//! same `impl` semantics.

use std::sync::Arc;
use std::time::Duration;

use symphonia::core::formats::{SeekMode, SeekTo};

use super::buffer::BufferedMediaSource;
use super::incremental::IncrementalStreamingSource;

impl IncrementalStreamingSource {
    /// Get the sample rate
    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get the number of channels
    pub fn get_channels(&self) -> u16 {
        self.channels
    }

    /// Get reference to the buffered source for cache retrieval
    pub fn buffered_source(&self) -> &Arc<BufferedMediaSource> {
        &self.buffered_source
    }

    /// Seek the decoder to the given time using Symphonia's native seek.
    ///
    /// For FLAC this uses the seek table to jump directly to the nearest
    /// seek point, then decodes forward to the exact sample — far cheaper
    /// than skip_duration's decode-every-sample-from-zero path. For MP3
    /// with Xing/VBRI headers it uses the TOC; without headers, Symphonia
    /// falls back to a binary search, still much cheaper than linear decode.
    ///
    /// The underlying BufferedMediaSource::seek is the I/O target. If the
    /// requested byte offset isn't buffered yet it will block on the
    /// condition variable — callers must only invoke this for times within
    /// the downloaded watermark.
    pub fn seek_to(&mut self, time: Duration) -> Result<(), String> {
        self.format
            .seek(
                SeekMode::Accurate,
                SeekTo::Time {
                    time: time.into(),
                    track_id: Some(self.track_id),
                },
            )
            .map_err(|e| format!("Symphonia seek failed: {}", e))?;
        self.decoder.reset();
        self.sample_queue.clear();
        self.packets_decoded = 0;
        self.finished = false;
        Ok(())
    }
}
