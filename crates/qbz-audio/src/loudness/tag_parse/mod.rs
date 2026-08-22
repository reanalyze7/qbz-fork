//! Pure parsing of Symphonia tag `Value`s into ReplayGain numbers.

#[cfg(test)]
mod tests;

use symphonia::core::meta::Value;

/// Parse a ReplayGain gain value like "-6.54 dB" or "-6.54"
pub(super) fn parse_gain_value(value: &Value) -> Option<f32> {
    let s = value_to_string(value)?;
    // Strip " dB" suffix if present, then parse
    let trimmed = s
        .trim()
        .trim_end_matches(" dB")
        .trim_end_matches(" db")
        .trim_end_matches("dB");
    trimmed.parse::<f32>().ok()
}

/// Parse a ReplayGain peak value like "0.988553" or "1.0"
pub(super) fn parse_peak_value(value: &Value) -> Option<f32> {
    let s = value_to_string(value)?;
    s.trim().parse::<f32>().ok()
}

/// Convert a Symphonia Value to a string representation
fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Float(f) => Some(f.to_string()),
        Value::SignedInt(i) => Some(i.to_string()),
        Value::UnsignedInt(u) => Some(u.to_string()),
        _ => None,
    }
}
