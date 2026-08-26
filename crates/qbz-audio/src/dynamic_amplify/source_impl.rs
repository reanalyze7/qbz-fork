//! Implementations `Iterator` / `Source` : application du gain echantillon
//! par echantillon, et delegation transparente des metadonnees de la source.

use std::time::Duration;

use rodio::Source;

use super::DynamicAmplify;

impl<S> Iterator for DynamicAmplify<S>
where
    S: Source<Item = f32>,
{
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Poll for new gain every 1024 samples to avoid atomic contention
        // (ramp_remaining check is essentially free)
        if self.ramp_remaining == 0 {
            self.poll_gain();
        }

        let sample = self.inner.next()?;

        if self.ramp_remaining > 0 {
            self.current_gain += self.ramp_step;
            self.ramp_remaining -= 1;
            if self.ramp_remaining == 0 {
                // Snap to target at end of ramp to avoid float drift
                self.current_gain = self.target_gain;
            }
        }

        Some(sample * self.current_gain)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<S> Source for DynamicAmplify<S>
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
