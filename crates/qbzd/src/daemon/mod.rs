// crates/qbzd/src/daemon/ — the `qbzd run` boot sequence (01-architecture.md
// §8.1, NORMATIVE order), the NeedsAuth-stays-up state machine (§6.2) and the
// graceful shutdown (§8.2). Later tasks splice into the numbered steps: the
// playback driver (T4) at step 10, the HTTP server (T6) at step 11, QConnect
// (T9/T10) at step 12. Until they land the daemon boots a playable core and
// parks on signals — API-less but fully diagnosable in-process.
mod bind;
mod boot;
mod driver_deps;
mod queue_persist;
mod reload;
mod run;
mod session;
mod shutdown;
mod subsystems;
#[cfg(test)]
mod tests;

use std::sync::{Arc, Mutex};

use qbz_app::shell::AppRuntime;
use qbz_models::CoreEvent;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::adapter::DaemonAdapter;
use crate::state::DaemonShared;

pub use run::run;

// `reload` (the settings-reload entry point) is called from `api/settings.rs`,
// outside this module tree.
pub(crate) use reload::reload;

/// The composed runtime handoff produced by [`boot`] and consumed by later
/// tasks: T4 spawns the playback driver on `runtime` + `shared`, T6 serves
/// `bus` over HTTP/SSE, T9/T10 wire QConnect. Held alive by [`run`] through the
/// signal park so the core stays up.
#[allow(dead_code)] // fields are the seam later tasks (T6/T9/T10) read.
pub struct BootedRuntime {
    pub runtime: Arc<AppRuntime<DaemonAdapter>>,
    pub shared: Arc<Mutex<DaemonShared>>,
    pub bus: broadcast::Sender<CoreEvent>,
    /// Background session-restore retry (network-class boot failure only). Held
    /// so shutdown can abort+join it BEFORE releasing the audio device: it holds
    /// an `Arc<AppRuntime>` clone, so leaving it running would keep the Player
    /// alive past `drop(booted)` and break the #521 clock-release ordering (§8.2).
    pub auth_retry: Option<JoinHandle<()>>,
}
