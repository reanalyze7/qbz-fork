//! Frontend-agnostic FFT/visualizer producer.
//!
//! All DSP — Hann window, FFT, log-bars, energy bands, transient detection,
//! waveform downsample, and the spectral ribbon — lives here so any frontend
//! (Tauri, Slint, headless) can consume the same five typed streams without a
//! framework dependency. A [`VizSink`] is the only seam: the producer thread
//! computes a [`VizFrame`] and hands it to `sink.submit(...)`.
//!
//! This is strictly downstream of the lockless [`RingBuffer`](super::RingBuffer)
//! (the read-only tap on the bit-perfect stream); it touches none of the
//! protected audio device/stream path. See
//! `qbz-nix-docs/immersive-slint-handoff/recon/source/audio-transport.md`.

mod bars;
mod energy;
mod log_bars;
mod run_loop;
#[cfg(test)]
mod tests;
mod waveform;

use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use super::VisualizerTap;

/// Number of energy bands for the Energy Bands visualizer
const NUM_ENERGY_BANDS: usize = 5;
const NUM_SPECTRAL_BANDS: usize = 512;
const SPECTRAL_UPDATE_RATE_HZ: u32 = 58;
const SPECTRAL_SMOOTHING: f32 = 0.30;

/// Energy band frequency ranges (Hz):
/// Sub-bass (20-60), Bass (60-250), Mids (250-2k), Presence (2k-6k), Air (6k-20k)
const ENERGY_BAND_RANGES: [(f32, f32); NUM_ENERGY_BANDS] = [
    (20.0, 60.0),
    (60.0, 250.0),
    (250.0, 2000.0),
    (2000.0, 6000.0),
    (6000.0, 20000.0),
];

/// One frame of computed visualization data. Each variant corresponds to one of
/// the five streams the Tauri build historically emitted as `viz:*` events; the
/// payload is the decoded magnitudes, not the LE-f32 byte blob (the Tauri
/// adapter re-serializes those for backward compatibility).
#[derive(Clone, Debug)]
pub enum VizFrame {
    /// 16 log-spaced FFT bars (`viz:data`).
    Viz16([f32; 16]),
    /// 256 L samples followed by 256 R samples (`viz:waveform`).
    Wave256x2(Box<[f32; 512]>),
    /// `NUM_SPECTRAL_BANDS` spectral bands (`viz:spectral`).
    Spectral512(Vec<f32>),
    /// 5 energy bands (`viz:energy`).
    Energy5([f32; 5]),
    /// A single transient intensity, submitted only on detection (`viz:transient`).
    Transient1(f32),
}

/// Frontend-agnostic consumer of visualization frames. Implemented by the Tauri
/// adapter (re-emits the `viz:*` events) and the Slint adapter (latches frames
/// for the UI-thread drain).
pub trait VizSink: Send + Sync {
    fn submit(&self, frame: VizFrame);
}

/// Idle poll for the disabled AND paused states. Instead of spinning at
/// `TARGET_FPS` while the tap is off (or the player is paused and the ring
/// buffer is stale), the producer parks for this long between re-checks of the
/// `enabled`/`paused` atomics. An enable/resume path MAY `unpark()` the thread
/// (via the returned `JoinHandle`'s `.thread()`) for an instant wake — the
/// Slint frontend does; callers that don't still get picked up within this
/// bound (≤200ms, well under the ~250ms resume-latency budget).
const IDLE_POLL: Duration = Duration::from_millis(200);

/// Spawn the FFT processing thread. Idempotency is the caller's concern (the
/// `Visualizer`/shell guards against a double start). Returns the join handle;
/// callers that run for the app lifetime can drop it (or keep `.thread()` to
/// `unpark()` on enable, see [`IDLE_POLL`]).
pub fn spawn_visualizer_thread(tap: VisualizerTap, sink: Arc<dyn VizSink>) -> JoinHandle<()> {
    std::thread::Builder::new()
        .name("visualizer-fft".to_string())
        .spawn(move || {
            run_loop::run_fft_loop(tap, sink);
        })
        .expect("Failed to spawn visualizer thread")
}
