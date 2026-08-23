//! Offline MODE engine (frontend-agnostic, ADR-006).
//!
//! The app operating without Qobuz — NOT the offline cache. Spec:
//! `qbz-nix-docs/offline-mode/2026-06-09-offline-mode-slint-port-spec.md`.
//!
//! Three states (D1):
//! - `Online` — connectivity up, Qobuz services available.
//! - `RealOffline` — detected connectivity loss, or a session started via
//!   "Start offline" with no Qobuz auth. SESSION-SCOPED: never persisted.
//! - `InducedOffline` — the user's persisted opt-in from Settings.
//!
//! Invariants:
//! - Offline (either flavor) ⇒ ZERO Qobuz services (D3). The engine owns the
//!   process-wide `qbz_qobuz::offline_gate`, flipping it on every transition.
//! - Induced wins over real for display; the raw connectivity rides along in
//!   the status so the UI can render the recovery banner logic (D2).
//! - Exiting induced offline is ALWAYS allowed (no probe gate — Tauri's
//!   trap is not ported); the state simply re-evaluates afterwards.
//! - Entering induced offline snapshots `audio_settings.stream_first_track`
//!   and forces it false; exiting restores it (issue #279 parity).

pub mod connectivity;
mod engine;
pub mod store;
#[cfg(test)]
mod tests;
mod types;

pub use connectivity::{Connectivity, ConnectivityActor, ConnectivitySnapshot};
pub use engine::OfflineModeEngine;
pub use store::{OfflineModeSettings, OfflineModeStore, QueuedScrobble};
pub use types::{OfflineMode, OfflineStatus};
