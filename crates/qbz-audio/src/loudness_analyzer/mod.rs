//! Background loudness analyzer thread.
//!
//! Long-lived thread that receives decoded audio samples from `AnalyzerTap`,
//! computes EBU R128 integrated LUFS, and updates a shared `Arc<AtomicU32>`
//! gain value that `DynamicAmplify` reads.
//!
//! - First measurement after ~10s of audio (EBU R128 needs sufficient data)
//! - Refinement every ~5s thereafter (gain converges by ~30-60s)
//! - Cached results are used immediately on cache hit

mod gain_math;
mod measure;
mod run_loop;
mod state;

use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::thread;

use super::analyzer_tap::AnalyzerMessage;
use super::loudness_cache::LoudnessCache;

use gain_math::compute_gain_capped;
use state::AnalyzerState;

/// Maximum gain boost in dB (conservative clipping prevention)
const MAX_GAIN_DB: f32 = 6.0;

pub struct LoudnessAnalyzer;

impl LoudnessAnalyzer {
    /// Spawn the analyzer thread. Returns the join handle.
    ///
    /// The thread blocks on `rx.recv()` when idle — zero CPU usage between tracks.
    pub fn spawn(
        rx: Receiver<AnalyzerMessage>,
        cache: Arc<LoudnessCache>,
    ) -> thread::JoinHandle<()> {
        thread::Builder::new()
            .name("loudness-analyzer".into())
            .spawn(move || {
                log::info!("[LoudnessAnalyzer] Thread started");
                Self::run(rx, cache);
                log::info!("[LoudnessAnalyzer] Thread exiting");
            })
            .expect("Failed to spawn loudness analyzer thread")
    }
}
