//! Single long-lived writer thread for DoP (DSD over PCM).
//!
//! Mirrors `alsa_writer_thread`'s shape: pulls pre-packed S32 DoP words from
//! the current source, writes them VERBATIM, and picks up the next queued
//! source seamlessly (gapless DSD). Differences forced by the format:
//! - pause writes 0x69 DSD silence (with valid alternating markers) instead
//!   of going quiet, so the DAC stays locked in DSD mode;
//! - stop / end-of-queue pads ~150 ms of DSD silence before the stream
//!   closes (DACs pop when a DSD stream stops mid-pattern).

#[cfg(target_os = "linux")]
use super::{BoxedDopIter, SourceQueue};
#[cfg(target_os = "linux")]
use qbz_audio::AlsaDirectStream;
#[cfg(target_os = "linux")]
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
#[cfg(target_os = "linux")]
use std::sync::Arc;
#[cfg(target_os = "linux")]
use std::time::Duration;

#[cfg(target_os = "linux")]
#[allow(clippy::too_many_arguments)]
pub(super) fn dop_writer_thread(
    stream: Arc<AlsaDirectStream>,
    is_playing: Arc<AtomicBool>,
    should_stop: Arc<AtomicBool>,
    position_frames: Arc<AtomicU64>,
    source_queue: Arc<SourceQueue<BoxedDopIter>>,
    source_transition: Arc<AtomicBool>,
    channels: u16,
    native: bool,
) {
    const CHUNK_FRAMES: usize = 4096;
    let chunk_words = CHUNK_FRAMES * channels as usize;
    let carrier = stream.sample_rate() as usize;
    let mut silence_packer = qbz_dsd::DopPacker::new();
    let mut silence_buf: Vec<i32> = Vec::new();
    let mut buf: Vec<i32> = Vec::with_capacity(chunk_words);
    let mut current: Option<BoxedDopIter> = None;
    let mut had_source = false;

    let write_silence = |packer: &mut qbz_dsd::DopPacker,
                             silence_buf: &mut Vec<i32>,
                             frames: usize| {
        silence_buf.clear();
        if native {
            // Native DSD silence: 0x69 in every byte lane, no DoP markers.
            silence_buf.resize(frames * channels as usize, qbz_dsd::NATIVE_DSD_SILENCE_U32);
        } else {
            packer.silence(frames, channels, silence_buf);
        }
        if let Err(e) = stream.write_dop_i32(silence_buf) {
            log::warn!("[DoP Engine] Silence write failed: {}", e);
        }
    };

    log::info!("[DoP Engine] Writer thread started (gapless-capable)");
    'thread: loop {
        if should_stop.load(Ordering::SeqCst) {
            write_silence(&mut silence_packer, &mut silence_buf, carrier * 150 / 1000);
            log::info!("[DoP Engine] Stop signal, writer thread exiting");
            break 'thread;
        }

        if current.is_none() {
            match source_queue.wait_for_source(Duration::from_millis(100)) {
                Some(src) => {
                    current = Some(src);
                    if had_source {
                        source_transition.store(true, Ordering::SeqCst);
                    }
                    had_source = true;
                    position_frames.store(0, Ordering::SeqCst);
                    log::info!("[DoP Engine] Acquired new DoP source");
                }
                None => continue 'thread,
            }
        }

        if !is_playing.load(Ordering::SeqCst) {
            // Paused: keep the DAC locked in DSD with real DSD silence. The
            // blocking PCM write self-paces this loop (~100 ms per chunk).
            write_silence(&mut silence_packer, &mut silence_buf, carrier / 10);
            continue 'thread;
        }

        buf.clear();
        let source = current.as_mut().unwrap();
        let mut source_ended = false;
        for _ in 0..chunk_words {
            match source.next() {
                Some(w) => buf.push(w),
                None => {
                    source_ended = true;
                    break;
                }
            }
        }

        if !buf.is_empty() {
            if let Err(e) = stream.write_dop_i32(&buf) {
                // Match PCM ALSA: a hard write failure must stop the writer.
                // Continuing would desync DoP markers (harsh noise) and leave
                // exclusive mode stuck while position still advances.
                log::error!("[DoP Engine] Write failed: {e} — stopping writer");
                is_playing.store(false, Ordering::SeqCst);
                should_stop.store(true, Ordering::SeqCst);
                break 'thread;
            }
            position_frames.fetch_add((buf.len() / channels as usize) as u64, Ordering::SeqCst);
        }

        if source_ended {
            current = None;
            if source_queue.is_empty() {
                write_silence(&mut silence_packer, &mut silence_buf, carrier * 150 / 1000);
                is_playing.store(false, Ordering::SeqCst);
                log::info!("[DoP Engine] Source ended, no next source (padded DSD silence)");
            }
            // else: the queued next source is picked up on the next iteration
            // with the PCM still running — the gapless DSD transition.
        }
    }
}
