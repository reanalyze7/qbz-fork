//! The `## Audio` markdown section.

use qbz_app::diagnostics::RuntimeDiagnostics;

use super::super::rows::{opt, yn};
use super::md_line;

pub(super) fn write_section(
    out: &mut String,
    d: &RuntimeDiagnostics,
    active_output: Option<&str>,
    available_outputs: &[String],
    active_fmt: &Option<(String, String)>,
) {
    out.push_str("\n## Audio\n\n");
    let sample_rate = match d.audio_preferred_sample_rate {
        Some(hz) => format!("{hz} Hz"),
        None => "Auto".to_string(),
    };
    let available_str = if available_outputs.is_empty() {
        "—".to_string()
    } else {
        available_outputs.join(", ")
    };
    md_line(out, "Output Device (saved)", &opt(&d.audio_output_device));
    md_line(out, "Active Output (runtime)", active_output.unwrap_or("—"));
    if let Some((rate, fmt)) = active_fmt {
        if !rate.is_empty() {
            md_line(out, "Active Rate (runtime)", rate);
        }
        if !fmt.is_empty() {
            md_line(out, "Active Format (runtime)", fmt);
        }
    }
    md_line(out, "Available Outputs", &available_str);
    md_line(out, "Backend", &opt(&d.audio_backend_type));
    md_line(out, "Exclusive Mode", yn(d.audio_exclusive_mode));
    md_line(out, "DAC Passthrough", yn(d.audio_dac_passthrough));
    md_line(out, "Preferred Sample Rate", &sample_rate);
    md_line(out, "ALSA Plugin", &opt(&d.audio_alsa_plugin));
    md_line(out, "ALSA HW Volume", yn(d.audio_alsa_hardware_volume));
    md_line(out, "Normalization", yn(d.audio_normalization_enabled));
    md_line(
        out,
        "Normalization Target",
        &format!("{} LUFS", d.audio_normalization_target_lufs),
    );
    md_line(out, "Gapless", yn(d.audio_gapless_enabled));
    md_line(out, "PW Force Bitperfect", yn(d.audio_pw_force_bitperfect));
    md_line(
        out,
        "Stream Buffer",
        &format!("{}s", d.audio_stream_buffer_seconds),
    );
    md_line(out, "Streaming Only", yn(d.audio_streaming_only));
}
