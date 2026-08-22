//! /proc/asound PCM device reading (per-card playback devices).

use std::fs;

/// Information about a PCM device
#[derive(Debug, Clone)]
pub(super) struct ProcPcmInfo {
    /// Device number within the card
    pub(super) device_num: String,
    /// Device name (e.g., "USB Audio", "HDMI 0")
    pub(super) name: String,
}

/// Read PCM playback devices for a specific card from /proc/asound
pub(super) fn read_card_pcm_devices(card_num: &str) -> Vec<ProcPcmInfo> {
    let mut devices = Vec::new();
    let card_path = format!("/proc/asound/card{}", card_num);

    // Read PCM device info files
    if let Ok(entries) = fs::read_dir(&card_path) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // PCM playback devices are named pcmXp (X = device number, p = playback)
            if name_str.starts_with("pcm") && name_str.ends_with('p') {
                let info_path = entry.path().join("info");
                if let Ok(content) = fs::read_to_string(&info_path) {
                    let mut pcm_name = String::new();
                    let mut device_num = String::new();

                    for line in content.lines() {
                        if let Some(val) = line.strip_prefix("name: ") {
                            pcm_name = val.trim().to_string();
                        }
                        if let Some(val) = line.strip_prefix("device: ") {
                            device_num = val.trim().to_string();
                        }
                    }

                    if !device_num.is_empty() {
                        devices.push(ProcPcmInfo {
                            device_num,
                            name: if pcm_name.is_empty() {
                                "Unknown".to_string()
                            } else {
                                pcm_name
                            },
                        });
                    }
                }
            }
        }
    }

    // Sort by device number
    devices.sort_by(|a, b| {
        a.device_num
            .parse::<u32>()
            .unwrap_or(0)
            .cmp(&b.device_num.parse::<u32>().unwrap_or(0))
    });

    devices
}
