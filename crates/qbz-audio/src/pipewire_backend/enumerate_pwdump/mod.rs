//! Native-PipeWire enumeration via `pw-dump`.
//!
//! This talks to the PipeWire daemon over its own socket and needs ONLY
//! `pipewire-bin` (the `pw-*` tools) — it does NOT require `pipewire-pulse`
//! (the Pulse-protocol server `pactl` talks to) nor the `pipewire-alsa`
//! bridge PCM that CPAL relies on. On a PipeWire-only box missing those
//! packages (the reported Ubuntu "empty sink list" bug) the legacy `pactl`
//! and CPAL paths return nothing, but `pw-dump` still reports every sink —
//! and gives us the exact `alsa_output.*` `node.name` for free.

mod parse;
#[cfg(test)]
mod tests;

use super::PipeWireBackend;
use crate::backend::AudioDevice;
use parse::parse_pw_dump_sinks;
use std::process::Command;

impl PipeWireBackend {
    /// Returns `None` when `pw-dump` is absent, fails, or finds no sink (so the
    /// caller falls back to `pactl`). Discovery only — never touches the
    /// stream-open path.
    pub(crate) fn enumerate_via_pw_dump(&self) -> Option<Vec<AudioDevice>> {
        let output = Command::new("pw-dump").output().ok()?;
        if !output.status.success() {
            return None;
        }
        let json = String::from_utf8_lossy(&output.stdout);
        let devices = parse_pw_dump_sinks(&json);
        if devices.is_empty() {
            return None;
        }
        log::info!(
            "[PipeWire Backend] Enumerated {} sink(s) via pw-dump (native, no pactl/pipewire-pulse needed)",
            devices.len()
        );
        for (idx, dev) in devices.iter().enumerate() {
            log::info!(
                "  [{}] {} (id: {}, bus: {:?}, hw: {}, default: {})",
                idx, dev.name, dev.id, dev.device_bus, dev.is_hardware, dev.is_default
            );
        }
        Some(devices)
    }
}
