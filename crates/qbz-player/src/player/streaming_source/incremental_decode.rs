//! `IncrementalStreamingSource::decode_more` and its `Source`/`Iterator`
//! impls. Split out from `incremental.rs` purely for line budget — same
//! type, same `impl` semantics. `decode_more` stays one function: its
//! WouldBlock/stall-tracking logic is a cohesive state machine tied to
//! issue #591 that resists further splitting without hurting readability.

use std::time::Duration;

use rodio::Source;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::errors::Error as SymphoniaError;

use super::incremental::IncrementalStreamingSource;

impl IncrementalStreamingSource {
    /// Decode more packets to fill the sample queue.
    ///
    /// This is called when the sample queue is running low.
    /// It will decode packets until the queue has at least `min_samples` or EOF is reached.
    pub(super) fn decode_more(&mut self, min_samples: usize) {
        if self.finished {
            return;
        }

        while self.sample_queue.len() < min_samples {
            let packet = match self.format.next_packet() {
                Ok(packet) => packet,
                Err(SymphoniaError::IoError(ref e))
                    if e.kind() == std::io::ErrorKind::WouldBlock =>
                {
                    // Not enough data buffered yet - wait briefly and retry
                    // This happens when playback catches up with download
                    if !self.stalled && self.packets_decoded > 0 {
                        // Mid-playback stall, not initial buffering (≥1 packet
                        // already decoded): put the prefetch throttle in panic
                        // mode so the live stream gets the pipe to itself (#591).
                        self.stalled = true;
                        qbz_audio::network_throttle::state().record_underrun();
                    }
                    std::thread::sleep(Duration::from_millis(5));
                    continue;
                }
                Err(SymphoniaError::IoError(_)) => {
                    // EOF or other IO error
                    log::info!(
                        "IncrementalStreamingSource: EOF reached after {} packets",
                        self.packets_decoded
                    );
                    self.finished = true;
                    return;
                }
                Err(err) => {
                    log::error!("Symphonia read error in stream: {}", err);
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

                    // Add samples to queue
                    self.sample_queue
                        .extend(sample_buf.samples().iter().copied());
                    self.packets_decoded += 1;
                    // Successful decode ends any stall episode; the next
                    // WouldBlock streak records a fresh underrun.
                    self.stalled = false;
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

impl Source for IncrementalStreamingSource {
    fn current_span_len(&self) -> Option<usize> {
        // We don't know frame boundaries in the queue
        None
    }

    fn channels(&self) -> std::num::NonZero<u16> {
        std::num::NonZero::new(self.channels).unwrap()
    }

    fn sample_rate(&self) -> std::num::NonZero<u32> {
        std::num::NonZero::new(self.sample_rate).unwrap()
    }

    fn total_duration(&self) -> Option<Duration> {
        // We don't know total duration until download completes
        // Could estimate from content-length if available
        None
    }
}

impl Iterator for IncrementalStreamingSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        // If queue is running low, decode more
        // Keep at least 0.5 seconds of audio buffered
        let min_buffer = (self.sample_rate as usize * self.channels as usize) / 2;
        if self.sample_queue.len() < min_buffer {
            self.decode_more(min_buffer);
        }

        self.sample_queue.pop_front()
    }
}
