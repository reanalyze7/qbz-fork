//! ALSA device-string parsing + D-Bus naming helpers.

use super::error::ReservationError;

/// Parse an ALSA device string and return the kernel card index.
///
/// Accepts any plugin prefix (`hw:`, `plughw:`, `front:`, `surround*:`,
/// `iec958:`, `hdmi:`, etc.) followed by either a positional numeric card
/// index (`hw:1,0`) or a `CARD=<name>` argument in any position
/// (`front:CARD=C20,DEV=0`, `hw:DEV=0,CARD=DacMagic`). The plugin prefix
/// itself is irrelevant for our purpose: D-Bus reservation is per-card,
/// not per-plugin alias.
///
/// Strings with no colon (`default`, `pulse`, etc.) and strings whose
/// args contain neither a `CARD=` argument nor a numeric first arg are
/// rejected as `InvalidDevice`. The caller (`acquire`) downgrades these
/// to a degraded guard rather than propagating the error.
pub(crate) fn parse_card_index(hw_device: &str) -> Result<u32, ReservationError> {
    let trimmed = hw_device.trim();

    // Find the first colon. Everything before is the plugin name (hw,
    // plughw, front, surround*, iec958, hdmi, etc.); we only care about
    // the args after it. Strings with no colon are not plugin-prefixed
    // device names and cannot identify a card.
    let args = match trimmed.find(':') {
        Some(idx) => trimmed[idx + 1..].trim(),
        None => return Err(ReservationError::InvalidDevice(hw_device.to_string())),
    };

    if args.is_empty() {
        return Err(ReservationError::InvalidDevice(hw_device.to_string()));
    }

    // Args are comma-separated. Look for `CARD=<name>` in any position;
    // if none, fall back to the first arg parsed as a numeric card index.
    let mut card_name: Option<&str> = None;
    let mut first_arg: Option<&str> = None;

    for (i, arg) in args.split(',').enumerate() {
        let arg = arg.trim();
        if let Some(name) = arg.strip_prefix("CARD=") {
            card_name = Some(name);
        }
        if i == 0 {
            first_arg = Some(arg);
        }
    }

    if let Some(name) = card_name {
        if name.is_empty() {
            return Err(ReservationError::InvalidDevice(hw_device.to_string()));
        }
        return resolve_card_index_by_name(name);
    }

    if let Some(arg) = first_arg {
        if let Ok(n) = arg.parse::<u32>() {
            return Ok(n);
        }
    }

    Err(ReservationError::InvalidDevice(hw_device.to_string()))
}

/// Resolve a symbolic ALSA card name (e.g., `"C20"`, `"DacMagic"`, `"PCH"`)
/// to its kernel index by iterating over `alsa::card::Iter`.
///
/// ALSA's `CARD=` parameter takes the card's *id* (short identifier like
/// `"C20"`, `"PCH"`, `"Generic"`, `"HDMI"`) — NOT the long descriptive name
/// (`"Cambridge Audio USB Audio 2.0"`). Verified against `aplay -l` output
/// on the user's host:
///
/// ```text
///     card 1: C20 [Cambridge Audio USB Audio 2.0], device 0: ...
///              ^id ^long name
/// ```
///
/// In `alsa-rs` 0.10, the short id is exposed as `ctl::CardInfo::get_id()`
/// (a wrapper around `snd_ctl_card_info_get_id`). The convenience
/// `Card::get_name()` actually returns the long name (it wraps
/// `snd_card_get_name`, which despite the name returns the descriptive
/// "Cambridge Audio USB Audio 2.0" string), so we cannot use it here. We
/// open a `Ctl` per card and read the id from its `CardInfo`.
fn resolve_card_index_by_name(name: &str) -> Result<u32, ReservationError> {
    for card in alsa::card::Iter::new() {
        let card = card.map_err(|e| ReservationError::AlsaError(e.to_string()))?;
        let ctl = alsa::ctl::Ctl::from_card(&card, true)
            .map_err(|e| ReservationError::AlsaError(e.to_string()))?;
        let info = ctl
            .card_info()
            .map_err(|e| ReservationError::AlsaError(e.to_string()))?;
        let id = info
            .get_id()
            .map_err(|e| ReservationError::AlsaError(e.to_string()))?;
        if id == name {
            return Ok(card.get_index() as u32);
        }
    }
    Err(ReservationError::InvalidDevice(format!(
        "ALSA card '{}' not found",
        name
    )))
}

/// Format the well-known D-Bus bus name for a given ALSA card index.
pub(crate) fn bus_name_for_card(card_index: u32) -> String {
    format!("org.freedesktop.ReserveDevice1.Audio{}", card_index)
}

/// Format the D-Bus object path for a given ALSA card index.
pub(crate) fn object_path_for_card(card_index: u32) -> String {
    format!("/org/freedesktop/ReserveDevice1/Audio{}", card_index)
}
