//! PipeWire audio backend
//!
//! Uses PipeWire/PulseAudio for audio output with device selection.
//! - Enumerates devices using pactl (pretty names)
//! - Sets PULSE_SINK environment variable for device routing
//! - Creates stream using CPAL "pulse" or "pipewire" device
//! - Does NOT change system default (only affects QBZ)

mod enumerate_pactl;
mod enumerate_pwdump;
mod probe;
mod rates;
mod rates_fallback;
mod stream;

use super::backend::{AudioBackend, AudioBackendType, AudioDevice, BackendConfig, BackendResult};
use rodio::MixerDeviceSink;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

/// Set true whenever QBZ writes a global `clock.force-rate` to the PipeWire
/// graph, so `reset_pipewire_clock` only resets a force WE applied and never
/// clobbers another app's intentional forced rate (issue #263, leak fix).
/// The force and the reset both run on the audio thread, so `Relaxed` is enough.
pub(crate) static CLOCK_FORCE_APPLIED: AtomicBool = AtomicBool::new(false);

/// Restores the `PIPEWIRE_NODE` env var to its previous value when dropped, so
/// locked-mode sink targeting (Tier 2a, #263) does not leak into later stream
/// opens. Edition 2021: `set_var`/`remove_var` are safe.
#[cfg(target_os = "linux")]
pub(crate) struct PwNodeEnvGuard(pub(crate) Option<String>);

#[cfg(target_os = "linux")]
impl Drop for PwNodeEnvGuard {
    fn drop(&mut self) {
        match self.0.take() {
            Some(v) => std::env::set_var("PIPEWIRE_NODE", v),
            None => std::env::remove_var("PIPEWIRE_NODE"),
        }
    }
}

pub struct PipeWireBackend {
    #[allow(dead_code)]
    host: rodio::cpal::Host,
}

impl PipeWireBackend {
    pub fn new() -> BackendResult<Self> {
        Ok(Self {
            host: rodio::cpal::default_host(),
        })
    }

    /// Reset PipeWire clock.force-rate and clock.force-quantum to 0.
    /// Call this when playback stops so other apps aren't stuck at a forced rate.
    /// Quantum reset is kept for safety even though we no longer force it.
    pub fn reset_pipewire_clock() {
        // Only reset a force WE applied — otherwise stopping QBZ would clobber
        // another app's intentional clock.force-rate. This also makes the call
        // safe to invoke unconditionally on every stop/suspend (issue #263 leak
        // fix): previously the reset was gated on `pw_force_bitperfect`, but QBZ
        // forces the clock for ANY non-locked PipeWire stream, so a plain
        // PipeWire (no-passthrough) user left the graph force-clocked after stop.
        if !CLOCK_FORCE_APPLIED.swap(false, Ordering::Relaxed) {
            return;
        }
        log::info!("[PipeWire Backend] Resetting clock.force-rate and clock.force-quantum to 0");
        let _ = Command::new("pw-metadata")
            .args(["-n", "settings", "0", "clock.force-rate", "0"])
            .output();
        let _ = Command::new("pw-metadata")
            .args(["-n", "settings", "0", "clock.force-quantum", "0"])
            .output();
    }
}

impl AudioBackend for PipeWireBackend {
    fn backend_type(&self) -> AudioBackendType {
        AudioBackendType::PipeWire
    }

    fn enumerate_devices(&self) -> BackendResult<Vec<AudioDevice>> {
        // Primary: native PipeWire via `pw-dump`. Works on PipeWire-only systems
        // that are missing `pipewire-alsa` / `pipewire-pulse` (the Ubuntu
        // empty-list bug) and yields the exact `alsa_output.*` node.name.
        if let Some(devices) = self.enumerate_via_pw_dump() {
            return Ok(devices);
        }
        // Fallback: `pactl` (requires pipewire-pulse + pulseaudio-utils).
        log::info!(
            "[PipeWire Backend] pw-dump unavailable or empty — falling back to pactl enumeration"
        );
        self.enumerate_pipewire_sinks()
    }

    fn create_output_stream(&self, config: &BackendConfig) -> BackendResult<MixerDeviceSink> {
        stream::create_output_stream(config)
    }

    fn is_available(&self) -> bool {
        // #591/#592: this probe used to require `pactl` (pulseaudio-utils +
        // pipewire-pulse). Sandboxed and minimal installs (Snap, Flatpak, .deb
        // without pulseaudio-utils) can lack it even though PipeWire itself is
        // up — device enumeration already succeeds natively via `pw-dump` on
        // those systems, so the availability probe must accept the same proof.
        // Order: native socket (no subprocess) → pw-dump → pactl (kept as the
        // fallback for PulseAudio-only systems, preserving pre-2.0 behavior).
        // Subprocess probes are time-bounded so a wedged daemon can never
        // stall stream init on the audio thread.
        if let Some(runtime_dir) = std::env::var_os("XDG_RUNTIME_DIR") {
            if std::path::Path::new(&runtime_dir).join("pipewire-0").exists() {
                return true;
            }
        }
        probe::probe_command_ok("pw-dump", &[], std::time::Duration::from_secs(3))
            || probe::probe_command_ok("pactl", &["info"], std::time::Duration::from_secs(3))
    }

    fn description(&self) -> &'static str {
        "PipeWire (Recommended) - Modern audio server with device sharing"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
