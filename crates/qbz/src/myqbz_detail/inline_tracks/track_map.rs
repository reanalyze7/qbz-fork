//! `QueueTrack` -> `TrackItem` mapping used only by the inline-tracks fetch.

use crate::TrackItem;

use crate::myqbz_detail::strings::{inline_track_title, track_duration_str};

/// Map one resolved `QueueTrack` into the shared `TrackItem` the inline
/// `TrackRow`s render. Quality tier/detail are derived the same way as the
/// now-playing + album-row badges (24-bit+ = Hi-Res), so the inline badge
/// matches every other surface. `source` drives the per-source `TrackRow`
/// affordances (local rows hide the favorite + offline columns).
///
/// `resolver_index` is the 0-based position of this track in the resolver's
/// output. The resolved `QueueTrack` carries no explicit album track number, so
/// the displayed number is the resolver's order (1-based) — i.e. "use the
/// resolver's track number when present", which for this model is the resolved
/// sequence position. (`TrackRow` would otherwise index-fall-back, but baking
/// the number here keeps the row number correct regardless of the caller.)
pub(super) fn track_to_item(track: &qbz_models::QueueTrack, resolver_index: usize) -> TrackItem {
    let quality_tier = match track.bit_depth {
        Some(d) if d >= 24 => "hires",
        Some(_) => "cd",
        None if track.hires => "hires",
        None => "",
    };
    let quality_detail = if quality_tier.is_empty() {
        String::new()
    } else {
        crate::quality::detail(track.bit_depth, track.sample_rate)
    };
    let source = track
        .source
        .clone()
        .unwrap_or_else(|| if track.is_local { "local".into() } else { "qobuz".into() });

    TrackItem {
        // MyQBZ detail is out of Task 6 row-stamping scope (mixed local/Qobuz
        // entity; handled by its own epic). Never stamped here.
        is_blacklisted: false,
        id: track.id.to_string().into(),
        number: (resolver_index + 1).to_string().into(),
        title: inline_track_title(track).into(),
        artist: track.artist.clone().into(),
        album: String::new().into(),
        duration: track_duration_str(track.duration_secs).into(),
        quality_tier: quality_tier.into(),
        quality_detail: quality_detail.into(),
        explicit: track.parental_warning,
        selected: false,
        artwork_url: String::new().into(),
        artwork: slint::Image::default(),
        is_favorite: false,
        artist_id: track.artist_id.map(|id| id.to_string()).unwrap_or_default().into(),
        album_id: track.album_id.clone().unwrap_or_default().into(),
        source: source.into(),
        removing: false,
        cache_status: 0,
        cache_progress: 0.0,
        unlocking: false,
        // Disc grouping is album-detail only; flat lists carry no header.
        disc_header_number: 0,
        // Work grouping is album-detail only too.
        work_header: "".into(),
        work_composer_name: "".into(),
        work_composer_id: "".into(),
    }
}
