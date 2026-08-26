//! Dynamic gain wrapper for real-time volume normalization.
//!
//! Reads gain from a shared `Arc<AtomicU32>` (f32 bits) and applies it to
//! each sample with a 50ms linear ramp to avoid audible clicks. When the
//! atomic holds 0.0 (not yet computed), stays at the initial gain.
//!
//! Un gain deja connu au moment de construire la source est applique tel quel,
//! sans rampe : le morceau doit partir au bon niveau des la premiere note.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use rodio::Source;

mod source_impl;

#[cfg(test)]
mod tests;

pub struct DynamicAmplify<S>
where
    S: Source<Item = f32>,
{
    pub(super) inner: S,
    pub(super) gain_atomic: Arc<AtomicU32>,
    pub(super) current_gain: f32,   // Current applied gain (smoothly ramped)
    pub(super) target_gain: f32,    // Target gain we're ramping toward
    pub(super) ramp_step: f32,      // Gain increment per sample during ramp
    pub(super) ramp_remaining: u32, // Samples remaining in the current ramp
    pub(super) ramp_samples: u32,   // Samples in a 50ms ramp at the current sample rate
}

impl<S> DynamicAmplify<S>
where
    S: Source<Item = f32>,
{
    pub fn new(source: S, gain_atomic: Arc<AtomicU32>, initial_gain: f32) -> Self {
        let sample_rate = source.sample_rate().get();
        let channels = source.channels().get() as u32;
        // 50ms ramp in total samples (all channels)
        let ramp_samples = (sample_rate * channels * 50) / 1000;

        // Un gain deja connu (mesure en cache ou pre-analyse hors-ligne) doit
        // s'appliquer des le PREMIER echantillon. Passer par la rampe ferait
        // demarrer chaque morceau 50 ms au mauvais niveau.
        let known = f32::from_bits(gain_atomic.load(Ordering::Relaxed));
        let start_gain = if known > 0.0 { known } else { initial_gain };

        Self {
            inner: source,
            gain_atomic,
            current_gain: start_gain,
            target_gain: start_gain,
            ramp_step: 0.0,
            ramp_remaining: 0,
            ramp_samples,
        }
    }

    /// Check for a new gain value and start a ramp if it changed.
    #[inline]
    pub(super) fn poll_gain(&mut self) {
        let bits = self.gain_atomic.load(Ordering::Relaxed);
        let new_gain = f32::from_bits(bits);

        // 0.0 means "not yet computed" — stay at current gain
        if new_gain == 0.0 {
            return;
        }

        // Only start a ramp if the target actually changed
        if (new_gain - self.target_gain).abs() > f32::EPSILON {
            self.target_gain = new_gain;
            if self.ramp_samples > 0 {
                self.ramp_step = (self.target_gain - self.current_gain) / self.ramp_samples as f32;
                self.ramp_remaining = self.ramp_samples;
            } else {
                self.current_gain = self.target_gain;
                self.ramp_remaining = 0;
            }
        }
    }
}
