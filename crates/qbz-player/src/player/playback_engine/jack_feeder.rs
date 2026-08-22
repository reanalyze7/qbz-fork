//! Single long-lived feeder thread for JACK (#263 Tier 3).
//!
//! Mirrors `alsa_writer_thread`, but writes graph-rate interleaved STEREO f32
//! into the JACK client's lock-free ring buffer via `JackStream::write_f32`
//! (the RT process callback drains it), pacing itself when the ring is full.
//! Sources are resampled to the graph rate + stereo at `append` time.

use super::{BoxedSampleIter, SourceQueue};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[cfg(target_os = "linux")]
use qbz_audio::JackStream;

#[cfg(target_os = "linux")]
pub(super) fn jack_feeder_thread(
    stream: Arc<JackStream>,
    is_playing: Arc<AtomicBool>,
    should_stop: Arc<AtomicBool>,
    position_frames: Arc<AtomicU64>,
    duration_frames: Arc<AtomicU64>,
    source_queue: Arc<SourceQueue<BoxedSampleIter>>,
    source_transition: Arc<AtomicBool>,
) {
    const CHUNK_FRAMES: usize = 4096;
    const CHANNELS: usize = 2;
    let chunk_samples = CHUNK_FRAMES * CHANNELS;
    let mut buffer_f32: Vec<f32> = Vec::with_capacity(chunk_samples);
    let mut current_source: Option<BoxedSampleIter> = None;
    let mut total_frames: u64 = 0;

    log::info!("[JACK Engine] Feeder thread started");

    'thread: loop {
        if should_stop.load(Ordering::SeqCst) {
            break 'thread;
        }
        if current_source.is_none() {
            match source_queue.wait_for_source(Duration::from_millis(100)) {
                Some(src) => {
                    current_source = Some(src);
                    total_frames = 0;
                    position_frames.store(0, Ordering::SeqCst);
                }
                None => continue 'thread,
            }
        }
        while !is_playing.load(Ordering::SeqCst) {
            if should_stop.load(Ordering::SeqCst) {
                break 'thread;
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        buffer_f32.clear();
        let source = current_source.as_mut().unwrap();
        let mut source_ended = false;
        for _ in 0..chunk_samples {
            match source.next() {
                Some(s) => buffer_f32.push(s),
                None => {
                    source_ended = true;
                    break;
                }
            }
        }

        // Write to the ring, paced: write_f32 returns frames accepted; a full
        // ring returns fewer (or 0) and we wait for the RT callback to drain.
        let mut off_samples = 0usize;
        while off_samples < buffer_f32.len() {
            if should_stop.load(Ordering::SeqCst) {
                break 'thread;
            }
            let frames = stream.write_f32(&buffer_f32[off_samples..]);
            if frames == 0 {
                std::thread::sleep(Duration::from_millis(2));
                continue;
            }
            off_samples += frames * CHANNELS;
            total_frames += frames as u64;
            position_frames.store(total_frames, Ordering::SeqCst);
            duration_frames.store(total_frames, Ordering::SeqCst);
        }

        if source_ended {
            match source_queue.try_pop() {
                Some(next_src) => {
                    current_source = Some(next_src);
                    total_frames = 0;
                    position_frames.store(0, Ordering::SeqCst);
                    source_transition.store(true, Ordering::SeqCst);
                }
                None => {
                    current_source = None;
                    is_playing.store(false, Ordering::SeqCst);
                }
            }
        }
    }

    is_playing.store(false, Ordering::SeqCst);
    log::info!("[JACK Engine] Feeder thread finished");
}
