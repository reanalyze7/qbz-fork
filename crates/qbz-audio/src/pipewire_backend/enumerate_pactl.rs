//! Pactl-based sink enumeration — the legacy fallback path used when
//! `pw-dump` is unavailable or returns nothing.

use super::PipeWireBackend;
use crate::backend::{AudioDevice, BackendResult};
use std::process::Command;

impl PipeWireBackend {
    /// Parse pactl output to get device list with pretty names
    pub(crate) fn enumerate_pipewire_sinks(&self) -> BackendResult<Vec<AudioDevice>> {
        // Get default sink
        let default_sink = Command::new("pactl")
            .args(["get-default-sink"])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    String::from_utf8(o.stdout)
                        .ok()
                        .map(|s| s.trim().to_string())
                } else {
                    None
                }
            });

        // Get all sinks with details
        let output = Command::new("pactl")
            .args(["list", "sinks"])
            .output()
            .map_err(|e| format!("Failed to run pactl: {}", e))?;

        if !output.status.success() {
            return Err("pactl command failed".to_string());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut devices = Vec::new();

        // Parse pactl output
        let mut current_name: Option<String> = None;
        let mut current_description: Option<String> = None;
        let mut current_max_rate: Option<u32> = None;
        let mut current_is_hardware: bool = false;
        let mut current_device_bus: Option<String> = None;

        for line in stdout.lines() {
            let line = line.trim();

            if line.starts_with("Sink #") {
                // Save previous device if complete
                if let (Some(id), Some(name)) = (current_name.take(), current_description.take()) {
                    let is_default = default_sink.as_ref().map(|d| d == &id).unwrap_or(false);
                    devices.push(AudioDevice {
                        id: id.clone(),
                        name,
                        description: None,
                        is_default,
                        max_sample_rate: current_max_rate.take(),
                        supported_sample_rates: None, // PipeWire handles sample rate conversion
                        device_bus: current_device_bus.take(),
                        is_hardware: current_is_hardware,
                    });
                }
                current_max_rate = None;
                current_is_hardware = false;
                current_device_bus = None;
            } else if line.starts_with("Name:") {
                current_name = Some(line.trim_start_matches("Name:").trim().to_string());
            } else if line.starts_with("Description:") {
                current_description =
                    Some(line.trim_start_matches("Description:").trim().to_string());
            } else if line.starts_with("Flags:") {
                // Check for HARDWARE flag
                current_is_hardware = line.contains("HARDWARE");
            } else if line.contains("Sample Specification:") {
                // Try to parse sample rate from lines like "Sample Specification: s32le 2ch 192000Hz"
                if let Some(hz_pos) = line.find("Hz") {
                    let before_hz = &line[..hz_pos];
                    if let Some(last_space) = before_hz.rfind(' ') {
                        if let Ok(rate) = before_hz[last_space + 1..].parse::<u32>() {
                            current_max_rate = Some(rate);
                        }
                    }
                }
            } else if line.starts_with("device.bus = ") {
                // Parse device.bus property (e.g., "usb", "pci", "bluetooth")
                let bus = line
                    .trim_start_matches("device.bus = ")
                    .trim_matches('"')
                    .to_string();
                current_device_bus = Some(bus);
            }
        }

        // Don't forget the last device
        if let (Some(id), Some(name)) = (current_name, current_description) {
            let is_default = default_sink.as_ref().map(|d| d == &id).unwrap_or(false);
            devices.push(AudioDevice {
                id,
                name,
                description: None,
                is_default,
                max_sample_rate: current_max_rate,
                supported_sample_rates: None, // PipeWire handles sample rate conversion
                device_bus: current_device_bus,
                is_hardware: current_is_hardware,
            });
        }

        log::info!(
            "[PipeWire Backend] Enumerated {} devices via pactl",
            devices.len()
        );
        for (idx, dev) in devices.iter().enumerate() {
            log::info!(
                "  [{}] {} (id: {}, bus: {:?}, hw: {})",
                idx,
                dev.name,
                dev.id,
                dev.device_bus,
                dev.is_hardware
            );
        }

        Ok(devices)
    }
}
