//! Diagnostics panel controller (Settings > Developer).
//!
//! Wires the `DiagnosticsState` Slint global. On `refresh()` it reads the three
//! settings stores (audio/graphics/developer) + computes the graphics runtime,
//! builds the frontend-agnostic `RuntimeDiagnostics` + `SystemInfo` snapshots
//! (`qbz_app::diagnostics`), snapshots the core player for the Playback rows, and
//! then pushes all per-section `[DiagRow]` models in one event-loop hop.
//! Export serializes the cached snapshot
//! (camelCase, matching the Tauri DiagnosticsPanel export) to the clipboard.
//!
//! 1:1 port of `src/lib/components/DiagnosticsPanel.svelte` (the row builders),
//! over the shared backend extracted to `qbz_app::diagnostics`.

mod collect;
mod controller;
mod export_json;
mod output_sinks;
mod redact;
mod report;
mod rows;
#[cfg(test)]
mod tests;

pub use report::build_full_report;

use std::sync::{Arc, Mutex};

use serde_json::Value;
use slint::ComponentHandle;

use crate::adapter::SlintAdapter;
use crate::{AppWindow, DiagnosticsState};

pub(crate) type Runtime = Arc<qbz_app::shell::AppRuntime<SlintAdapter>>;

/// The Diagnostics controller. Cloned into each `DiagnosticsState` callback.
#[derive(Clone)]
struct DiagController {
    pub(super) runtime: Runtime,
    pub(super) weak: slint::Weak<AppWindow>,
    pub(super) handle: tokio::runtime::Handle,
    /// Cached export base built on each `refresh()` — a JSON object with the
    /// runtime-diagnostics fields flattened + `systemInfo` + `playback` +
    /// `exportedAt` is merged in at export time.
    pub(super) export: Arc<Mutex<Option<Value>>>,
}

/// Wire every `DiagnosticsState` callback. Call once at shell setup.
pub fn install(window: &AppWindow, runtime: Runtime, handle: tokio::runtime::Handle) {
    let ctrl = DiagController {
        runtime,
        weak: window.as_weak(),
        handle,
        export: Arc::new(Mutex::new(None)),
    };

    let state = window.global::<DiagnosticsState>();
    {
        let c = ctrl.clone();
        state.on_refresh(move || c.refresh());
    }
    {
        let c = ctrl.clone();
        state.on_export_clipboard(move || c.export_clipboard());
    }
}
