//! DAC sample-rate discovery: ALSA capability probing, the current PipeWire
//! graph rate, and same-family fallback-rate selection.

use super::PipeWireBackend;
use std::process::Command;

impl PipeWireBackend {
    /// Get the ALSA card number for a PipeWire/PulseAudio sink name.
    /// Parses `pactl list sinks` to find the `alsa.card` property.
    pub(crate) fn get_alsa_card_for_sink(sink_name: &str) -> Option<String> {
        let output = Command::new("pactl")
            .args(["list", "sinks"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut in_target_sink = false;

        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Sink #") {
                if in_target_sink {
                    return None; // Passed target sink without finding alsa.card
                }
            } else if trimmed.starts_with("Name:") {
                let name = trimmed.trim_start_matches("Name:").trim();
                in_target_sink = name == sink_name;
            } else if in_target_sink && trimmed.starts_with("alsa.card = ") {
                let card = trimmed
                    .trim_start_matches("alsa.card = ")
                    .trim_matches('"')
                    .to_string();
                return Some(card);
            }
        }

        None
    }

    /// Query the DAC's supported sample rates from /proc/asound/cardN/stream0.
    /// Returns None if rates can't be determined (non-USB device, continuous range, etc.)
    pub fn get_sink_supported_rates(sink_name: &str) -> Option<Vec<u32>> {
        let alsa_card = Self::get_alsa_card_for_sink(sink_name)?;

        let stream_path = format!("/proc/asound/card{}/stream0", alsa_card);
        let content = std::fs::read_to_string(&stream_path).ok()?;

        // Collect all rates from Playback Rates: lines (handles multiple alt settings)
        let mut in_playback = false;
        let mut all_rates = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "Playback:" {
                in_playback = true;
            } else if trimmed == "Capture:" {
                in_playback = false;
            }
            if in_playback && trimmed.starts_with("Rates:") {
                let rates_str = trimmed.trim_start_matches("Rates:").trim();
                if rates_str.contains("continuous") {
                    return None; // Any rate in range is supported
                }
                for rate_str in rates_str.split(',') {
                    if let Ok(rate) = rate_str.trim().parse::<u32>() {
                        if !all_rates.contains(&rate) {
                            all_rates.push(rate);
                        }
                    }
                }
            }
        }

        if all_rates.is_empty() {
            None
        } else {
            all_rates.sort();
            Some(all_rates)
        }
    }

    /// Query the current PipeWire graph sample rate via pw-metadata.
    pub(crate) fn get_pipewire_current_rate() -> Option<u32> {
        let output = Command::new("pw-metadata")
            .args(["-n", "settings", "0", "clock.rate"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        // pw-metadata output: "Found "settings" metadata 0\nupdate: id:0 key:'clock.rate' value:'96000' type:''"
        for line in stdout.lines() {
            if line.contains("clock.rate") && line.contains("value:") {
                // Extract value between single quotes after "value:"
                if let Some(start) = line.find("value:'") {
                    let after = &line[start + 7..];
                    if let Some(end) = after.find('\'') {
                        return after[..end].parse::<u32>().ok();
                    }
                }
            }
        }
        None
    }
}
