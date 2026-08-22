//! Sample-rate negotiation against the DAC's capabilities, and the
//! pre-stream `clock.force-rate` apply. The post-stream re-apply/verify lives
//! in `rate_verify.rs` — a near-duplicate `pw-metadata` call kept in its own
//! file (a de-duplication opportunity for a future pass).

use super::super::PipeWireBackend;
use crate::backend::BackendConfig;
use std::process::Command;
use std::sync::atomic::Ordering;

pub(super) fn negotiate_rate(effective_sink: &Option<String>, requested: u32) -> u32 {
    let Some(sink_name) = effective_sink else {
        return requested;
    };
    match PipeWireBackend::get_sink_supported_rates(sink_name) {
        Some(rates) if rates.contains(&requested) => {
            log::info!(
                "[PipeWire Backend] DAC supports {}Hz (available: {:?})",
                requested,
                rates
            );
            requested
        }
        Some(rates) => {
            let fallback = PipeWireBackend::find_best_fallback_rate(requested, &rates);
            log::warn!(
                "[PipeWire Backend] DAC doesn't support {}Hz. Supported: {:?}. Falling back to {}Hz (resampled by rodio)",
                requested, rates, fallback
            );
            fallback
        }
        None => {
            log::info!(
                "[PipeWire Backend] Could not determine DAC supported rates, using {}Hz",
                requested
            );
            requested
        }
    }
}

/// Force PipeWire to use the effective sample rate (for bit-perfect playback).
///
/// Tier 1 (issue #263): when skip_sink_switch ("Lock output device") is ON, the
/// user has asked QBZ not to mutate GLOBAL graph state to preserve external
/// routing (JACK/qjackctl/qpwgraph). So we skip the global `clock.force-rate`
/// write too — not just `set-default-sink`. This trades device-native-rate
/// playback for routing freedom (PipeWire resamples when track rate != graph
/// rate). Safe: skip_sink_switch is transitively mutually exclusive with
/// dac_passthrough / pw_force_bitperfect (the bit-perfect clock path), so this
/// never collides with a forced-rate bit-perfect session.
pub(super) fn force_rate_pre_stream(config: &BackendConfig, effective_rate: u32) {
    if config.skip_sink_switch {
        log::info!(
            "[PipeWire Backend] Skipping clock.force-rate (skip_sink_switch enabled) — \
             external routing preserved; PipeWire may resample if graph rate differs"
        );
        return;
    }

    log::info!(
        "[PipeWire Backend] Forcing sample rate to {}Hz via pw-metadata",
        effective_rate
    );
    let metadata_result = Command::new("pw-metadata")
        .args([
            "-n",
            "settings",
            "0",
            "clock.force-rate",
            &effective_rate.to_string(),
        ])
        .output();

    match metadata_result {
        Ok(output) if output.status.success() => {
            log::info!(
                "[PipeWire Backend] Sample rate forced to {}Hz",
                effective_rate
            );
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::warn!("[PipeWire Backend] Failed to force sample rate: {}", stderr);
        }
        Err(e) => {
            log::warn!("[PipeWire Backend] Error executing pw-metadata: {}", e);
        }
    }
    // Remember WE forced the clock so the stop/suspend reset undoes it
    // (and only it). This runs for every non-locked PipeWire stream, so
    // a plain no-passthrough user no longer leaks a forced rate on stop.
    super::super::CLOCK_FORCE_APPLIED.store(true, Ordering::Relaxed);
}
