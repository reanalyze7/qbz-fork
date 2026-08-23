// crates/qbzd/src/cli/settings/codec_bool.rs — value parse/render for the
// bool-flavored keys plus the enum-like backend/ALSA-plugin choices.

use qbz_audio::{AlsaPlugin, AudioBackendType};

pub(super) fn parse_bool(v: &str) -> Result<bool, String> {
    match v.to_ascii_lowercase().as_str() {
        "true" | "on" | "1" | "yes" => Ok(true),
        "false" | "off" | "0" | "no" => Ok(false),
        other => Err(format!("invalid value '{other}' — expected true or false")),
    }
}
pub(super) fn render_bool(v: bool) -> String {
    v.to_string()
}

pub(super) fn parse_backend(v: &str) -> Result<Option<AudioBackendType>, String> {
    match v.to_ascii_lowercase().as_str() {
        "system" | "systemdefault" | "system_default" => Ok(Some(AudioBackendType::SystemDefault)),
        "pipewire" | "pw" => Ok(Some(AudioBackendType::PipeWire)),
        "alsa" => Ok(Some(AudioBackendType::Alsa)),
        "pulse" | "pulseaudio" => Ok(Some(AudioBackendType::Pulse)),
        "jack" => Ok(Some(AudioBackendType::Jack)),
        other => Err(format!(
            "invalid backend '{other}' — expected one of: system, pipewire, alsa, pulse, jack"
        )),
    }
}
pub(super) fn render_backend(v: Option<AudioBackendType>) -> String {
    match v {
        Some(AudioBackendType::SystemDefault) => "system".to_string(),
        Some(AudioBackendType::PipeWire) => "pipewire".to_string(),
        Some(AudioBackendType::Alsa) => "alsa".to_string(),
        Some(AudioBackendType::Pulse) => "pulse".to_string(),
        Some(AudioBackendType::Jack) => "jack".to_string(),
        None => "auto".to_string(),
    }
}

pub(super) fn parse_alsa_plugin(v: &str) -> Result<Option<AlsaPlugin>, String> {
    match v.to_ascii_lowercase().as_str() {
        "hw" => Ok(Some(AlsaPlugin::Hw)),
        "plughw" => Ok(Some(AlsaPlugin::PlugHw)),
        "pcm" => Ok(Some(AlsaPlugin::Pcm)),
        // Not a documented TUI option (03-setup-tui.md §3.2.1 lists only the 3
        // concrete plugins, default Hw) — accepted here only so `settings
        // show`'s value on a never-migrated-to-a-plugin row (the seed INSERT
        // leaves this column NULL, unlike `backend_type`) round-trips back
        // into `set` unchanged instead of erroring on its own read value.
        "auto" => Ok(None),
        other => Err(format!(
            "invalid ALSA plugin '{other}' — expected one of: hw, plughw, pcm"
        )),
    }
}
pub(super) fn render_alsa_plugin(v: Option<AlsaPlugin>) -> String {
    match v {
        Some(AlsaPlugin::Hw) => "hw".to_string(),
        Some(AlsaPlugin::PlugHw) => "plughw".to_string(),
        Some(AlsaPlugin::Pcm) => "pcm".to_string(),
        None => "auto".to_string(),
    }
}
