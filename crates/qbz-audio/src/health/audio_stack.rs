use std::process::Command;

/// Result of the audio-stack probes. All best-effort; a failed probe reads as
/// `false` (the wizard then surfaces the matching remediation).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioStackHealth {
    /// WirePlumber session manager is running (`systemctl --user is-active`).
    pub wireplumber_active: bool,
    /// `pw-dump` is installed (native enumeration; `pipewire-bin`).
    pub has_pw_dump: bool,
    /// CPAL can see PipeWire through the ALSA bridge PCM (`aplay -L` lists
    /// `pipewire`). This needs `pipewire-alsa` and is required for PLAYBACK
    /// (the stream is opened via CPAL), not just enumeration.
    pub cpal_sees_pipewire: bool,
    /// `pactl` is available (Pulse compat path; `pulseaudio-utils`).
    pub has_pactl: bool,
    /// At least one audio sink is visible to `pw-dump`.
    pub any_devices: bool,
}

impl AudioStackHealth {
    /// Everything the wizard needs for bit-perfect playback is present.
    pub fn is_ready(&self) -> bool {
        self.wireplumber_active && self.cpal_sees_pipewire && self.any_devices
    }
}

/// True if `sh -c "<probe>"` exits 0.
fn sh_ok(probe: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(probe)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Run the audio-stack probes. Linux-only meaningfully; elsewhere everything
/// reads false except where trivially true.
pub fn audio_stack_health() -> AudioStackHealth {
    // systemd path first; fall back to a process check so non-systemd inits
    // (Gentoo/OpenRC, Void/runit) don't read as "WirePlumber down".
    let wireplumber_active = Command::new("systemctl")
        .args(["--user", "is-active", "wireplumber"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "active")
        .unwrap_or(false)
        || sh_ok("pgrep -x wireplumber >/dev/null 2>&1");

    AudioStackHealth {
        wireplumber_active,
        has_pw_dump: sh_ok("command -v pw-dump >/dev/null 2>&1"),
        // `^pipewire$` line in `aplay -L` = the ALSA->PipeWire bridge PCM.
        cpal_sees_pipewire: sh_ok("aplay -L 2>/dev/null | grep -q '^pipewire$'"),
        has_pactl: sh_ok("command -v pactl >/dev/null 2>&1"),
        any_devices: sh_ok("pw-dump 2>/dev/null | grep -q 'Audio/Sink'"),
    }
}
