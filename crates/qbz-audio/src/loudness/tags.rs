//! Search Symphonia tag lists for ReplayGain values.

use symphonia::core::meta::{StandardTagKey, Tag};

use super::tag_parse::{parse_gain_value, parse_peak_value};

pub(super) fn extract_from_tags(tags: &[Tag], gain_db: &mut Option<f32>, peak: &mut Option<f32>) {
    for tag in tags {
        // Check Symphonia's standard tag key mapping first
        if let Some(std_key) = tag.std_key {
            match std_key {
                StandardTagKey::ReplayGainTrackGain => {
                    if let Some(g) = parse_gain_value(&tag.value) {
                        *gain_db = Some(g);
                    }
                }
                StandardTagKey::ReplayGainTrackPeak => {
                    if let Some(p) = parse_peak_value(&tag.value) {
                        *peak = Some(p);
                    }
                }
                _ => {}
            }
        }

        // Also check raw tag keys (case-insensitive) for formats where
        // Symphonia might not map to StandardTagKey
        let key_lower = tag.key.to_lowercase();
        match key_lower.as_str() {
            "replaygain_track_gain" => {
                if gain_db.is_none() {
                    if let Some(g) = parse_gain_value(&tag.value) {
                        *gain_db = Some(g);
                    }
                }
            }
            "replaygain_track_peak" => {
                if peak.is_none() {
                    if let Some(p) = parse_peak_value(&tag.value) {
                        *peak = Some(p);
                    }
                }
            }
            _ => {}
        }
    }
}
