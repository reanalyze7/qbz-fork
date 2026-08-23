//! Small pure formatters and quality-resolution helpers shared by the
//! metadata sync, the poll loop, the engine, and the enqueue commands.

use qbz_models::{Quality, QualityLimit};

use super::Runtime;

/// Streaming quality for playback, resolved at playback time from the
/// persisted Settings preference (`ui_prefs.streaming_quality`, the
/// Settings > Audio dropdown). Unset/unknown keys fall back to the
/// highest tier (Hi-Res+ = `Quality::UltraHiRes`, the previous hardcoded
/// behavior); the player still falls back internally when the requested
/// tier is not available (#590).
///
/// This returns the PURE user preference and must stay that way: any
/// device-capped variant belongs in a separate wrapper, never here.
pub(crate) fn playback_quality() -> Quality {
    crate::ui_prefs::streaming_quality_for_key(&crate::ui_prefs::load().streaming_quality)
}

/// Streaming quality for LOCAL playback plus the request-time cause: the
/// user's preference clamped by the local output device's cap when "Limit
/// quality to device" is on (#638 fix 3; cached in `crate::device_cap`, so
/// this stays as cheap as `playback_quality` plus one RwLock read). A tie
/// between the preference and the cap reports `LocalDeviceCap` — the more
/// specific, more surprising constraint.
/// A resolved Hi-Res+ request reports `None`: nothing constrained it.
pub(crate) fn local_playback_quality() -> (Quality, QualityLimit) {
    let pref = playback_quality();
    match crate::device_cap::cap() {
        Some((cap, _)) if cap < pref => (cap, QualityLimit::LocalDeviceCap),
        Some((cap, _)) if cap == pref && cap < Quality::UltraHiRes => {
            (pref, QualityLimit::LocalDeviceCap)
        }
        _ => (
            pref,
            if pref < Quality::UltraHiRes {
                QualityLimit::Preference
            } else {
                QualityLimit::None
            },
        ),
    }
}

/// Mirror the playing/paused state onto the visualizer tap so the FFT producer
/// parks while nothing plays (paused/stopped it would otherwise re-FFT the
/// stale ring buffer at 30fps — the NPB Large dock idled at ~2.5% CPU). Called
/// next to every `NowPlayingState.set_playing` flip so the producer stays
/// consistent with the UI-thread drain gate (visualizer.rs), which keys off the
/// same flag. Atomic store — safe from any thread, never blocks; a paused park
/// self-wakes within 200ms after resume (no unpark required).
pub(super) fn set_viz_paused(runtime: &Runtime, paused: bool) {
    if let Some(tap) = runtime.visualizer_tap() {
        tap.set_paused(paused);
    }
}

/// Wall-clock now in milliseconds.
pub(super) fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// `M:SS` for the elapsed string.
pub(super) fn fmt_elapsed(secs: u64) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

/// `-M:SS` for the remaining string.
pub(super) fn fmt_remaining(position: u64, duration: u64) -> String {
    let left = duration.saturating_sub(position);
    format!("-{}:{:02}", left / 60, left % 60)
}

/// Build the album-level metadata (genre, release date, quality) captured
/// when an album is fetched for playback, so `record_recent` can stamp the
/// Recently Played card with the same genre + release date + quality badge
/// the Discover carousels show. Mirrors Tauri's `album_to_card_meta`, which
/// reads these straight off the `Album`.
pub(super) fn album_card_meta(album: &qbz_models::Album) -> crate::recently::AlbumMeta {
    let genre = album
        .genre
        .as_ref()
        .map(|g| g.name.clone())
        .unwrap_or_default();
    let release_date = album.release_date_original.clone().unwrap_or_default();
    // The album summary carries its own max bit depth / sample rate, which
    // are more reliable than a single track's for the card badge.
    let (quality_tier, quality_label) =
        recent_quality(album.maximum_bit_depth, album.maximum_sampling_rate);
    crate::recently::AlbumMeta {
        genre,
        release_date,
        quality_tier,
        quality_label,
    }
}

/// Quality tier + exact-quality label from a queue track's bit depth /
/// sample rate, matching the discover card badge format.
pub(super) fn recent_quality(bit_depth: Option<u32>, sample_rate: Option<f64>) -> (String, String) {
    let tier = match bit_depth {
        Some(d) if d >= 24 => "hires",
        Some(_) => "cd",
        None => "",
    };
    let label = match (bit_depth, sample_rate) {
        (Some(bd), Some(sr)) => {
            let t = if bd >= 24 { "Hi-Res" } else { "CD" };
            let rate = if (sr.fract()).abs() < f64::EPSILON {
                format!("{}", sr as i64)
            } else {
                format!("{sr}")
            };
            format!("{t}: {bd}-bit / {rate} kHz")
        }
        _ => String::new(),
    };
    (tier.to_string(), label)
}
