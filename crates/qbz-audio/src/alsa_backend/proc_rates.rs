//! Hardware-supported sample rate reading from /proc/asound/cardN/stream0.

use super::proc_cards::find_card_number_by_name;
use std::fs;

/// Read hardware-supported sample rates from /proc/asound/cardN/stream0.
/// Returns None if rates cannot be determined (treat as "try anyway").
pub(super) fn get_hw_supported_rates(card_name: &str) -> Option<Vec<u32>> {
    let card_num = find_card_number_by_name(card_name)?;
    let stream_path = format!("/proc/asound/card{}/stream0", card_num);
    let content = fs::read_to_string(&stream_path).ok()?;

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
                return None; // Any rate supported — don't restrict
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
