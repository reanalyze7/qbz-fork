//! /proc/asound card enumeration — no `aplay`/`alsa-utils` dependency.

use super::proc_pcm::{read_card_pcm_devices, ProcPcmInfo};
use std::collections::HashMap;
use std::fs;

/// Information about an ALSA sound card read from /proc/asound
#[derive(Debug, Clone)]
pub(super) struct ProcCardInfo {
    /// Card number (0, 1, 2, ...)
    pub(super) number: String,
    /// Short name used in ALSA device IDs (e.g., "C20", "NVidia", "sofhdadsp")
    pub(super) short_name: String,
    /// Long descriptive name (e.g., "Cambridge Audio USB Audio 2.0")
    pub(super) long_name: String,
    /// PCM playback devices on this card
    pub(super) pcm_playback_devices: Vec<ProcPcmInfo>,
}

/// Read all sound card information from /proc/asound
pub(super) fn read_proc_asound_cards() -> Vec<ProcCardInfo> {
    let mut cards = Vec::new();

    // Parse /proc/asound/cards for basic card info
    // Format: " 0 [C20            ]: USB-Audio - Cambridge Audio USB Audio 2.0"
    let cards_content = match fs::read_to_string("/proc/asound/cards") {
        Ok(c) => c,
        Err(e) => {
            log::warn!("[ALSA] Cannot read /proc/asound/cards: {}", e);
            return cards;
        }
    };

    // Parse cards file - each card has two lines
    let lines: Vec<&str> = cards_content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();

        // First line format: " 0 [C20            ]: USB-Audio - Cambridge Audio USB Audio 2.0"
        if let Some(card_info) = parse_proc_card_line(line) {
            // Read PCM devices for this card
            let pcm_devices = read_card_pcm_devices(&card_info.0);

            cards.push(ProcCardInfo {
                number: card_info.0,
                short_name: card_info.1,
                long_name: card_info.2,
                pcm_playback_devices: pcm_devices,
            });
        }
        i += 1;
    }

    cards
}

/// Parse a line from /proc/asound/cards
/// Returns (card_number, short_name, long_name)
fn parse_proc_card_line(line: &str) -> Option<(String, String, String)> {
    // Format: " 0 [C20            ]: USB-Audio - Cambridge Audio USB Audio 2.0"
    let line = line.trim();

    // Find card number (first number)
    let parts: Vec<&str> = line.splitn(2, '[').collect();
    if parts.len() < 2 {
        return None;
    }

    let card_num = parts[0].trim().to_string();
    if card_num.is_empty() || !card_num.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    // Find short name (inside brackets)
    let rest = parts[1];
    let bracket_end = rest.find(']')?;
    let short_name = rest[..bracket_end].trim().to_string();

    // Find long name (after " - ")
    let long_name = if let Some(dash_pos) = rest.find(" - ") {
        rest[dash_pos + 3..].trim().to_string()
    } else {
        // Fallback: use everything after ]:
        rest[bracket_end + 1..]
            .trim()
            .trim_start_matches(':')
            .trim()
            .split(" - ")
            .last()
            .unwrap_or(&short_name)
            .to_string()
    };

    Some((card_num, short_name, long_name))
}

/// Build a map of card_number -> (short_name, long_name) from /proc/asound
pub(super) fn build_card_info_map() -> HashMap<String, (String, String)> {
    let cards = read_proc_asound_cards();
    let mut map = HashMap::new();

    for card in cards {
        map.insert(card.number.clone(), (card.short_name, card.long_name));
    }

    map
}

/// Find card number by short name (e.g., "C20" -> "0")
pub(super) fn find_card_number_by_name(short_name: &str) -> Option<String> {
    let cards = read_proc_asound_cards();
    cards
        .iter()
        .find(|c| c.short_name == short_name)
        .map(|c| c.number.clone())
}
