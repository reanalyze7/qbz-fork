//! Detection refresh (spawn_blocking probe) + default-output resolution.

use super::state::{tier_for_max_rate_hz, CapState, CAP};

/// Re-run detection and refresh the cache. `limit_enabled` off clears it
/// immediately (no probe). The probe runs in `spawn_blocking` (pw-dump
/// subprocess + /proc reads — never on the UI thread). Await-able so the
/// Settings controller can re-push the summary row right after.
pub async fn refresh(limit_enabled: bool, output_device: Option<String>) {
    if !limit_enabled {
        *CAP.write().unwrap_or_else(|e| e.into_inner()) = None;
        log::info!("[qbz-slint] device cap: disabled");
        return;
    }
    let probed = tokio::task::spawn_blocking(move || {
        // The configured device id, or the system-default sink when the
        // selection is "System default" (None). An unresolvable default
        // probes with an empty node name, which lands on the fallback set →
        // detected=false → Hi-Res+ no-op cap with the caveat disclosed.
        let node = output_device.unwrap_or_else(default_output_node);
        qbz_audio::query_dac_capabilities(&node)
    })
    .await;
    let caps = match probed {
        Ok(c) => c,
        Err(e) => {
            log::warn!("[qbz-slint] device cap: probe task failed: {e}");
            return;
        }
    };
    let max_rate_hz = caps.sample_rates.iter().copied().max().unwrap_or(0);
    // assemble() always yields a non-empty rate list (fallback set), but
    // never store a 0 Hz cap if that invariant ever breaks.
    if max_rate_hz == 0 {
        *CAP.write().unwrap_or_else(|e| e.into_inner()) = None;
        return;
    }
    let state = CapState {
        tier: tier_for_max_rate_hz(max_rate_hz),
        detected: caps.detected,
        max_rate_hz,
        description: caps.description.unwrap_or_else(|| caps.node_name.clone()),
    };
    log::info!(
        "[qbz-slint] device cap: {} -> max {} Hz -> {:?} ({})",
        state.description,
        state.max_rate_hz,
        state.tier,
        if state.detected {
            "detected"
        } else {
            "fallback set"
        },
    );
    *CAP.write().unwrap_or_else(|e| e.into_inner()) = Some(state);
}

/// The system-default PipeWire sink's node id, for the "System default"
/// device selection. Empty when nothing enumerates (no PipeWire) — the probe
/// then reports the fallback set honestly.
fn default_output_node() -> String {
    qbz_audio::backend::BackendManager::create_backend(
        qbz_audio::backend::AudioBackendType::PipeWire,
    )
    .ok()
    .and_then(|b| b.enumerate_devices().ok())
    .and_then(|devs| devs.into_iter().find(|d| d.is_default))
    .map(|d| d.id)
    .unwrap_or_default()
}
