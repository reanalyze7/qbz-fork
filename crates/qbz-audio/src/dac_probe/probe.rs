//! IO-performing probes: shell out to `pw-dump` and read `/proc/asound`.

use std::process::Command;

use super::parse::{parse_alsa_card_for_node, parse_hw_params};
use super::NegotiatedRate;

/// Resolve the ALSA card number for a sink node by running `pw-dump`.
fn alsa_card_for_node(node_name: &str) -> Option<u32> {
    let output = Command::new("pw-dump").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let json = String::from_utf8_lossy(&output.stdout);
    parse_alsa_card_for_node(&json, node_name)
}

/// Read the live hardware params for an ALSA card's playback substream.
fn read_hw_params_for_card(card: u32) -> Option<NegotiatedRate> {
    // The DAC's playback PCM is almost always pcm0p; scan a few in case of
    // multi-PCM cards (e.g. an HDMI PCM at index 0 and analog later).
    for pcm in 0..4 {
        let path = format!("/proc/asound/card{}/pcm{}p/sub0/hw_params", card, pcm);
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Some(nr) = parse_hw_params(&content) {
                return Some(nr);
            }
        }
    }
    None
}

/// Probe the DAC's live negotiated hardware rate for the given PipeWire sink
/// `node.name`. `None` = idle/closed or unresolvable. Read-only; safe to poll.
pub fn negotiated_stream_rate(node_name: &str) -> Option<NegotiatedRate> {
    let card = alsa_card_for_node(node_name)?;
    read_hw_params_for_card(card)
}

/// The negotiated rate of whichever ALSA card is ACTIVELY playing right now.
///
/// Scans every card's playback substream and returns the first one that is
/// open (not `closed`). This is DAC-agnostic — it reports the rate of whatever
/// QBZ is currently outputting to, so it works no matter which (or how many)
/// DACs the user selected; you can only play through one output at a time.
/// `None` = nothing is playing. Read-only; safe to poll.
pub fn negotiated_active_rate() -> Option<NegotiatedRate> {
    for card in 0..16 {
        if let Some(nr) = read_hw_params_for_card(card) {
            return Some(nr);
        }
    }
    None
}
