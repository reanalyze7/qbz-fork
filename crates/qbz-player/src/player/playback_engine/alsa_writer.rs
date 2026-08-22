//! Single long-lived writer thread for ALSA Direct.
//!
//! Continuously reads samples from the current source and writes to ALSA.
//! When a source ends, seamlessly picks up the next one from the queue
//! (gapless transition). If no next source is available, drains the ALSA
//! buffer and waits for the next source or a stop signal.

use super::{BoxedSampleIter, SourceQueue};
use qbz_audio::AlsaDirectStream;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub(super) fn alsa_writer_thread(
    stream: Arc<AlsaDirectStream>,
    is_playing: Arc<AtomicBool>,
    should_stop: Arc<AtomicBool>,
    position_frames: Arc<AtomicU64>,
    duration_frames: Arc<AtomicU64>,
    source_queue: Arc<SourceQueue<BoxedSampleIter>>,
    source_transition: Arc<AtomicBool>,
    channels: u16,
) {
    const CHUNK_FRAMES: usize = 8192;
    let chunk_samples = CHUNK_FRAMES * channels as usize;
    let mut buffer_f32 = Vec::with_capacity(chunk_samples);
    let mut current_source: Option<BoxedSampleIter> = None;
    let mut total_frames: u64 = 0;

    log::info!("[ALSA Direct Engine] Writer thread started (gapless-capable)");

    'thread: loop {
        // Check global stop
        if should_stop.load(Ordering::SeqCst) {
            log::info!("[ALSA Direct Engine] Stop signal, writer thread exiting");
            break 'thread;
        }

        // If no current source, try to get one
        if current_source.is_none() {
            // Wait for a source (with 100ms timeout to recheck stop flag)
            match source_queue.wait_for_source(Duration::from_millis(100)) {
                Some(src) => {
                    current_source = Some(src);
                    total_frames = 0;
                    position_frames.store(0, Ordering::SeqCst);
                    log::info!("[ALSA Direct Engine] Acquired new source from queue");
                }
                None => {
                    // No source available, loop back to check stop
                    continue 'thread;
                }
            }
        }

        // Wait while paused
        while !is_playing.load(Ordering::SeqCst) {
            if should_stop.load(Ordering::SeqCst) {
                break 'thread;
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        // Fill buffer from current source
        buffer_f32.clear();
        let source = current_source.as_mut().unwrap();
        let mut source_ended = false;

        for _ in 0..chunk_samples {
            match source.next() {
                Some(sample) => buffer_f32.push(sample),
                None => {
                    source_ended = true;
                    break;
                }
            }
        }

        // Write whatever we have to ALSA (even partial chunks on source end)
        if !buffer_f32.is_empty() {
            if let Err(e) = stream.write_f32(&buffer_f32) {
                log::error!("[ALSA Direct Engine] Write failed: {}", e);
                break 'thread;
            }

            let frames_written = buffer_f32.len() / channels as usize;
            total_frames += frames_written as u64;
            position_frames.store(total_frames, Ordering::SeqCst);
            duration_frames.store(total_frames, Ordering::SeqCst);
        }

        if source_ended {
            log::info!(
                "[ALSA Direct Engine] Source ended (total frames: {})",
                total_frames
            );

            // Try to get next source immediately (gapless transition)
            match source_queue.try_pop() {
                Some(next_src) => {
                    log::info!("[ALSA Direct Engine] Gapless transition to next source");
                    current_source = Some(next_src);
                    total_frames = 0;
                    position_frames.store(0, Ordering::SeqCst);
                    // Signal that a transition happened
                    source_transition.store(true, Ordering::SeqCst);
                    // Continue immediately — no drain, no gap
                }
                None => {
                    // No next source — this is a natural end of playback
                    log::info!("[ALSA Direct Engine] No next source, draining ALSA buffer");
                    if let Err(e) = stream.drain() {
                        log::warn!("[ALSA Direct Engine] Drain failed: {}", e);
                    }
                    current_source = None;
                    is_playing.store(false, Ordering::SeqCst);
                    // Don't break — stay alive waiting for next append()
                }
            }
        }
    }

    is_playing.store(false, Ordering::SeqCst);
    log::info!("[ALSA Direct Engine] Writer thread finished");
}
