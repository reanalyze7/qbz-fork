//! Output-stream creation orchestration.
//!
//! Order matters and must not be reshuffled: sink routing runs first (it may
//! change the system default sink), then rate negotiation (queries the now-
//! effective sink), then the pre-stream `clock.force-rate`, then CPAL device
//! selection and stream construction, then a post-stream re-apply/verify of
//! the forced rate (PipeWire can revert the graph rate while no stream is
//! open).

mod build;
mod device_select;
mod rate_forcing;
mod rate_verify;
mod sink_routing;

use crate::backend::{BackendConfig, BackendResult};
use rodio::MixerDeviceSink;

pub(super) fn create_output_stream(config: &BackendConfig) -> BackendResult<MixerDeviceSink> {
    let target_sink = sink_routing::switch_default_sink_if_needed(config);
    let effective_sink = sink_routing::resolve_effective_sink(&target_sink);
    let effective_rate = rate_forcing::negotiate_rate(&effective_sink, config.sample_rate);

    rate_forcing::force_rate_pre_stream(config, effective_rate);

    // Note: clock.force-quantum is intentionally NOT set.
    // rodio 0.22's MixerDeviceSink has its own internal mixer thread that
    // cannot synchronize with PipeWire's forced quantum, causing massive
    // buffer underruns at sample rates >= 88.2kHz. clock.force-rate alone
    // is sufficient for bit-perfect sample rate switching.

    // Wait for PipeWire to apply the sample rate change.
    // USB hubs (e.g. Razer USB4 Dock) may need longer than direct DACs.
    std::thread::sleep(std::time::Duration::from_millis(500));

    let device = device_select::select_cpal_device()?;

    // Tier 2a (#263): in locked mode (skip_sink_switch) QBZ does NOT steal the
    // system default sink, so route THIS stream to the selected sink via the
    // pipewire-ALSA plugin's PIPEWIRE_NODE env — it targets that node WITHOUT
    // changing the system default. The guard restores the prior env value
    // after the PCM open (kept alive until the end of this function).
    #[cfg(target_os = "linux")]
    let _pw_node_guard = device_select::lock_pipewire_node(config, &target_sink);

    let mixer_sink = build::build_stream(device, config, effective_rate)?;

    // Re-apply clock.force-rate AFTER stream creation.
    // When resuming after PipeWire dropped the stream during pause,
    // the graph may have reverted to the DAC's default rate (e.g. 44100).
    // The pre-stream force-rate can be ignored if no streams were active.
    // Re-applying now that the stream exists forces PipeWire to reconfigure
    // the graph at the correct rate.
    rate_verify::reapply_and_verify_rate(config, effective_rate);

    Ok(mixer_sink)
}
