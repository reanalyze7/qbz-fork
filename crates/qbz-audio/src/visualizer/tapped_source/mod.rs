//! Tapped Audio Source
//!
//! A wrapper around any rodio Source that intercepts samples for visualization
//! without affecting audio playback. The tap is completely transparent to the
//! audio pipeline.

#[cfg(test)]
mod tests;

use rodio::Source;
use std::sync::Arc;
use std::time::Duration;

use super::RingBuffer;

/// Wraps a Source and sends samples to a ring buffer for visualization
pub struct TappedSource<S>
where
    S: Source<Item = f32>,
{
    inner: S,
    ring_buffer: Arc<RingBuffer>,
    enabled: Arc<std::sync::atomic::AtomicBool>,
}

impl<S> TappedSource<S>
where
    S: Source<Item = f32>,
{
    /// Create a new TappedSource wrapping the given source
    pub fn new(
        source: S,
        ring_buffer: Arc<RingBuffer>,
        enabled: Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        Self {
            inner: source,
            ring_buffer,
            enabled,
        }
    }
}

impl<S> Iterator for TappedSource<S>
where
    S: Source<Item = f32>,
{
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.inner.next()?;

        // Only send to visualizer if enabled - this is a fast atomic check
        if self.enabled.load(std::sync::atomic::Ordering::Relaxed) {
            // f32 samples are already normalized to [-1.0, 1.0]
            self.ring_buffer.push(sample);
        }

        Some(sample)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<S> Source for TappedSource<S>
where
    S: Source<Item = f32>,
{
    #[inline]
    fn current_span_len(&self) -> Option<usize> {
        self.inner.current_span_len()
    }

    #[inline]
    fn channels(&self) -> std::num::NonZero<u16> {
        self.inner.channels()
    }

    #[inline]
    fn sample_rate(&self) -> std::num::NonZero<u32> {
        self.inner.sample_rate()
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }
}
