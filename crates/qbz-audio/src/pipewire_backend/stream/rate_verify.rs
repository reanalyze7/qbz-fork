//! Post-stream `clock.force-rate` re-apply/verify/retry. Split out of
//! `rate_forcing.rs` for the line-count limit — a near-duplicate of the
//! pre-stream force there (de-duplication opportunity for a future pass).

use super::super::PipeWireBackend;
use crate::backend::BackendConfig;
use std::process::Command;

/// Tier 1 (issue #263): also gated by skip_sink_switch — in "Lock output
/// device" mode QBZ never writes the global clock force (pre-stream OR
/// re-apply), so the user's external routing/graph clock is left untouched.
pub(super) fn reapply_and_verify_rate(config: &BackendConfig, effective_rate: u32) {
    if config.skip_sink_switch || effective_rate == 44100 || effective_rate == 48000 {
        return;
    }

    let _ = Command::new("pw-metadata")
        .args([
            "-n",
            "settings",
            "0",
            "clock.force-rate",
            &effective_rate.to_string(),
        ])
        .output();
    log::info!(
        "[PipeWire Backend] Re-applied clock.force-rate={}Hz after stream creation",
        effective_rate
    );

    // Verify PipeWire actually applied the rate.
    // USB hubs/docks may need extra time for rate switching.
    std::thread::sleep(std::time::Duration::from_millis(100));
    let Some(actual_rate) = PipeWireBackend::get_pipewire_current_rate() else {
        return;
    };
    if actual_rate == effective_rate {
        log::info!("[PipeWire Backend] Rate verified: {}Hz", actual_rate);
        return;
    }

    log::warn!(
        "[PipeWire Backend] Rate mismatch: requested {}Hz but PipeWire reports {}Hz. \
         Retrying with longer delay...",
        effective_rate,
        actual_rate
    );
    // Give slower USB devices more time, then force again
    std::thread::sleep(std::time::Duration::from_millis(500));
    let _ = Command::new("pw-metadata")
        .args([
            "-n",
            "settings",
            "0",
            "clock.force-rate",
            &effective_rate.to_string(),
        ])
        .output();
    std::thread::sleep(std::time::Duration::from_millis(200));

    if let Some(retry_rate) = PipeWireBackend::get_pipewire_current_rate() {
        if retry_rate == effective_rate {
            log::info!(
                "[PipeWire Backend] Rate verified after retry: {}Hz",
                retry_rate
            );
        } else {
            log::warn!(
                "[PipeWire Backend] Rate still {}Hz after retry (expected {}Hz). \
                 Audio may play at wrong speed.",
                retry_rate,
                effective_rate
            );
        }
    }
}
