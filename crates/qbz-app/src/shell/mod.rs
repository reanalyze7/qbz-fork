//! Framework-agnostic application runtime facade.
//!
//! [`AppRuntime`] is the composition root that a non-Tauri UI shell (Slint,
//! TUI, headless) builds on. It owns an `Arc<QbzCore<A>>`, the framework-
//! agnostic runtime state machine, and the per-user session, all without any
//! Tauri dependency.
//!
//! Scope (Slint POC readiness audit, sessions 21-22):
//!
//! - Task 1 — composition and accessors: [`AppRuntime::new`], [`AppRuntime::core`].
//! - Task 2 — minimal session activation: [`AppRuntime::activate`] and
//!   friends. This is deliberately minimal. It opens only the session store
//!   and performs the portable session scaffolding (user paths, directories,
//!   last-user marker, runtime state). It does NOT touch Tauri's
//!   `session_lifecycle`, does not initialize the `src-tauri`-side per-user
//!   stores (`library`, `reco`, `lyrics`, ...), and does not run the
//!   flat-to-user migration. A shell opens further stores per view, as the
//!   views that need them come online.
//!
//! The Tauri app does not consume this module; `CoreBridge` and
//! `session_lifecycle` keep their own paths. `AppRuntime` is purely additive.

mod construct;
mod guest_profile;
mod session;
#[cfg(test)]
mod tests;

use std::sync::Mutex;

use qbz_audio::VisualizerTap;
use qbz_core::{FrontendAdapter, QbzCore};
use std::sync::Arc;

use crate::runtime::RuntimeManager;
use crate::session_store::SessionStore;
use crate::user_data::UserDataPaths;

/// The per-user stores opened for the currently active session.
///
/// Minimal by design: it holds only the session store. A shell opens further
/// per-user stores as the views that need them come online, rather than
/// loading the full WebKit-era store set up front.
struct ActiveSession {
    user_id: u64,
    session_store: SessionStore,
}

/// Composition root for a non-Tauri UI shell.
///
/// Generic over the [`FrontendAdapter`] the shell supplies, so the same
/// facade serves a Slint adapter, a TUI adapter, or a headless one.
pub struct AppRuntime<A: FrontendAdapter + Send + Sync + 'static> {
    core: Arc<QbzCore<A>>,
    runtime: Arc<RuntimeManager>,
    user_paths: UserDataPaths,
    session: Mutex<Option<ActiveSession>>,
    /// The visualizer tap handed to the player, retained so the shell can start
    /// the FFT producer and toggle capture. `None` for shells that do not drive
    /// audio visualization (the default `new`/`with_audio_settings` path).
    visualizer_tap: Option<VisualizerTap>,
}
