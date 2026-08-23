//! Quality-tier / quality-label / release-type helpers shared by the album
//! and slim-grid mappers.

use qbz_models::DiscoverAudioInfo;

/// Classify a Discover album's release type for the list-row TYPE column.
/// The Discover index carries no explicit release_type, so this mirrors
/// DiscographyBuilderView's track-count fallback heuristic (<=3 = Single,
/// <=6 = EP, otherwise Album).
pub(in crate::home) fn classify_release_type(track_count: Option<u32>) -> &'static str {
    match track_count {
        Some(n) if n <= 3 => "Single",
        Some(n) if n <= 6 => "EP",
        _ => "Album",
    }
}

/// Bare exact-quality detail for QualityBadgeFull's detail line, e.g.
/// "24-bit / 96 kHz" (no "Hi-Res:" prefix — the badge supplies the tier
/// label itself). Empty when the entry carries no audio info.
pub(in crate::home) fn quality_detail(audio: Option<&DiscoverAudioInfo>) -> String {
    let Some(audio) = audio else {
        return String::new();
    };
    let hi_res = matches!(audio.maximum_bit_depth, Some(depth) if depth >= 24);
    let depth = audio
        .maximum_bit_depth
        .unwrap_or(if hi_res { 24 } else { 16 });
    let rate = audio
        .maximum_sampling_rate
        .unwrap_or(if hi_res { 96.0 } else { 44.1 });
    format!("{depth}-bit / {} kHz", format_rate(rate))
}

/// Classify the quality tier for the icon-only badge: 24-bit and up is
/// Hi-Res, anything else with audio info is CD-quality.
pub(in crate::home) fn quality_tier(audio: Option<&DiscoverAudioInfo>) -> &'static str {
    let Some(audio) = audio else {
        return "";
    };
    match audio.maximum_bit_depth {
        Some(depth) if depth >= 24 => "hires",
        _ => "cd",
    }
}

/// Exact-quality label for the badge hover tooltip, mirroring the Tauri
/// `QualityBadge` (`{tier}: {depth}-bit / {rate} kHz`). Empty when the
/// discover entry carries no audio info, matching `quality_tier`.
pub(in crate::home) fn quality_label(audio: Option<&DiscoverAudioInfo>) -> String {
    let Some(audio) = audio else {
        return String::new();
    };
    let hi_res = matches!(audio.maximum_bit_depth, Some(depth) if depth >= 24);
    let tier = if hi_res { "Hi-Res" } else { "CD" };
    let depth = audio
        .maximum_bit_depth
        .unwrap_or(if hi_res { 24 } else { 16 });
    let rate = audio
        .maximum_sampling_rate
        .unwrap_or(if hi_res { 96.0 } else { 44.1 });
    format!("{tier}: {depth}-bit / {} kHz", format_rate(rate))
}

/// Format a kHz sample rate without a trailing `.0` (96.0 -> "96",
/// 44.1 -> "44.1").
pub(in crate::home) fn format_rate(rate: f64) -> String {
    if (rate.fract()).abs() < f64::EPSILON {
        format!("{}", rate as i64)
    } else {
        format!("{rate}")
    }
}
