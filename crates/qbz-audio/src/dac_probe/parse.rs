//! Pure parsers for ALSA hw_params text and PipeWire `pw-dump` JSON.

use super::NegotiatedRate;

#[cfg(test)]
#[path = "parse_tests.rs"]
mod tests;

/// Parse the contents of `/proc/asound/cardN/pcm*p/sub*/hw_params`.
///
/// Returns `None` when the device is idle (`closed`), empty, or has no `rate:`
/// line. Pure (no I/O) so it is unit-testable against captured fixtures.
pub fn parse_hw_params(content: &str) -> Option<NegotiatedRate> {
    let trimmed = content.trim();
    if trimmed.is_empty() || trimmed == "closed" {
        return None;
    }
    let mut sample_rate: Option<u32> = None;
    let mut format: Option<String> = None;
    let mut channels: Option<u32> = None;
    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("rate:") {
            // "rate: 192000 (192000/1)" -> 192000
            sample_rate = rest
                .trim()
                .split_whitespace()
                .next()
                .and_then(|s| s.parse().ok());
        } else if let Some(rest) = line.strip_prefix("format:") {
            format = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("channels:") {
            channels = rest.trim().parse().ok();
        }
    }
    let sample_rate = sample_rate?;
    Some(NegotiatedRate {
        sample_rate,
        format: format.unwrap_or_default(),
        channels: channels.unwrap_or(0),
    })
}

/// Pure helper: find the ALSA card number backing a PipeWire sink `node.name`
/// in `pw-dump` JSON. Reads `api.alsa.pcm.card` / `alsa.card` (string or int).
pub fn parse_alsa_card_for_node(json: &str, node_name: &str) -> Option<u32> {
    let root: serde_json::Value = serde_json::from_str(json).ok()?;
    let arr = root.as_array()?;
    for obj in arr {
        if obj.get("type").and_then(|v| v.as_str()) != Some("PipeWire:Interface:Node") {
            continue;
        }
        let props = match obj.get("info").and_then(|i| i.get("props")) {
            Some(p) => p,
            None => continue,
        };
        if props.get("node.name").and_then(|v| v.as_str()) != Some(node_name) {
            continue;
        }
        for key in ["api.alsa.pcm.card", "alsa.card", "card.id"] {
            if let Some(v) = props.get(key) {
                if let Some(n) = v.as_u64() {
                    return Some(n as u32);
                }
                if let Some(n) = v.as_str().and_then(|s| s.parse::<u32>().ok()) {
                    return Some(n);
                }
            }
        }
        return None; // matched the node but it carries no card property
    }
    None
}

