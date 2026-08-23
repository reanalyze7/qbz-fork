//! Shared blocking-settings-read pipeline: both `controller::refresh_async`
//! (feeds the panel rows) and `report::build_full_report` (feeds the
//! markdown export) gather the exact same inputs.

use qbz_app::diagnostics::{RuntimeDiagnostics, SystemInfo};

use super::output_sinks::collect_output_sinks;

pub(super) type Collected = (
    RuntimeDiagnostics,
    SystemInfo,
    Option<String>,
    Vec<String>,
    Option<(String, String)>,
);

/// Read the three settings stores + /proc + /sys + the live CPAL/pactl
/// output sinks. BLOCKING — call inside `spawn_blocking`.
pub(super) fn gather_blocking() -> Collected {
    let audio = qbz_audio::settings::AudioSettingsStore::new()
        .and_then(|s| s.get_settings())
        .unwrap_or_default();
    let (graphics, gfx_failed) =
        match qbz_app::settings::graphics::GraphicsSettingsStore::new().and_then(|s| s.get_settings())
        {
            Ok(g) => (g, false),
            Err(_) => (Default::default(), true),
        };
    let developer = qbz_app::settings::developer::DeveloperSettingsStore::new()
        .and_then(|s| s.get_settings())
        .unwrap_or_default();
    let gfx = qbz_app::diagnostics::detect_graphics_runtime(&graphics, gfx_failed);
    let runtime_diag =
        qbz_app::diagnostics::runtime_diagnostics(&qbz_app::diagnostics::DiagnosticsInputs {
            audio: &audio,
            graphics: &graphics,
            developer: &developer,
            gfx,
            app_version: env!("CARGO_PKG_VERSION"),
        });
    let sys = qbz_app::diagnostics::system_info();
    // Live output sinks (BLOCKING CPAL enumeration — stays inside this
    // spawn_blocking, never on the async path).
    let (active_output, available_outputs, active_fmt) = collect_output_sinks();
    (runtime_diag, sys, active_output, available_outputs, active_fmt)
}
