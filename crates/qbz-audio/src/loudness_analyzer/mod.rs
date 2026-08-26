//! Background loudness analyzer thread.
//!
//! Long-lived thread that receives decoded audio samples from `AnalyzerTap`,
//! computes EBU R128 integrated LUFS, and updates a shared `Arc<AtomicU32>`
//! gain value that `DynamicAmplify` reads.
//!
//! Regle de volume, tenue par `AnalyzerState::gain_applied` :
//! - mesure connue (ecoute precedente ou pre-analyse hors-ligne) -> gain pose
//!   des la premiere note ;
//! - sinon, un unique gain provisoire vers 2 s, a partir de la loudness
//!   court-terme ;
//! - ensuite plus rien ne bouge pendant le morceau : les mesures integrees
//!   (10 s puis toutes les 5 s) ne nourrissent que le cache, pour la
//!   prochaine ecoute.

mod measure;
mod run_loop;
mod state;

use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::thread;

use super::analyzer_tap::AnalyzerMessage;
use super::loudness_cache::LoudnessCache;

use state::AnalyzerState;

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
