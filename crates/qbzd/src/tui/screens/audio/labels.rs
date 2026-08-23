use qbz_audio::{AlsaPlugin, AudioBackendType};

use crate::tui::strings as s;

// ============================ value/label mappers ============================

pub fn backend_label(b: AudioBackendType) -> String {
    match b {
        AudioBackendType::PipeWire => "PipeWire".to_string(),
        AudioBackendType::Alsa => "ALSA".to_string(),
        AudioBackendType::Pulse => "PulseAudio".to_string(),
        AudioBackendType::SystemDefault => "System default".to_string(),
        AudioBackendType::Jack => "JACK".to_string(),
    }
}

/// The `settings set audio.backend` value token (matches write_one's parse_backend).
pub(super) fn backend_value(b: AudioBackendType) -> &'static str {
    match b {
        AudioBackendType::SystemDefault => "system",
        AudioBackendType::PipeWire => "pipewire",
        AudioBackendType::Alsa => "alsa",
        AudioBackendType::Pulse => "pulse",
        AudioBackendType::Jack => "jack",
    }
}

pub(super) fn alsa_plugin_label(p: AlsaPlugin) -> &'static str {
    match p {
        AlsaPlugin::Hw => s::ALSA_HW,
        AlsaPlugin::PlugHw => s::ALSA_PLUGHW,
        AlsaPlugin::Pcm => s::ALSA_PCM,
    }
}
pub(super) fn alsa_plugin_value(p: AlsaPlugin) -> &'static str {
    match p {
        AlsaPlugin::Hw => "hw",
        AlsaPlugin::PlugHw => "plughw",
        AlsaPlugin::Pcm => "pcm",
    }
}

pub(super) fn dsd_label(mode: &str) -> &'static str {
    match mode {
        "dop" => s::DSD_DOP,
        "native" => s::DSD_NATIVE,
        _ => s::DSD_CONVERT,
    }
}

/// Compact a long device id for menu/summary lines (char-safe).
pub(super) fn short_device(id: &str) -> String {
    let count = id.chars().count();
    if count <= 24 {
        id.to_string()
    } else {
        let tail: String = id.chars().skip(count - 23).collect();
        format!("…{tail}")
    }
}
