//! Public device-id helpers re-exported at `alsa_backend::` — stable-id
//! normalization/resolution and hardware sample-rate queries.

use super::device_id::extract_card_name_from_device;
use super::proc_cards::{build_card_info_map, find_card_number_by_name};
use super::proc_rates::get_hw_supported_rates;

/// Convert an unstable hw:X,0 device ID to a stable front:CARD=name,DEV=0 format.
/// This survives reboots and USB reconnections since it uses the card name, not the number.
///
/// Examples:
/// - `hw:0,0` with card "C20" -> `front:CARD=C20,DEV=0`
/// - `hw:2,0` with card "NVidia" -> `front:CARD=NVidia,DEV=0`
/// - `front:CARD=C20,DEV=0` -> unchanged (already stable)
/// - `plughw:0,0` -> unchanged (plugin devices don't benefit from this)
/// - `default` -> unchanged (not a hardware device)
pub fn normalize_device_id_to_stable(device_id: &str) -> String {
    // Already stable formats - return as-is
    if device_id.starts_with("front:CARD=")
        || device_id.starts_with("plughw:")
        || !device_id.starts_with("hw:")
    {
        return device_id.to_string();
    }

    // Parse hw:X,Y format
    let stripped = device_id.strip_prefix("hw:").unwrap_or(device_id);
    let parts: Vec<&str> = stripped.split(',').collect();
    if parts.len() < 2 {
        log::warn!("[ALSA] Could not parse hw device format: {}", device_id);
        return device_id.to_string();
    }

    let card_num = parts[0];
    let device_num = parts[1];

    // Get card info from /proc/asound
    let card_map = build_card_info_map();

    if let Some((short_name, _long_name)) = card_map.get(card_num) {
        let stable_id = format!("front:CARD={},DEV={}", short_name, device_num);
        log::info!(
            "[ALSA] Normalized device ID: {} -> {} (stable)",
            device_id,
            stable_id
        );
        return stable_id;
    }

    log::warn!(
        "[ALSA] Could not find card {} in /proc/asound, keeping original ID",
        card_num
    );
    device_id.to_string()
}

/// Get the current card number for a stable device ID.
/// Used when we need to resolve front:CARD=X to hw:N,0 for certain operations.
///
/// Returns None if the card is not currently present.
pub fn resolve_stable_to_current_hw(device_id: &str) -> Option<String> {
    // Only resolve front:CARD= format
    if !device_id.starts_with("front:CARD=") {
        return Some(device_id.to_string());
    }

    // Extract card name: front:CARD=C20,DEV=0 -> C20
    let stripped = device_id.strip_prefix("front:CARD=")?;
    let parts: Vec<&str> = stripped.split(',').collect();
    let card_name = parts.first()?;
    let dev_part = parts
        .get(1)
        .and_then(|s| s.strip_prefix("DEV="))
        .unwrap_or("0");

    // Find current card number for this name using /proc/asound
    if let Some(card_num) = find_card_number_by_name(card_name) {
        let hw_id = format!("hw:{},{}", card_num, dev_part);
        log::debug!("[ALSA] Resolved {} -> {}", device_id, hw_id);
        return Some(hw_id);
    }

    log::warn!(
        "[ALSA] Card '{}' not found in current enumeration",
        card_name
    );
    None
}

/// Check if a hardware device supports a given sample rate.
/// Returns `Some(true)` if supported, `Some(false)` if not, `None` if unknown.
/// Uses /proc/asound/cardN/stream0 for accurate hardware capabilities.
pub fn device_supports_sample_rate(device_id: &str, sample_rate: u32) -> Option<bool> {
    let card_name = extract_card_name_from_device(device_id)?;
    let hw_rates = get_hw_supported_rates(&card_name)?;
    Some(hw_rates.contains(&sample_rate))
}

/// Get the hardware-supported sample rates for a device.
/// Returns None if rates cannot be determined.
pub fn get_device_supported_rates(device_id: &str) -> Option<Vec<u32>> {
    let card_name = extract_card_name_from_device(device_id)?;
    get_hw_supported_rates(&card_name)
}
