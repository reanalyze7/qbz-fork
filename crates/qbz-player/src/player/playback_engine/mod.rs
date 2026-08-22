//! Playback Engine Abstraction
//!
//! Unified interface for different playback backends:
//! - Rodio (PipeWire, Pulse, ALSA via CPAL) - uses rodio::Sink
//! - ALSA Direct (hw: devices) - bypasses rodio, writes directly to ALSA PCM
//!
//! ALSA Direct uses a single long-lived writer thread with a source queue
//! to enable gapless playback. When one source ends, the next is picked up
//! seamlessly without interrupting the PCM stream.
//!
//! This module is split by concern:
//! - `constructors`: `new_rodio` / `new_alsa_direct` / `new_jack` / `new_alsa_dop`
//! - `alsa_writer` / `jack_feeder` / `dop_writer`: the long-lived backend threads
//! - `dispatch::transport`: play/pause/stop/Drop
//! - `dispatch::append`: append/append_dop/crossfade_to
//! - `dispatch::query`: empty/position/duration/is_*/supports_crossfade

mod alsa_writer;
mod constructors;
mod dispatch;
mod dop_writer;
mod jack_feeder;
mod source_queue;

use qbz_audio::AlsaDirectStream;
use rodio::{mixer::Mixer, Player as RodioPlayer};
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;
use std::thread;

pub(crate) use source_queue::SourceQueue;

/// A boxed sample iterator that can be sent across threads
pub(crate) type BoxedSampleIter = Box<dyn Iterator<Item = f32> + Send>;

/// A boxed DoP word iterator (pre-packed S32 DoP samples — see qbz-dsd)
#[cfg(target_os = "linux")]
pub(crate) type BoxedDopIter = Box<dyn Iterator<Item = i32> + Send>;

/// Unified playback engine
pub enum PlaybackEngine {
    /// Rodio-based (PipeWire, Pulse, ALSA via CPAL)
    Rodio {
        sink: RodioPlayer,
        // Kept so `crossfade_to` can connect a SECOND overlapping player to
        // the same output (true crossfade needs two sources mixed at once,
        // not `append`'s sequential queue). Not used by any other engine
        // variant — bit-perfect paths (ALSA Direct/JACK/DoP) stay strictly
        // gapless (owner decision, 2026-08-21).
        mixer: Mixer,
    },
    /// Direct ALSA (hw: devices, bit-perfect) with gapless source queue
    AlsaDirect {
        stream: Arc<AlsaDirectStream>,
        is_playing: Arc<AtomicBool>,
        should_stop: Arc<AtomicBool>,
        position_frames: Arc<AtomicU64>,
        duration_frames: Arc<AtomicU64>,
        source_queue: Arc<SourceQueue<BoxedSampleIter>>,
        playback_thread: Option<thread::JoinHandle<()>>,
        /// Signals that the writer thread has consumed a source and moved to next
        source_transition: Arc<AtomicBool>,
        hardware_volume: bool,
    },
    /// Native JACK output (#263 Tier 3). Mirrors AlsaDirect (gapless source queue
    /// + a single long-lived feeder thread), but the feeder resamples each source
    /// to the JACK graph rate and writes interleaved stereo f32 into the client's
    /// lock-free ring buffer via `JackStream::write_f32`. NOT bit-perfect.
    #[cfg(target_os = "linux")]
    Jack {
        is_playing: Arc<AtomicBool>,
        should_stop: Arc<AtomicBool>,
        position_frames: Arc<AtomicU64>,
        duration_frames: Arc<AtomicU64>,
        source_queue: Arc<SourceQueue<BoxedSampleIter>>,
        feeder_thread: Option<thread::JoinHandle<()>>,
        source_transition: Arc<AtomicBool>,
        graph_rate: u32,
    },
    /// DoP (DSD over PCM) direct output (DSD plan Phase 2). Mirrors
    /// AlsaDirect's writer-thread + source-queue shape but carries
    /// pre-packed S32 DoP words written VERBATIM (no f32, no gain — one
    /// altered sample breaks the DoP markers and the DAC plays the raw
    /// bitstream as loud noise). Pause feeds 0x69 DSD silence so the DAC
    /// stays locked in DSD mode; the queue gives gapless DSD.
    #[cfg(target_os = "linux")]
    AlsaDop {
        stream: Arc<AlsaDirectStream>,
        is_playing: Arc<AtomicBool>,
        should_stop: Arc<AtomicBool>,
        position_frames: Arc<AtomicU64>,
        source_queue: Arc<SourceQueue<BoxedDopIter>>,
        writer_thread: Option<thread::JoinHandle<()>>,
        source_transition: Arc<AtomicBool>,
    },
}
