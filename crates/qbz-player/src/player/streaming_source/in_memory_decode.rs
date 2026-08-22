//! `InMemorySource::decode_more` and its `Source`/`Iterator` impls. Split
//! out from `in_memory.rs` purely for line budget — same type, same
//! `impl` semantics. Near-duplicate shape of `incremental_decode.rs`'s
//! decode loop (a future dedup opportunity, not attempted here to keep
//! this a behavior-preserving split).

use std::time::Duration;

use rodio::Source;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::errors::Error as SymphoniaError;

use super::in_memory::InMemorySource;

impl InMemorySource {
    pub(super) fn decode_more(&mut self, min_samples: usize) {
        if self.finished {
            return;
        }

        while self.sample_queue.len() < min_samples {
            let packet = match self.format.next_packet() {
                Ok(packet) => packet,
                Err(SymphoniaError::IoError(_)) => {
                    self.finished = true;
                    return;
                }
                Err(err) => {
                    log::error!("Symphonia read error in in-memory source: {}", err);
                    self.finished = true;
                    return;
                }
            };

            if packet.track_id() != self.track_id {
                continue;
            }

            match self.decoder.decode(&packet) {
                Ok(audio_buf) => {
                    let spec = *audio_buf.spec();
                    let mut sample_buf = SampleBuffer::<f32>::new(audio_buf.frames() as u64, spec);
                    sample_buf.copy_interleaved_ref(audio_buf);
                    self.sample_queue
                        .extend(sample_buf.samples().iter().copied());
                }
                Err(SymphoniaError::DecodeError(e)) => {
                    log::warn!("Decode error (skipping packet): {}", e);
                    continue;
                }
                Err(SymphoniaError::ResetRequired) => {
                    self.decoder.reset();
                    continue;
                }
                Err(err) => {
                    log::error!("Symphonia decode error: {}", err);
                    self.finished = true;
                    return;
                }
            }
        }
    }
}

impl Source for InMemorySource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> std::num::NonZero<u16> {
        std::num::NonZero::new(self.channels).unwrap()
    }

    fn sample_rate(&self) -> std::num::NonZero<u32> {
        std::num::NonZero::new(self.sample_rate).unwrap()
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl Iterator for InMemorySource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let min_buffer = (self.sample_rate as usize * self.channels as usize) / 2;
        if self.sample_queue.len() < min_buffer {
            self.decode_more(min_buffer);
        }
        self.sample_queue.pop_front()
    }
}
