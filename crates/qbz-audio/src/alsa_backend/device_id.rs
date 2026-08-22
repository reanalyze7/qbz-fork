//! Device-id string manipulation for ALSA PCM names.

use super::proc_cards::{find_card_number_by_name, read_proc_asound_cards};

/// Return `true` when the given CPAL/ALSA PCM name matches one of the ID
/// shapes our `/proc/asound`-driven enumeration ever looks up.
///
/// Used by `build_cpal_device_map` to drop virtual PCMs (dmix, route,
/// surround*, pulse, null, …) whose probing only produces noise.
pub(super) fn is_known_pcm_id(name: &str) -> bool {
    name == "default"
        || name.starts_with("sysdefault:CARD=")
        || name.starts_with("front:CARD=")
        || name.starts_with("hdmi:CARD=")
        || name.starts_with("iec958:CARD=")
}

/// Build a `hw:CARD=<name>,DEV=<n>` fallback id from an aliased device id.
/// Returns None for ids that don't carry a `CARD=<name>,DEV=<n>` shape
/// (e.g. `default`, `hw:0,0`, unknown formats).
///
/// The raw `hw:` PCM is defined by the kernel driver for every card the
/// system lists in /proc/asound, which makes it a safe last-resort when
/// higher-level aliases like `iec958:` or `front:` aren't declared in the
/// user's asound.conf (issue #331 — minimal Raspberry Pi OS installs don't
/// ship a config entry for `iec958:CARD=<name>`, so the HifiBerry Digi2
/// Pro's only selectable id failed to open even though the card was
/// present and usable via `hw:`).
pub(super) fn build_hw_fallback_id(device_id: &str) -> Option<String> {
    if !(device_id.starts_with("front:CARD=")
        || device_id.starts_with("sysdefault:CARD=")
        || device_id.starts_with("iec958:CARD=")
        || device_id.starts_with("hdmi:CARD="))
    {
        return None;
    }
    let after_card = device_id.split("CARD=").nth(1)?;
    let mut parts = after_card.splitn(2, ',');
    let card_name = parts.next()?.to_string();
    let dev_part = parts.next().unwrap_or("DEV=0");
    let dev_num = dev_part.strip_prefix("DEV=").unwrap_or("0");
    Some(format!("hw:CARD={},DEV={}", card_name, dev_num))
}

/// Derive the raw `(hw, plughw)` open ids for an aliased device id.
///
/// Aliases like `front:` are resolved by alsa-lib through per-card config
/// (`cards.<driver>.pcm.front.<DEV>`), which some cards don't declare for
/// every device — snd-aloop's `Loopback.conf` defines `front` for DEV=0
/// only, and there it routes through softvol — so opening the alias itself
/// can fail with ENOENT at config-expansion time, or add an unwanted plugin
/// stage (discussion #641). The raw ids take the same `CARD=`/`DEV=` args
/// directly from the kernel driver, with no config dependency.
///
/// Returns None for ids that carry no `CARD=` alias shape (raw `hw:`,
/// `default`, etc.) — the caller keeps its existing handling for those.
pub(super) fn raw_open_ids(device_id: &str) -> Option<(String, String)> {
    let hw = build_hw_fallback_id(device_id)?;
    let plug = hw.replacen("hw:", "plughw:", 1);
    Some((hw, plug))
}

/// Extract card name from an ALSA device ID.
/// `front:CARD=C20,DEV=0` -> `"C20"`, `hw:0,0` -> card 0 short name, etc.
///
/// Handles every alias shape `enumerate_with_proc_descriptions` produces:
/// `front:CARD=`, `sysdefault:CARD=`, `iec958:CARD=`, `hdmi:CARD=`, and the
/// raw `hw:N,M` / `plughw:N,M` forms. Missing any of these meant the
/// device-not-found error message downgraded to the generic "disconnected,
/// renamed, or handled by another app" wording for S/PDIF / HDMI devices
/// (issue #331 — HifiBerry Digi2 Pro is S/PDIF-only, so its selected id is
/// `iec958:CARD=sndrpihifiberry,DEV=0`).
pub(super) fn extract_card_name_from_device(device_id: &str) -> Option<String> {
    if device_id.starts_with("front:CARD=")
        || device_id.starts_with("sysdefault:CARD=")
        || device_id.starts_with("iec958:CARD=")
        || device_id.starts_with("hdmi:CARD=")
    {
        // <prefix>:CARD=<name>,DEV=<n> -> <name>
        let after_card = device_id.split("CARD=").nth(1)?;
        Some(after_card.split(',').next()?.to_string())
    } else if device_id.starts_with("hw:") || device_id.starts_with("plughw:") {
        // hw:0,0 -> card number 0 -> look up short name
        let prefix = if device_id.starts_with("hw:") {
            "hw:"
        } else {
            "plughw:"
        };
        let card_num = device_id.strip_prefix(prefix)?.split(',').next()?;
        let cards = read_proc_asound_cards();
        cards
            .iter()
            .find(|c| c.number == card_num)
            .map(|c| c.short_name.clone())
    } else {
        None
    }
}

/// Check whether the card referenced by an ALSA device id is registered
/// in `/proc/asound/cards`. Used as a presence gate before attempting the
/// `hw:CARD=…,DEV=…` fallback in `create_output_stream` — we only want to
/// retry against the raw kernel PCM when the card actually exists.
///
/// Distinct from `get_hw_supported_rates`: that one parses
/// `/proc/asound/cardN/stream0`, which is only emitted by the USB-audio
/// driver. I2S / PCI / built-in cards (HifiBerry, intel-hda, etc.) don't
/// have a stream0 file at all, so using rate-readability as a presence
/// check made the fallback a no-op for the very devices that needed it
/// (issue #331 — HifiBerry Digi2 Pro on Raspberry Pi OS).
pub(super) fn is_card_present_in_proc(device_id: &str) -> bool {
    extract_card_name_from_device(device_id)
        .and_then(|card| find_card_number_by_name(&card))
        .is_some()
}
