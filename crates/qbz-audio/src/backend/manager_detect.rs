//! Linux-only runtime detection helpers for `BackendManager`, split out from
//! `manager.rs` to keep both files under the line-count limit.

use super::manager::BackendManager;

impl BackendManager {
    #[cfg(target_os = "linux")]
    pub(super) fn is_pipewire_available() -> bool {
        // Detect PipeWire via its runtime socket ($XDG_RUNTIME_DIR/pipewire-0),
        // which exists whenever PipeWire is running. Unlike `pactl`, this does
        // NOT require pulseaudio-utils to be installed — PipeWire-only systems
        // frequently lack it, which used to hide the PipeWire backend entirely
        // (issue #466). pw-cli / pactl remain as fallbacks for unusual setups
        // (e.g. a non-default socket name, or Flatpak where the socket path
        // differs but the pulse shim is bridged).
        if let Some(runtime_dir) = std::env::var_os("XDG_RUNTIME_DIR") {
            if std::path::Path::new(&runtime_dir).join("pipewire-0").exists() {
                return true;
            }
        }
        if std::process::Command::new("pw-cli")
            .args(["info", "0"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return true;
        }
        std::process::Command::new("pactl")
            .arg("info")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[cfg(target_os = "linux")]
    pub(super) fn is_pulse_available() -> bool {
        // Check if PulseAudio is running
        std::process::Command::new("pactl")
            .arg("info")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}
