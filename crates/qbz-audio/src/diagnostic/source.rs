//! Source wrapper — transparent tap for bit-depth capture

use std::time::Duration;

use rodio::Source;

use super::state::AudioDiagnostic;

pub struct DiagnosticSource<S>
where
    S: Source<Item = f32>,
{
    inner: S,
    diagnostic: AudioDiagnostic,
}

impl<S> DiagnosticSource<S>
where
    S: Source<Item = f32>,
{
    pub fn new(source: S, diagnostic: AudioDiagnostic) -> Self {
        Self {
            inner: source,
            diagnostic,
        }
    }
}

impl<S> Iterator for DiagnosticSource<S>
where
    S: Source<Item = f32>,
{
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.inner.next()?;
        self.diagnostic.push_sample(sample);
        Some(sample)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<S> Source for DiagnosticSource<S>
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
