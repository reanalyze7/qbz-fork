use qbz_app::diagnostics::RuntimeDiagnostics;

use super::helpers::{opt, row, yn};

pub(super) fn build_audio_rows(
    d: &RuntimeDiagnostics,
    active_output: Option<&str>,
    available_outputs: &[String],
    active_rate: Option<&str>,
    active_fmt: Option<&str>,
) -> Vec<crate::DiagRow> {
    let sample_rate = match d.audio_preferred_sample_rate {
        Some(hz) => format!("{hz} Hz"),
        None => "Auto".to_string(),
    };

    // Output Device: saved id (may be a stale/unplugged DAC) vs the live active
    // output. Match (1) when the active equals OR is contained/suffixed by the
    // saved value; mismatch (2) when an active device exists but differs (so the
    // stale-saved-vs-live discrepancy is visible); info (0) when no live device.
    let saved_output = opt(&d.audio_output_device);
    let output_runtime = active_output.unwrap_or("—");
    let output_status = match active_output {
        Some(active) => {
            if saved_output == active
                || saved_output.contains(active)
                || saved_output.ends_with(active)
            {
                1
            } else {
                2
            }
        }
        None => 0,
    };
    let available_runtime = if available_outputs.is_empty() {
        "—".to_string()
    } else {
        available_outputs.join(", ")
    };

    vec![
        row("Output Device", &saved_output, output_runtime, output_status),
        row("Backend", &opt(&d.audio_backend_type), "—", 0),
        row("Exclusive Mode", yn(d.audio_exclusive_mode), "—", 0),
        row("DAC Passthrough", yn(d.audio_dac_passthrough), "—", 0),
        row("Preferred Sample Rate", &sample_rate, active_rate.unwrap_or("—"), 0),
        row("Active Format", "—", active_fmt.unwrap_or("—"), 0),
        row("ALSA Plugin", &opt(&d.audio_alsa_plugin), "—", 0),
        row("ALSA HW Volume", yn(d.audio_alsa_hardware_volume), "—", 0),
        row("Normalization", yn(d.audio_normalization_enabled), "—", 0),
        row(
            "Normalization Target",
            &format!("{} LUFS", d.audio_normalization_target_lufs),
            "—",
            0,
        ),
        row("Gapless", yn(d.audio_gapless_enabled), "—", 0),
        row("PW Force Bitperfect", yn(d.audio_pw_force_bitperfect), "—", 0),
        row(
            "Stream Buffer",
            &format!("{}s", d.audio_stream_buffer_seconds),
            "—",
            0,
        ),
        row("Streaming Only", yn(d.audio_streaming_only), "—", 0),
        row("Available Outputs", "—", &available_runtime, 0),
    ]
}
